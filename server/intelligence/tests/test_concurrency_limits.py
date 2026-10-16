import asyncio

import pytest
from langchain_core.language_models.chat_models import BaseChatModel
from langchain_core.messages import AIMessage
from langchain_core.outputs import ChatGeneration, ChatResult

from src.agents import runtime as runtime_module
from src.agents.registry import AgentDefinition, registry
from src.agents.runtime import AutonomousAgentRuntime, current_ledger_var
from src.orchestration import limits as limits_module
from src.orchestration.limits import ConcurrencyLimits


class MemoryLedger:
    def __init__(self):
        self.events: list[tuple[str, dict]] = []

    async def append_event_transactional(self, run_id, event_type, payload=None, **kwargs):
        self.events.append((event_type, payload or {}))

    async def list_artifacts_for_run(self, run_id):
        return []

    async def save_agent_execution(self, execution):
        return None

    async def save_artifact(self, artifact):
        return None

    async def save_tool_invocation(self, invocation, run_id=None):
        return None

    async def save_delegation(self, delegation):
        return None

    async def get_agent_execution(self, run_id, execution_id):
        return None


class Gauge:
    def __init__(self):
        self.current: dict[str, int] = {}
        self.peak: dict[str, int] = {}

    def enter(self, *keys: str) -> None:
        for k in keys:
            self.current[k] = self.current.get(k, 0) + 1
            self.peak[k] = max(self.peak.get(k, 0), self.current[k])

    def leave(self, *keys: str) -> None:
        for k in keys:
            self.current[k] -= 1


class SlowFakeModel(BaseChatModel):
    gauge: Gauge
    run_id: str
    agent_id: str
    delay_s: float = 0.05
    release: asyncio.Event | None = None

    model_config = {"arbitrary_types_allowed": True}

    @property
    def _llm_type(self) -> str:
        return "slow_fake_model"

    def bind_tools(self, tools, **kwargs):
        return self

    def _generate(self, messages, stop=None, run_manager=None, **kwargs):
        raise NotImplementedError

    async def _agenerate(self, messages, stop=None, run_manager=None, **kwargs):
        keys = ("global", f"run:{self.run_id}", f"agent:{self.agent_id}", "provider")
        self.gauge.enter(*keys)
        try:
            if self.release is not None:
                await self.release.wait()
            else:
                await asyncio.sleep(self.delay_s)
        finally:
            self.gauge.leave(*keys)
        msg = AIMessage(
            content="",
            tool_calls=[
                {
                    "name": "AgentTaskOutput",
                    "args": {
                        "summary": "Done.",
                        "claims": [],
                        "tools_used": [],
                        "status": "completed",
                        "confidence": 0.9,
                    },
                    "id": "tc_out",
                    "type": "tool_call",
                }
            ],
        )
        return ChatResult(generations=[ChatGeneration(message=msg)])


PROBE_AGENTS = ("probe-alpha", "probe-beta", "probe-gamma", "probe-solo")


@pytest.fixture
def isolated_runtime(monkeypatch):
    for agent_id in PROBE_AGENTS:
        monkeypatch.setitem(
            registry._agents,
            agent_id,
            AgentDefinition(
                id=agent_id,
                name=agent_id,
                description="Concurrency probe agent",
                allowed_tools=[],
                wall_time_seconds=30,
                instructions="Call AgentTaskOutput.",
            ),
        )
    ledger = MemoryLedger()
    token = current_ledger_var.set(ledger)

    def install(**limit_kwargs) -> tuple[AutonomousAgentRuntime, ConcurrencyLimits, MemoryLedger]:
        limits = ConcurrencyLimits(**limit_kwargs)
        monkeypatch.setattr(runtime_module, "agent_limits", limits)
        monkeypatch.setattr(limits_module, "agent_limits", limits)
        return AutonomousAgentRuntime(ledger=ledger), limits, ledger

    yield install
    current_ledger_var.reset(token)


async def run_one(runtime, gauge, run_id, agent_id, release=None, delay_s=0.05):
    model = SlowFakeModel(
        gauge=gauge, run_id=run_id, agent_id=agent_id, release=release, delay_s=delay_s
    )
    return await runtime.execute_agent_loop(
        agent_id=agent_id,
        objective="Concurrency probe",
        task_id=f"task_{run_id}",
        run_id=run_id,
        input_data={},
        model_override=model,
    )


@pytest.mark.asyncio
async def test_global_per_run_and_per_agent_limits_are_never_exceeded(isolated_runtime):
    runtime, limits, _ = isolated_runtime(
        global_limit=3,
        per_run_limit=2,
        llm_inflight_limit=10,
        agent_limit=lambda a: 1 if a == "probe-solo" else 3,
    )
    gauge = Gauge()
    jobs = []
    for r in range(3):
        for _ in range(3):
            jobs.append(run_one(runtime, gauge, f"run_{r}", "probe-alpha"))
    for _ in range(3):
        jobs.append(run_one(runtime, gauge, "run_p", "probe-solo"))
    results = await asyncio.gather(*jobs)

    assert all(rec.status == "completed" for rec, _ in results)
    assert gauge.peak["global"] == 3
    for r in ("run_0", "run_1", "run_2"):
        assert gauge.peak[f"run:{r}"] == 2
    assert gauge.peak["agent:probe-solo"] == 1
    assert gauge.peak["agent:probe-alpha"] == 3
    assert limits.tracked_run_ids() == []


