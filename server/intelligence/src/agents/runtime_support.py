"""LLM configuration scope, retry policy, and execution budget primitives."""
import asyncio
import contextlib
import contextvars
import logging
from dataclasses import dataclass
from typing import Any, Callable, Iterator

import openai
from langchain.agents.middleware import wrap_model_call
from langchain_core.language_models.chat_models import BaseChatModel

from src.orchestration import limits as limits_module
from src.orchestration.ledger import SurrealLedgerRepository

current_agent_id_var: contextvars.ContextVar[str | None] = contextvars.ContextVar(
    "current_agent_id", default=None
)
current_ledger_var: contextvars.ContextVar[SurrealLedgerRepository | None] = contextvars.ContextVar(
    "current_ledger", default=None
)
current_lost_lease_var: contextvars.ContextVar[asyncio.Event | None] = contextvars.ContextVar(
    "current_lost_lease", default=None
)
current_model_factory_var: contextvars.ContextVar[Callable[[], BaseChatModel] | None] = (
    contextvars.ContextVar("current_model_factory", default=None)
)

current_llm_config_var: contextvars.ContextVar[dict[str, Any] | None] = contextvars.ContextVar(
    "current_llm_config", default=None
)


@contextlib.contextmanager
def llm_config_scope(config: dict[str, Any] | None) -> Iterator[None]:
    token = current_llm_config_var.set(config)
    try:
        yield
    finally:
        current_llm_config_var.reset(token)


def has_explicit_llm_config(config: dict[str, Any] | None, role: str) -> bool:
    if not config:
        return False

    from src.agents.model_router import ROLE_SLOT_MAP

    routing = config.get("model_routing") or {}
    slot_name = ROLE_SLOT_MAP.get(role, role)
    if routing.get(slot_name) or routing.get(role):
        return True

    legacy = config.get("legacy") or {}
    if legacy:
        return True

    provider_key = legacy.get("provider") or "openai"
    for source in ("user_providers", "system_providers"):
        provider = (config.get(source) or {}).get(provider_key) or {}
        if provider.get("api_key") or provider.get("base_url"):
            return True
    return False


logger = logging.getLogger("mensetsu.agents.runtime")

MAX_DELEGATION_DEPTH = 1
MAX_DELEGATION_CHILDREN = 2
MAX_CONSECUTIVE_NOT_FOUND = 3

FINALIZE_RESERVED_TOOL_CALLS = 2
FINALIZE_RESERVED_MODEL_CALLS = 3
FINALIZE_ACTIVE_BUDGET_FRACTION = 0.75
FINALIZE_TIME_RATIO = 1.0 - FINALIZE_ACTIVE_BUDGET_FRACTION
FINALIZE_MIN_WALL_TIME_RESERVE_S = 90.0
FINALIZE_MODEL_CALL_HEADROOM = 4
MAX_MISSING_OUTPUT_TOOL_NUDGES = 3
DELEGATION_PARENT_RESERVE_S = 90.0
SEARCH_QUOTA_BASE = 6
SEARCH_QUOTA_PER_SAVE = 4
SEARCH_QUOTA_MIN_SAVES = 2
EVIDENCE_FLOOR_BY_AGENT = {
    "company-research": 4,
    "technical-research": 4,
    "job-analysis": 1,
    "systems-deep-dive": 2,
}


@dataclass(frozen=True, slots=True)
class AgentRecoveryContext:
    attempt: int
    previous_error: str
    alternative_strategy: str


RECOVERY_STRATEGIES = {
    "company-research": (
        "Change source-finding method: pivot between targeted search and supplied official URLs, "
        "reuse saved evidence, and stop repeating inaccessible sources. Preserve provenance and "
        "state evidence gaps instead of inventing company facts."
    ),
    "technical-research": (
        "Change the research path: narrow the question, pivot source types or search terms, and "
        "reuse saved evidence. Separate sourced facts from clearly labeled inference."
    ),
    "job-analysis": (
        "Re-parse the supplied job description by section and extract only requirements supported "
        "by its text; do not use external assumptions."
    ),
    "planner": (
        "Rebuild the plan with a materially different task decomposition while retaining required "
        "deliverable coverage, allowed agents, and DAG limits."
    ),
    "evidence-evaluator": (
        "Re-evaluate using a claim-to-evidence matrix, omit unsupported claims, and retain the same "
        "match-analysis sections."
    ),
    "curriculum": (
        "Rebuild the graph or lesson in smaller steps, preserve canonical skill names and evidence "
        "links, and keep the same structured curriculum contract."
    ),
    "interview-generation": (
        "Use a different question angle, select skills only from the supplied canonical list, and "
        "preserve the requested category, count, and structured rubric."
    ),
}


def build_agent_recovery_context(
    agent_id: str, previous_error: BaseException | str, attempt: int
) -> AgentRecoveryContext:
    if attempt < 1:
        raise ValueError("Recovery attempt must be positive.")
    diagnostic = " ".join(str(previous_error).split())[:1200]
    strategy = RECOVERY_STRATEGIES.get(
        agent_id,
        "Choose a materially different allowed method or narrower decomposition while preserving "
        "the same objective and structured output contract.",
    )
    return AgentRecoveryContext(
        attempt=attempt,
        previous_error=diagnostic or "The previous execution did not complete.",
        alternative_strategy=strategy,
    )


