import asyncio
import contextlib
import uuid

import pytest
from langchain_core.language_models.chat_models import BaseChatModel
from langchain_core.messages import AIMessage, ToolMessage
from langchain_core.outputs import ChatGeneration, ChatResult

from src.agents import runtime as runtime_module
from src.agents.registry import AgentDefinition, registry
from src.agents.runtime import AgentExecutionError, AutonomousAgentRuntime, current_agent_id_var
from src.contracts.v1 import Artifact, WorkflowRun
from src.orchestration.ledger import SurrealLedgerRepository
from src.orchestration.limits import ConcurrencyLimits

PARENT = "probe-parent"
CHILD = "probe-child"


def final_output() -> AIMessage:
    return AIMessage(
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
                "id": f"tc_out_{uuid.uuid4().hex[:6]}",
                "type": "tool_call",
            }
        ],
    )


def tool_call(name: str, args: dict) -> AIMessage:
    return AIMessage(
        content="",
        tool_calls=[
            {"name": name, "args": args, "id": f"tc_{uuid.uuid4().hex[:6]}", "type": "tool_call"}
        ],
    )


class LifecycleModel(BaseChatModel):
    parent_pre_delegate_sleep_s: float = 0.0
    parent_post_delegate_sleep_s: float = 0.0
    child_sleep_s: float = 0.0
    tool_results: list[str] = []

    @property
    def _llm_type(self) -> str:
        return "lifecycle_model"

    def bind_tools(self, tools, **kwargs):
        return self

    def _generate(self, messages, stop=None, run_manager=None, **kwargs):
        raise NotImplementedError

    async def _agenerate(self, messages, stop=None, run_manager=None, **kwargs):
        agent = current_agent_id_var.get()
        tool_msgs = [m for m in messages if isinstance(m, ToolMessage)]
        if agent == CHILD:
            await asyncio.sleep(self.child_sleep_s)
            msg = final_output()
        elif not tool_msgs:
            await asyncio.sleep(self.parent_pre_delegate_sleep_s)
            msg = tool_call(
                "delegate_task",
                {"target_agent_id": CHILD, "objective": "Deep dive", "reason": "Needs depth"},
            )
        else:
            self.tool_results.append(str(tool_msgs[-1].content))
            await asyncio.sleep(self.parent_post_delegate_sleep_s)
            msg = final_output()
        return ChatResult(generations=[ChatGeneration(message=msg)])


@pytest.fixture
def lifecycle(monkeypatch):
    def register(parent_wall_s: float, child_wall_s: float) -> None:
        monkeypatch.setitem(
            registry._agents,
            PARENT,
            AgentDefinition(
                id=PARENT,
                name=PARENT,
                description="Lifecycle parent",
                allowed_tools=["delegate_task"],
                delegation_permissions=[CHILD],
                wall_time_seconds=parent_wall_s,
                instructions="Delegate, then call AgentTaskOutput.",
            ),
        )
        monkeypatch.setitem(
            registry._agents,
            CHILD,
            AgentDefinition(
                id=CHILD,
                name=CHILD,
                description="Lifecycle child",
                allowed_tools=[],
                wall_time_seconds=child_wall_s,
                instructions="Call AgentTaskOutput.",
            ),
        )

    monkeypatch.setattr(runtime_module, "agent_limits", ConcurrencyLimits(agent_limit=lambda a: 4))
    return register


async def new_run() -> tuple[SurrealLedgerRepository, str]:
    ledger = SurrealLedgerRepository()
    run_id = f"run_life_{uuid.uuid4().hex[:8]}"
    await ledger.create_workflow_run(
        WorkflowRun(
            id=run_id,
            workflow_id="interview-preparation-static",
            workflow_name="Lifecycle",
            workspace_id=f"ws_{run_id}",
            user_id="usr_life",
            type="static",
            status="running",
        )
    )
    return ledger, run_id


async def all_events(ledger: SurrealLedgerRepository, run_id: str):
    events, after = [], 0
    while True:
        page = await ledger.get_events_history(run_id=run_id, after_seq=after, limit=100)
        events.extend(page.events)
        if page.next_after_seq is None:
            return events
        after = page.next_after_seq


class ChildBlocksForeverModel(LifecycleModel):
    async def _agenerate(self, messages, stop=None, run_manager=None, **kwargs):
        if current_agent_id_var.get() == CHILD:
            await asyncio.sleep(self.child_sleep_s)
            return ChatResult(generations=[ChatGeneration(message=final_output())])
        if not [m for m in messages if isinstance(m, ToolMessage)]:
            return ChatResult(
                generations=[
                    ChatGeneration(
                        message=tool_call(
                            "delegate_task",
                            {
                                "target_agent_id": CHILD,
                                "objective": "Deep dive",
                                "reason": "Needs depth",
                            },
                        )
                    )
                ]
            )
        return ChatResult(generations=[ChatGeneration(message=final_output())])