@pytest.mark.asyncio
async def test_provider_inflight_limit_caps_concurrent_model_calls(isolated_runtime):
    runtime, limits, _ = isolated_runtime(
        global_limit=8, per_run_limit=8, llm_inflight_limit=2, agent_limit=lambda a: 8
    )
    gauge = Gauge()
    results = await asyncio.gather(
        *(run_one(runtime, gauge, f"run_prov_{i}", "probe-alpha") for i in range(6))
    )
    assert all(rec.status == "completed" for rec, _ in results)
    assert gauge.peak["provider"] == 2
    assert gauge.peak["global"] == 2
    assert limits.tracked_run_ids() == []


@pytest.mark.asyncio
async def test_parallelism_still_happens_within_limits(isolated_runtime):
    runtime, _, _ = isolated_runtime(
        global_limit=4, per_run_limit=4, llm_inflight_limit=4, agent_limit=lambda a: 4
    )
    gauge = Gauge()
    release = asyncio.Event()
    agents = ("probe-gamma", "probe-alpha", "probe-beta")
    tasks = [
        asyncio.create_task(run_one(runtime, gauge, "run_par", a, release=release)) for a in agents
    ]
    for _ in range(200):
        if gauge.current.get("run:run_par", 0) == 3:
            break
        await asyncio.sleep(0.01)
    assert gauge.current["run:run_par"] == 3
    release.set()
    await asyncio.gather(*tasks)


@pytest.mark.asyncio
async def test_cancellation_releases_slots_and_run_map(isolated_runtime):
    runtime, limits, _ = isolated_runtime(
        global_limit=1, per_run_limit=1, llm_inflight_limit=1, agent_limit=lambda a: 1
    )
    gauge = Gauge()
    blocker = asyncio.Event()
    holder = asyncio.create_task(
        run_one(runtime, gauge, "run_cancel", "probe-alpha", release=blocker)
    )
    waiter = asyncio.create_task(run_one(runtime, gauge, "run_cancel_2", "probe-alpha"))
    for _ in range(200):
        if gauge.current.get("global", 0) == 1:
            break
        await asyncio.sleep(0.01)
    assert sorted(limits.tracked_run_ids()) == ["run_cancel", "run_cancel_2"]

    holder.cancel()
    waiter.cancel()
    for t in (holder, waiter):
        with pytest.raises(asyncio.CancelledError):
            await t
    assert limits.tracked_run_ids() == []

    rec, _ = await asyncio.wait_for(run_one(runtime, gauge, "run_after", "probe-alpha"), timeout=5)
    assert rec.status == "completed"
    assert limits.tracked_run_ids() == []


@pytest.mark.asyncio
async def test_waiting_longer_than_notice_threshold_emits_activity(isolated_runtime):
    runtime, limits, ledger = isolated_runtime(
        global_limit=1, per_run_limit=4, llm_inflight_limit=4, agent_limit=lambda a: 4
    )
    limits.wait_notice_s = 0.05
    gauge = Gauge()
    release = asyncio.Event()
    holder = asyncio.create_task(
        run_one(runtime, gauge, "run_wait", "probe-alpha", release=release)
    )
    for _ in range(200):
        if gauge.current.get("global", 0) == 1:
            break
        await asyncio.sleep(0.01)
    waiter = asyncio.create_task(run_one(runtime, gauge, "run_wait", "probe-beta"))
    await asyncio.sleep(0.2)
    release.set()
    await asyncio.gather(holder, waiter)

    waits = [
        p
        for t, p in ledger.events
        if t == "agent.activity" and p.get("activity_type") == "waiting_for_slot"
    ]
    assert len(waits) == 1
    assert "Waiting for execution slot (global limit 1)" in waits[0]["description"]


@pytest.mark.asyncio
async def test_short_waits_emit_no_activity(isolated_runtime):
    runtime, _, ledger = isolated_runtime(
        global_limit=1, per_run_limit=4, llm_inflight_limit=4, agent_limit=lambda a: 4
    )
    gauge = Gauge()
    await asyncio.gather(
        *(run_one(runtime, gauge, "run_short", "probe-alpha", delay_s=0.02) for _ in range(3))
    )
    waits = [
        p
        for t, p in ledger.events
        if t == "agent.activity" and p.get("activity_type") == "waiting_for_slot"
    ]
    assert waits == []


@pytest.mark.asyncio
async def test_nested_delegation_does_not_take_global_or_run_slots():
    limits = ConcurrencyLimits(
        global_limit=1, per_run_limit=1, llm_inflight_limit=1, agent_limit=lambda a: 1
    )

    async def child():
        async with limits.execution_slot("run_nested", "systems-deep-dive"):
            return "child-done"

    async with limits.execution_slot("run_nested", "probe-beta"):
        assert await asyncio.wait_for(child(), timeout=2) == "child-done"
    assert limits.tracked_run_ids() == []
