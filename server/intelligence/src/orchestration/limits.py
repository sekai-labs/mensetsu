import asyncio
import contextlib
import contextvars
import logging
import os
from collections.abc import AsyncIterator, Awaitable, Callable

logger = logging.getLogger("mensetsu.orchestration.limits")

WAIT_NOTICE_S = 1.0

holding_execution_slot: contextvars.ContextVar[bool] = contextvars.ContextVar(
    "holding_execution_slot", default=False
)

WaitCallback = Callable[[str], Awaitable[None]]


def env_limit(name: str, default: int) -> int:
    raw = os.environ.get(name, "").strip()
    if not raw:
        return default
    value = int(raw)
    if value < 1:
        raise ValueError(f"{name} must be >= 1, got {value}")
    return value


def registry_agent_limit(agent_id: str) -> int:
    from src.agents.registry import registry

    return registry.get(agent_id).max_concurrency


def provider_key(model: object) -> str:
    base_url = getattr(model, "openai_api_base", None) or ""
    return f"{type(model).__name__}:{base_url}"


class RunSlot:
    def __init__(self, limit: int):
        self.semaphore = asyncio.Semaphore(limit)
        self.users = 0


async def acquire_with_notice(
    sem: asyncio.Semaphore, label: str, on_wait: WaitCallback | None, notice_after_s: float
) -> None:
    if on_wait is None or not sem.locked():
        await sem.acquire()
        return
    fired = asyncio.Event()

    async def notice() -> None:
        await asyncio.sleep(notice_after_s)
        fired.set()
        try:
            await on_wait(label)
        except Exception as e:
            logger.warning(f"Execution slot wait notice failed ({label}): {e}")

    notice_task = asyncio.create_task(notice())
    try:
        await sem.acquire()
    except BaseException:
        notice_task.cancel()
        raise
    if fired.is_set():
        await notice_task
        return
    notice_task.cancel()
    with contextlib.suppress(asyncio.CancelledError):
        await notice_task


@contextlib.asynccontextmanager
async def held(
    sem: asyncio.Semaphore, label: str, on_wait: WaitCallback | None, notice_after_s: float
) -> AsyncIterator[None]:
    await acquire_with_notice(sem, label, on_wait, notice_after_s)
    try:
        yield
    finally:
        sem.release()


class ConcurrencyLimits:
    def __init__(
        self,
        global_limit: int | None = None,
        per_run_limit: int | None = None,
        llm_inflight_limit: int | None = None,
        agent_limit: Callable[[str], int] | None = None,
        wait_notice_s: float = WAIT_NOTICE_S,
    ):
        self.global_limit = global_limit or env_limit("MAX_CONCURRENT_AGENTS", 8)
        self.per_run_limit = per_run_limit or env_limit("MAX_CONCURRENT_AGENTS_PER_RUN", 4)
        self.llm_inflight_limit = llm_inflight_limit or env_limit("LLM_MAX_INFLIGHT", 6)
        self.agent_limit = agent_limit or registry_agent_limit
        self.wait_notice_s = wait_notice_s
        self._loop: asyncio.AbstractEventLoop | None = None
        self._reset_state()

    def _reset_state(self) -> None:
        self._global = asyncio.Semaphore(self.global_limit)
        self._runs: dict[str, RunSlot] = {}
        self._agents: dict[str, asyncio.Semaphore] = {}
        self._providers: dict[str, asyncio.Semaphore] = {}

    def _bind_loop(self) -> None:
        loop = asyncio.get_running_loop()
        if loop is not self._loop:
            self._loop = loop
            self._reset_state()

    def tracked_run_ids(self) -> list[str]:
        return list(self._runs)

    def _agent_semaphore(self, agent_id: str) -> tuple[asyncio.Semaphore, int]:
        limit = self.agent_limit(agent_id)
        sem = self._agents.get(agent_id)
        if sem is None:
            sem = asyncio.Semaphore(limit)
            self._agents[agent_id] = sem
        return sem, limit

    def _claim_run_slot(self, run_id: str) -> RunSlot:
        run_slot = self._runs.get(run_id)
        if run_slot is None:
            run_slot = RunSlot(self.per_run_limit)
            self._runs[run_id] = run_slot
        run_slot.users += 1
        return run_slot

    def _release_run_slot(self, run_id: str, run_slot: RunSlot) -> None:
        run_slot.users -= 1
        if run_slot.users == 0 and self._runs.get(run_id) is run_slot:
            del self._runs[run_id]

    @contextlib.asynccontextmanager
    async def execution_slot(
        self, run_id: str, agent_id: str, on_wait: WaitCallback | None = None
    ) -> AsyncIterator[None]:
        self._bind_loop()
        agent_sem, agent_max = self._agent_semaphore(agent_id)
        nested = holding_execution_slot.get()
        run_slot = None if nested else self._claim_run_slot(run_id)
        notice_s = self.wait_notice_s
        try:
            async with contextlib.AsyncExitStack() as stack:
                if run_slot is not None:
                    await stack.enter_async_context(
                        held(self._global, f"global limit {self.global_limit}", on_wait, notice_s)
                    )
                    await stack.enter_async_context(
                        held(run_slot.semaphore, f"per-run limit {self.per_run_limit}", on_wait, notice_s)
                    )
                await stack.enter_async_context(
                    held(agent_sem, f"{agent_id} limit {agent_max}", on_wait, notice_s)
                )
                token = holding_execution_slot.set(True)
                try:
                    yield
                finally:
                    holding_execution_slot.reset(token)
        finally:
            if run_slot is not None:
                self._release_run_slot(run_id, run_slot)

    @contextlib.asynccontextmanager
    async def provider_slot(
        self, provider: str, wait_scope: Callable[[], contextlib.AbstractContextManager] | None = None
    ) -> AsyncIterator[None]:
        self._bind_loop()
        sem = self._providers.get(provider)
        if sem is None:
            sem = asyncio.Semaphore(self.llm_inflight_limit)
            self._providers[provider] = sem
        if sem.locked() and wait_scope is not None:
            with wait_scope():
                await sem.acquire()
        else:
            await sem.acquire()
        try:
            yield
        finally:
            sem.release()


agent_limits = ConcurrencyLimits()