async def assert_child_and_delegation_closed(ledger, run_id, reason_fragment):
    execs = await ledger.list_agent_executions_for_run(run_id)
    child = next(e for e in execs if e.agent_id == CHILD)
    assert child.status == "cancelled"
    assert reason_fragment in (child.error or "")
    delegations = await ledger.list_delegations_for_run(run_id)
    assert len(delegations) == 1 and delegations[0].status == "failed"
    events = await all_events(ledger, run_id)
    assert any(e.type == "agent.failed" and e.execution_id == child.id for e in events)
    assert any(e.type == "delegation.failed" for e in events)


@pytest.mark.asyncio
async def test_parent_wall_timeout_closes_delegated_child_and_delegation(lifecycle, monkeypatch):
    lifecycle(parent_wall_s=2, child_wall_s=30)
    monkeypatch.setattr(runtime_module, "DELEGATION_PARENT_RESERVE_S", -60.0)
    ledger, run_id = await new_run()
    model = ChildBlocksForeverModel(child_sleep_s=10.0)
    runtime = AutonomousAgentRuntime(ledger=ledger)

    real_paused = runtime_module.ExecutionBudgetClock.paused
    monkeypatch.setattr(
        runtime_module.ExecutionBudgetClock,
        "paused",
        lambda self, reason: (
            contextlib.nullcontext()
            if reason == "delegated child wait"
            else real_paused(self, reason)
        ),
    )
    started = asyncio.get_running_loop().time()
    with pytest.raises(AgentExecutionError, match="wall-time budget exceeded"):
        await runtime.execute_agent_loop(
            agent_id=PARENT,
            objective="Parent",
            task_id="task_life",
            run_id=run_id,
            input_data={},
            model_override=model,
        )
    assert asyncio.get_running_loop().time() - started < 8.0
    await assert_child_and_delegation_closed(
        ledger, run_id, "ended before delegated child finished"
    )


@pytest.mark.asyncio
async def test_parent_cancellation_closes_delegated_child_and_delegation(lifecycle, monkeypatch):
    lifecycle(parent_wall_s=60, child_wall_s=30)
    monkeypatch.setattr(runtime_module, "DELEGATION_PARENT_RESERVE_S", -60.0)
    ledger, run_id = await new_run()
    model = ChildBlocksForeverModel(child_sleep_s=10.0)
    runtime = AutonomousAgentRuntime(ledger=ledger)
    parent = asyncio.create_task(
        runtime.execute_agent_loop(
            agent_id=PARENT,
            objective="Parent",
            task_id="task_life2",
            run_id=run_id,
            input_data={},
            model_override=model,
        )
    )
    for _ in range(300):
        if any(e.agent_id == CHILD for e in await ledger.list_agent_executions_for_run(run_id)):
            break
        await asyncio.sleep(0.05)
    await asyncio.sleep(0.3)
    parent.cancel()
    with pytest.raises(asyncio.CancelledError):
        await parent
    await assert_child_and_delegation_closed(
        ledger, run_id, "ended before delegated child finished"
    )


@pytest.mark.asyncio
async def test_late_delegation_is_refused(lifecycle):
    lifecycle(parent_wall_s=60, child_wall_s=240)
    ledger, run_id = await new_run()
    model = LifecycleModel()
    exec_rec, _ = await AutonomousAgentRuntime(ledger=ledger).execute_agent_loop(
        agent_id=PARENT,
        objective="Parent",
        task_id="task_late",
        run_id=run_id,
        input_data={},
        model_override=model,
    )
    assert exec_rec.status == "completed"
    assert any(
        "Not enough budget left to delegate; finish your research" in r for r in model.tool_results
    )
    assert await ledger.list_delegations_for_run(run_id) == []
    assert all(e.agent_id != CHILD for e in await ledger.list_agent_executions_for_run(run_id))