LLM_REQUEST_TIMEOUT_S = 90.0
LLM_RETRY_DELAYS_S = (5.0, 15.0)
LLM_MAX_SERVER_RETRY_AFTER_S = 20.0
LLM_RETRYABLE_STATUS = {408, 409, 425, 429, 524}


class AgentExecutionError(RuntimeError):
    pass


def best_matching_excerpt(document: str, snippet: str, width: int = 300) -> str:
    flat_doc = " ".join(document.split())
    if not flat_doc:
        return ""
    words = [w for w in " ".join(snippet.split()).lower().split(" ") if len(w) > 3]
    lowered = flat_doc.lower()
    best_pos, best_hits = 0, -1
    step = max(1, width // 3)
    for start in range(0, max(1, len(flat_doc) - width + 1), step):
        window = lowered[start : start + width]
        hits = sum(1 for w in words if w in window)
        if hits > best_hits:
            best_pos, best_hits = start, hits
    return flat_doc[best_pos : best_pos + width]


def is_retryable_llm_error(err: BaseException) -> bool:
    if isinstance(err, (asyncio.TimeoutError, TimeoutError, openai.APITimeoutError, openai.APIConnectionError)):
        return True
    if isinstance(err, openai.RateLimitError):
        return True
    if isinstance(err, openai.APIStatusError):
        return err.status_code in LLM_RETRYABLE_STATUS or err.status_code >= 500
    return False


def llm_retry_delay(err: BaseException, default_delay: float) -> float:
    response = getattr(err, "response", None)
    headers = getattr(response, "headers", None) or {}
    raw = headers.get("retry-after") if hasattr(headers, "get") else None
    try:
        server_delay = float(raw) if raw is not None else None
    except ValueError:
        server_delay = None
    if server_delay is not None and 0 < server_delay <= LLM_MAX_SERVER_RETRY_AFTER_S:
        return max(server_delay, default_delay)
    return default_delay


def build_llm_retry_middleware(label: str):
    @wrap_model_call
    async def llm_retry_middleware(request, handler):
        for attempt, delay in enumerate((*LLM_RETRY_DELAYS_S, None), start=1):
            try:
                return await handler(request)
            except Exception as llm_err:
                if delay is None or not is_retryable_llm_error(llm_err):
                    raise
                wait_s = llm_retry_delay(llm_err, delay)
                logger.warning(
                    f"{label}: model call attempt {attempt} failed "
                    f"({llm_err.__class__.__name__}: {str(llm_err)[:200]}); retrying in {wait_s:.0f}s"
                )
                with budget_pause("llm retry backoff"):
                    await asyncio.sleep(wait_s)
        raise RuntimeError("Unreachable model retry loop")

    return llm_retry_middleware


class ExecutionBudgetClock:
    def __init__(self, label: str, budget_s: float):
        self.label = label
        self.budget_s = budget_s
        self.timeout_cm: asyncio.Timeout | None = None
        self.started: float | None = None
        self.excluded_by_reason: dict[str, float] = {}
        self._pause_depth = 0
        self._pause_started: float | None = None
        self._pause_reason = ""

    def start(self, timeout_cm: asyncio.Timeout) -> None:
        self.timeout_cm = timeout_cm
        self.started = asyncio.get_running_loop().time()

    @property
    def excluded_s(self) -> float:
        return sum(self.excluded_by_reason.values())

    def active_elapsed_s(self) -> float:
        if self.started is None:
            return 0.0
        now = asyncio.get_running_loop().time()
        paused_now = now - self._pause_started if self._pause_started is not None else 0.0
        return max(0.0, now - self.started - self.excluded_s - paused_now)

    def remaining_s(self) -> float:
        return self.budget_s - self.active_elapsed_s()

    @contextlib.contextmanager
    def paused(self, reason: str):
        loop = asyncio.get_running_loop()
        cm = self.timeout_cm
        if self._pause_depth == 0:
            self._pause_started = loop.time()
            self._pause_reason = reason
            if cm is not None and cm.when() is not None and not cm.expired():
                cm.reschedule(None)
        self._pause_depth += 1
        try:
            yield
        finally:
            self._pause_depth -= 1
            if self._pause_depth == 0 and self._pause_started is not None:
                waited = loop.time() - self._pause_started
                outer_reason = self._pause_reason
                self._pause_started = None
                self.excluded_by_reason[outer_reason] = self.excluded_by_reason.get(outer_reason, 0.0) + waited
                if cm is not None and not cm.expired():
                    cm.reschedule(loop.time() + max(0.0, self.remaining_s()))
                if waited >= 1.0:
                    logger.info(f"{self.label}: {outer_reason} took {waited:.1f}s; deadline paused for it")


current_budget_clock_var: contextvars.ContextVar[ExecutionBudgetClock | None] = contextvars.ContextVar(
    "current_budget_clock", default=None
)


def budget_pause(reason: str):
    clock = current_budget_clock_var.get()
    return clock.paused(reason) if clock is not None else contextlib.nullcontext()


def build_provider_slot_middleware(provider: str):
    @wrap_model_call
    async def provider_slot_middleware(request, handler):
        async with limits_module.agent_limits.provider_slot(provider, wait_scope=lambda: budget_pause("provider slot wait")):
            return await handler(request)

    return provider_slot_middleware