@pytest.mark.asyncio
async def test_child_wait_does_not_consume_parent_budget(lifecycle, monkeypatch):
    lifecycle(parent_wall_s=6, child_wall_s=30)
    monkeypatch.setattr(runtime_module, "DELEGATION_PARENT_RESERVE_S", -60.0)
    ledger, run_id = await new_run()
    model = LifecycleModel(
        parent_pre_delegate_sleep_s=0.5, child_sleep_s=12.0, parent_post_delegate_sleep_s=0.5
    )
    exec_rec, _ = await AutonomousAgentRuntime(ledger=ledger).execute_agent_loop(
        agent_id=PARENT,
        objective="Parent",
        task_id="task_pause",
        run_id=run_id,
        input_data={},
        model_override=model,
    )
    assert exec_rec.status == "completed"
    assert exec_rec.budgets_used.wall_time_ms > 12000
    child = next(
        e for e in await ledger.list_agent_executions_for_run(run_id) if e.agent_id == CHILD
    )
    assert child.status == "completed"
    assert (await ledger.list_delegations_for_run(run_id))[0].status == "completed"


@pytest.mark.asyncio
async def test_llm_retry_backoff_does_not_consume_wall_budget(lifecycle, monkeypatch, caplog):
    lifecycle(parent_wall_s=3, child_wall_s=10)
    monkeypatch.setattr(runtime_module, "LLM_RETRY_DELAYS_S", (4.0,))
    ledger, run_id = await new_run()
    attempts = 0

    class FlakyRetryModel(BaseChatModel):
        @property
        def _llm_type(self) -> str:
            return "flaky_retry_model"

        def bind_tools(self, tools, **kwargs):
            return self

        def _generate(self, messages, stop=None, run_manager=None, **kwargs):
            raise NotImplementedError

        async def _agenerate(self, messages, stop=None, run_manager=None, **kwargs):
            nonlocal attempts
            attempts += 1
            if attempts == 1:
                raise TimeoutError("Simulated LLM call timeout")
            return ChatResult(generations=[ChatGeneration(message=final_output())])

    with caplog.at_level("INFO", logger="mensetsu.agents.runtime"):
        exec_rec, _ = await AutonomousAgentRuntime(ledger=ledger).execute_agent_loop(
            agent_id=PARENT,
            objective="Parent retry test",
            task_id="task_retry_pause",
            run_id=run_id,
            input_data={},
            model_override=FlakyRetryModel(),
        )
    assert exec_rec.status == "completed"
    assert attempts == 2
    assert exec_rec.budgets_used.wall_time_ms >= 4000
    messages = [r.getMessage() for r in caplog.records]
    assert any("llm retry backoff took" in m and "deadline paused" in m for m in messages)
    assert any("excluded from wall budget" in m and "llm retry backoff" in m for m in messages)


@pytest.mark.asyncio
async def test_execution_slot_wait_does_not_consume_wall_budget(lifecycle):
    lifecycle(parent_wall_s=3, child_wall_s=10)
    ledger, run_id = await new_run()
    limits = runtime_module.agent_limits
    limits._bind_loop()
    parent_sem, limit = limits._agent_semaphore(PARENT)
    for _ in range(limit):
        await parent_sem.acquire()

    async def delayed_release():
        await asyncio.sleep(4.0)
        parent_sem.release()

    release_task = asyncio.create_task(delayed_release())

    class FastModel(BaseChatModel):
        @property
        def _llm_type(self) -> str:
            return "fast_model"

        def bind_tools(self, tools, **kwargs):
            return self

        def _generate(self, messages, stop=None, run_manager=None, **kwargs):
            raise NotImplementedError

        async def _agenerate(self, messages, stop=None, run_manager=None, **kwargs):
            return ChatResult(generations=[ChatGeneration(message=final_output())])

    started = asyncio.get_running_loop().time()
    exec_rec, _ = await AutonomousAgentRuntime(ledger=ledger).execute_agent_loop(
        agent_id=PARENT,
        objective="Slot wait test",
        task_id="task_slot_wait",
        run_id=run_id,
        input_data={},
        model_override=FastModel(),
    )
    await release_task
    assert asyncio.get_running_loop().time() - started >= 3.8
    assert exec_rec.status == "completed"
    assert exec_rec.budgets_used.wall_time_ms < 3000


@pytest.mark.asyncio
async def test_provider_slot_wait_does_not_consume_wall_budget(lifecycle, monkeypatch, caplog):
    lifecycle(parent_wall_s=3, child_wall_s=10)
    monkeypatch.setattr(
        runtime_module,
        "agent_limits",
        ConcurrencyLimits(llm_inflight_limit=1, agent_limit=lambda a: 4),
    )
    ledger, run_id = await new_run()
    limits = runtime_module.agent_limits
    limits._bind_loop()

    class FastModel(BaseChatModel):
        @property
        def _llm_type(self) -> str:
            return "fast_model"

        def bind_tools(self, tools, **kwargs):
            return self

        def _generate(self, messages, stop=None, run_manager=None, **kwargs):
            raise NotImplementedError

        async def _agenerate(self, messages, stop=None, run_manager=None, **kwargs):
            return ChatResult(generations=[ChatGeneration(message=final_output())])

    model = FastModel()
    provider_sem = asyncio.Semaphore(1)
    limits._providers[runtime_module.provider_key(model)] = provider_sem
    await provider_sem.acquire()

    async def delayed_release():
        await asyncio.sleep(4.0)
        provider_sem.release()

    release_task = asyncio.create_task(delayed_release())
    started = asyncio.get_running_loop().time()
    with caplog.at_level("INFO", logger="mensetsu.agents.runtime"):
        exec_rec, _ = await AutonomousAgentRuntime(ledger=ledger).execute_agent_loop(
            agent_id=PARENT,
            objective="Provider wait test",
            task_id="task_provider_wait",
            run_id=run_id,
            input_data={},
            model_override=model,
        )
    await release_task
    assert asyncio.get_running_loop().time() - started >= 3.8
    assert exec_rec.status == "completed"
    messages = [r.getMessage() for r in caplog.records]
    assert any("provider slot wait took" in m and "deadline paused" in m for m in messages)
    assert any("excluded from wall budget" in m and "provider slot wait" in m for m in messages)


@pytest.mark.asyncio
async def test_run_end_sweep_closes_open_records():
    from src.contracts.v1 import AgentExecution, Delegation, ToolInvocation

    ledger, run_id = await new_run()
    for eid, agent in (("exec_sweep_p", PARENT), ("exec_sweep_c", CHILD)):
        await ledger.save_agent_execution(
            AgentExecution(
                id=eid, run_id=run_id, task_id="task_sweep", agent_id=agent, status="running"
            )
        )
    await ledger.save_delegation(
        Delegation(
            id="del_sweep_1",
            run_id=run_id,
            parent_execution_id="exec_sweep_p",
            child_execution_id="exec_sweep_c",
            parent_agent_id=PARENT,
            child_agent_id=CHILD,
            delegation_task_id="task_sweep",
            objective="o",
            reason="r",
            depth=1,
            status="running",
        )
    )
    await ledger.save_tool_invocation(
        ToolInvocation(
            id="tool_sweep_1", execution_id="exec_sweep_c", tool_name="web_search", status="running"
        ),
        run_id=run_id,
    )

    counts = await ledger.sweep_run_to_terminal(run_id, reason="run failed")
    assert counts == {"executions": 2, "tools": 1, "delegations": 1}
    assert {e.status for e in await ledger.list_agent_executions_for_run(run_id)} == {"cancelled"}
    assert (await ledger.list_delegations_for_run(run_id))[0].status == "failed"
    assert (await ledger.get_tool_invocation(run_id, "tool_sweep_1")).status == "failed"
    types = [e.type for e in await all_events(ledger, run_id)]
    assert types.count("agent.failed") == 2
    assert "delegation.failed" in types and "tool.failed" in types
    assert await ledger.sweep_run_to_terminal(run_id, reason="again") == {
        "executions": 0,
        "tools": 0,
        "delegations": 0,
    }


@pytest.mark.asyncio
async def test_save_finding_links_existing_evidence_and_rejects_unknown(lifecycle):
    lifecycle(parent_wall_s=60, child_wall_s=30)
    ledger, run_id = await new_run()
    await ledger.save_artifact(
        Artifact(
            id="ev_known_1", run_id=run_id, type="evidence", title="E", content={"snippet": "s"}
        )
    )
    from src.contracts.v1 import AgentExecution

    runtime = AutonomousAgentRuntime(ledger=ledger)
    execution = AgentExecution(
        id="exec_find_1", run_id=run_id, task_id="task_find", agent_id=PARENT
    )
    tools = {
        t.name: t
        for t in runtime.create_tools_for_execution(
            execution=execution,
            run_id=run_id,
            depth=0,
            ancestors=[PARENT],
            tool_call_callback=None,
            model_override=None,
        )
    }
    ok = await tools["save_finding"].ainvoke(
        {
            "category": "architecture",
            "claim": "Uses Raft",
            "confidence": 0.9,
            "evidence_ids": ["ev_known_1"],
        }
    )
    assert "linked to ev_known_1" in ok
    bad = await tools["save_finding"].ainvoke(
        {
            "category": "architecture",
            "claim": "Uses Paxos",
            "confidence": 0.9,
            "evidence_ids": ["ev_missing"],
        }
    )
    assert "do not exist in this run" in bad
    findings = [e for e in await all_events(ledger, run_id) if e.type == "finding.saved"]
    assert len(findings) == 1 and findings[0].payload.evidence_ids == ["ev_known_1"]
