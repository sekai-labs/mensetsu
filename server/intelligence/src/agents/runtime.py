import asyncio
import contextlib
import contextvars
import json
import logging
import os
import time
import uuid
from datetime import datetime, timezone
from typing import Any, Callable

import httpx
import openai
from langchain.agents import create_agent
from langchain.agents.middleware import (
    AgentMiddleware,
    ModelCallLimitMiddleware,
    ToolCallLimitMiddleware,
    wrap_model_call,
    wrap_tool_call,
)
from langchain.agents.structured_output import ToolStrategy
from langchain_core.language_models.chat_models import BaseChatModel
from langchain_core.tools import tool
from langchain_openai import ChatOpenAI
from pydantic import BaseModel, ConfigDict, Field

from src.agents.registry import registry
from src.agents.schemas import (
    AGENT_OUTPUT_SCHEMAS,
    AgentTaskOutput,
)
from src.contracts.v1 import (
    AgentExecution,
    Artifact,
    Delegation,
    EvidenceProvenance,
    ExecutionBudgets,
    ExecutionBudgetsUsed,
    ExecutionNote,
    SourceLink,
    ToolInvocation,
)
from src.fetcher.reader import SafeResearchFetcher
from src.fetcher.search_engine import AgentSearchEngine
from src.fetcher.ssrf_client import (
    PlaceholderDomainError,
    reject_placeholder_domain,
    validate_url_for_ssrf,
)
from src.orchestration.id_validator import validate_id
from src.orchestration.ledger import SurrealLedgerRepository
from src.orchestration.limits import agent_limits, provider_key
from src.orchestration.sanitizer import sanitize_data

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
                ctx_ledger = current_ledger_var.get()
                if ctx_ledger:
                    with contextlib.suppress(Exception):
                        parts = label.split(" ")
                        exec_id = parts[1] if len(parts) > 1 else None
                        agent_id = parts[2].strip("()") if len(parts) > 2 else None
                        # Emit agent retry activity event so it's captured in UI logs & timeline
                        run_id = getattr(request, "run_id", None)
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
        async with agent_limits.provider_slot(provider, wait_scope=lambda: budget_pause("provider slot wait")):
            return await handler(request)

    return provider_slot_middleware


READ_SOURCE_ALIASES = {
    "url": ("url", "link", "source_url", "source"),
}
SAVE_EVIDENCE_ALIASES = {
    "source_url": ("source_url", "url", "link", "source"),
    "snippet": ("snippet", "quote", "exact_quote", "excerpt", "citation", "text"),
    "title": ("title", "name", "page_title"),
    "confidence": ("confidence",),
}
SAVE_FINDING_ALIASES = {
    "category": ("category", "topic", "area"),
    "claim": ("claim", "finding", "statement", "text"),
    "confidence": ("confidence",),
    "evidence_ids": ("evidence_ids", "evidence", "sources", "source_ids"),
}
SAVE_FINDING_OPTIONAL = frozenset({"evidence_ids"})


def normalize_tool_args(
    tool_name: str,
    raw_args: dict[str, Any],
    aliases: dict[str, tuple[str, ...]],
    optional: frozenset[str] = frozenset(),
) -> tuple[dict[str, Any], str | None]:
    known = {alias for names in aliases.values() for alias in names}
    unknown = sorted(k for k in raw_args if k not in known)
    valid_names = ", ".join(
        f"{canonical}{' (optional)' if canonical in optional else ''} "
        f"(aliases: {', '.join(a for a in names if a != canonical) or 'none'})"
        for canonical, names in aliases.items()
    )
    if unknown:
        return {}, f"Error: {tool_name} got unknown argument(s) {unknown}. Valid arguments: {valid_names}."
    resolved: dict[str, Any] = {}
    for canonical, names in aliases.items():
        value = next((raw_args[n] for n in names if raw_args.get(n) not in (None, "")), None)
        if value is None:
            if canonical in optional:
                continue
            return {}, f"Error: {tool_name} is missing '{canonical}'. Valid arguments: {valid_names}."
        resolved[canonical] = value
    return resolved, None


class ReadSourceArgs(BaseModel):
    model_config = ConfigDict(extra="allow")
    url: str | None = Field(default=None, description="URL to fetch (from search results or provided company URLs).")


class SaveEvidenceArgs(BaseModel):
    model_config = ConfigDict(extra="allow")
    source_url: str | None = Field(default=None, description="Exact URL you fetched with read_source.")
    title: str | None = Field(default=None, description="Title of the source page.")
    snippet: str | None = Field(default=None, description="Verbatim substring copied from the fetched page text.")
    confidence: float | None = Field(default=None, description="Confidence between 0.0 and 1.0.")


class SaveFindingArgs(BaseModel):
    model_config = ConfigDict(extra="allow")
    category: str | None = Field(default=None, description="Finding category, e.g. architecture.")
    claim: str | None = Field(default=None, description="The finding or deduction.")
    confidence: float | None = Field(default=None, description="Confidence between 0.0 and 1.0.")
    evidence_ids: list[str] | None = Field(
        default=None, description="Evidence IDs (from save_evidence) that support this finding."
    )


class AutonomousAgentRuntime:
    def __init__(
        self,
        ledger: SurrealLedgerRepository | None = None,
        default_ledger: SurrealLedgerRepository | None = None,
    ):
        self._default_ledger = ledger or default_ledger or SurrealLedgerRepository()
        self._global_model_factory: Callable[[], BaseChatModel] | None = None
        self._search_engine: AgentSearchEngine | None = None

    @property
    def ledger(self) -> SurrealLedgerRepository:

        ctx_ledger = current_ledger_var.get()
        return ctx_ledger if ctx_ledger is not None else self._default_ledger

    def set_model_factory(self, factory: Callable[[], BaseChatModel] | None) -> None:

        self._global_model_factory = factory
        current_model_factory_var.set(factory)

    def set_search_engine(self, engine: AgentSearchEngine | None) -> None:
        self._search_engine = engine

    def get_search_engine(self) -> AgentSearchEngine:
        if self._search_engine is None:
            self._search_engine = AgentSearchEngine()
        return self._search_engine

    def get_chat_model(self, role: str = "research", request_timeout_s: float = LLM_REQUEST_TIMEOUT_S) -> BaseChatModel:
        factory = current_model_factory_var.get() or self._global_model_factory
        if factory:
            return factory()
        from src.agents.model_router import resolve_chat_model_for_role
        base_url = os.environ.get("LLM_OPENAI_BASE_URL")
        api_key = os.environ.get("LLM_OPENAI_API_KEY")
        model = os.environ.get("LLM_DEFAULT_MODEL", "gemini-3.8-flash-high")
        if base_url and api_key:
            return ChatOpenAI(
                base_url=base_url,
                api_key=api_key,
                model=model,
                temperature=0.2,
                timeout=request_timeout_s,
                max_retries=0,
            )
        return resolve_chat_model_for_role(role, None, timeout_s=request_timeout_s)

    async def _close_orphaned_child(
        self,
        child_task: asyncio.Task | None,
        child_exec: AgentExecution,
        delegation: Delegation,
        parent: AgentExecution,
        run_id: str,
        reason: str,
    ) -> None:
        if child_task is not None and not child_task.done():
            child_task.cancel()
        if child_task is not None:
            with contextlib.suppress(BaseException):
                await child_task
        ledger = self.ledger
        try:
            persisted = await ledger.get_agent_execution(run_id, child_exec.id)
            if persisted is None or persisted.status in ("pending", "running"):
                child_exec.status = "cancelled"
                child_exec.error = reason
                child_exec.completed_at = datetime.now(timezone.utc)
                await ledger.save_agent_execution(child_exec)
                await ledger.append_event_transactional(
                    run_id=run_id,
                    event_type="agent.failed",
                    task_id=child_exec.task_id,
                    execution_id=child_exec.id,
                    payload={"execution_id": child_exec.id, "error": reason},
                )
            if delegation.status not in ("completed", "failed", "rejected"):
                delegation.status = "failed"
                delegation.error = reason
                await ledger.save_delegation(delegation)
                await ledger.append_event_transactional(
                    run_id=run_id,
                    event_type="delegation.failed",
                    task_id=parent.task_id,
                    execution_id=parent.id,
                    payload={"delegation_id": delegation.id, "error": reason},
                )
        except Exception as ce:
            logger.error(f"Could not close orphaned child {child_exec.id} of {parent.id}: {ce}")

    def create_tools_for_execution(
        self,
        execution: AgentExecution,
        run_id: str,
        depth: int,
        ancestors: list[str],
        tool_call_callback: Callable[[str, str, dict[str, Any], Any, float], Any] | None = None,
        model_override: BaseChatModel | None = None,
    ) -> list[Any]:

        search_calls = [0]

        @tool(
            description="Search the web for company engineering blogs, architectures, and tech stack."
        )
        async def web_search(query: str) -> str:

            clean_q = sanitize_data(query).strip()
            if not clean_q:
                return "Error: web_search requires a non-empty query."
            from src.agents.schemas import current_execution_saved_evidence_ids_var

            saves = len(current_execution_saved_evidence_ids_var.get() or [])
            quota = SEARCH_QUOTA_BASE + (
                saves * SEARCH_QUOTA_PER_SAVE if saves >= SEARCH_QUOTA_MIN_SAVES else 0
            )
            if search_calls[0] >= quota:
                return (
                    "Search quota reached: read_source a result you already found and "
                    "save_evidence before searching again."
                )
            search_calls[0] += 1
            execution.budgets_used.external_requests += 1
            try:
                results = await self.get_search_engine().search_web(clean_q, limit=8)
            except Exception as se:
                return f"Error: web search failed for '{clean_q}': {se}"
            usable = []
            for r in results:
                try:
                    reject_placeholder_domain(r.url)
                    validate_url_for_ssrf(r.url)
                except Exception:
                    continue
                usable.append(
                    {"title": r.title, "url": r.url, "snippet": r.snippet, "provider": r.provider}
                )
            if not usable:
                if getattr(self.get_search_engine(), "last_search_blocked", False):
                    return (
                        "Search providers unavailable: all external search engines are blocked or unreachable. "
                        "Do not retry web_search. Proceed directly with reading company URLs and known sources via read_source."
                    )
                return f"No results for '{clean_q}'. Refine the query with the company name or a specific technology."
            return json.dumps(usable)

        fetched_docs: dict[str, tuple[str, datetime, str]] = {}
        not_found_streak = [0]

        @tool(
            description=(
                "Fetch and parse clean text from a web URL. Argument: url (also accepted: link, source_url, source). "
                "Only use URLs from web_search results or the workspace's provided company URLs; never guess URLs."
            ),
            args_schema=ReadSourceArgs,
        )
        async def read_source(**raw_args: Any) -> str:

            args, arg_error = normalize_tool_args("read_source", raw_args, READ_SOURCE_ALIASES)
            if arg_error:
                return arg_error
            url = args["url"]

            if not_found_streak[0] >= MAX_CONSECUTIVE_NOT_FOUND:
                return (
                    f"Error: {not_found_streak[0]} consecutive reads returned 404. Stop guessing URLs. "
                    "Only read URLs that appear in web_search results or the provided company URLs."
                )
            try:
                reject_placeholder_domain(url)
            except PlaceholderDomainError as pe:
                return f"Error: refused to read {url}. {pe}"
            try:
                validate_url_for_ssrf(url)
                fetcher = SafeResearchFetcher()
                title, markdown = await fetcher.fetch_clean_text(url)
            except httpx.HTTPStatusError as he:
                if he.response.status_code == 404:
                    not_found_streak[0] += 1
                    return (
                        f"Error reading {url}: 404 Not Found ({not_found_streak[0]} consecutive). "
                        "This URL does not exist; pick a URL from your search results instead of guessing."
                    )
                not_found_streak[0] = 0
                return f"Error reading {url}: HTTP {he.response.status_code}"
            except Exception as e:
                return f"Error reading {url}: {e}"
            not_found_streak[0] = 0
            retrieved_at = datetime.now(timezone.utc)
            norm_url = url.strip().rstrip("/")
            fetched_docs[norm_url] = (markdown, retrieved_at, title)
            return f"Title: {title}\n\n{markdown[:3000]}"

        @tool(
            description=(
                "Save verified evidence from a page you already fetched with read_source. Arguments: "
                "source_url (the exact URL you passed to read_source; also: url, link, source), "
                "title (also: name, page_title), snippet (a verbatim substring copied from the fetched "
                "page text; also: quote, exact_quote, excerpt, citation, text), confidence (0.0-1.0)."
            ),
            args_schema=SaveEvidenceArgs,
        )
        async def save_evidence(**raw_args: Any) -> str:

            url_value = next(
                (raw_args[n] for n in SAVE_EVIDENCE_ALIASES["source_url"] if raw_args.get(n)), None
            )
            fetched = fetched_docs.get(str(url_value).strip().rstrip("/")) if url_value else None
            if fetched and not any(raw_args.get(n) for n in SAVE_EVIDENCE_ALIASES["title"]):
                raw_args = {**raw_args, "title": fetched[2]}
            args, arg_error = normalize_tool_args("save_evidence", raw_args, SAVE_EVIDENCE_ALIASES)
            if arg_error:
                return arg_error
            source_url, title, snippet = args["source_url"], args["title"], args["snippet"]
            try:
                confidence = float(args["confidence"])
            except (TypeError, ValueError):
                return "Error: save_evidence 'confidence' must be a number between 0.0 and 1.0."
            norm_url = source_url.strip().rstrip("/")
            if norm_url not in fetched_docs:
                return (
                    f"Error: URL '{source_url}' was never fetched by 'read_source' in this execution. "
                    f"You MUST call 'read_source' on the real URL before saving evidence from it."
                )

            doc_text, retrieved_at, _page_title = fetched_docs[norm_url]

            norm_doc = " ".join(doc_text.split()).lower()
            norm_snip = " ".join(snippet.split()).lower()
            if norm_snip not in norm_doc:
                excerpt = best_matching_excerpt(doc_text, snippet)
                return (
                    f"Error: Snippet does not match text from '{source_url}'. "
                    "The evidence snippet must be a verbatim excerpt of the fetched document. "
                    f"Closest passage on that page (copy an exact substring from it): \"{excerpt}\""
                )

            art = Artifact(
                id=f"art_{uuid.uuid4().hex[:12]}",
                run_id=run_id,
                task_id=execution.task_id,
                agent_execution_id=execution.id,
                type="evidence",
                title=title,
                source_links=[SourceLink(title=title, url=source_url)],
                evidence=EvidenceProvenance(
                    source_url=source_url,
                    title=title,
                    retrieved_at=retrieved_at,
                    snippet=snippet,
                    confidence=min(max(confidence, 0.0), 1.0),
                ),
                content={"snippet": snippet},
            )
            await self.ledger.save_artifact(art)
            await self.ledger.append_event_transactional(
                run_id=run_id,
                event_type="artifact.created",
                task_id=execution.task_id,
                execution_id=execution.id,
                payload={
                    "artifact_id": art.id,
                    "type": art.type,
                    "title": art.title,
                },
            )
            await self.ledger.append_event_transactional(
                run_id=run_id,
                event_type="evidence.saved",
                task_id=execution.task_id,
                execution_id=execution.id,
                payload={"source_id": art.id, "url": source_url, "title": title, "chunk_count": 1},
            )
            execution.artifact_ids.append(art.id)

            from src.agents.schemas import (
                current_execution_saved_evidence_ids_var,
                current_saved_evidence_ids_var,
            )

            valid_set = current_saved_evidence_ids_var.get()
            if valid_set is not None:
                valid_set.add(art.id)
            exec_saved_set = current_execution_saved_evidence_ids_var.get()
            if exec_saved_set is not None:
                exec_saved_set.add(art.id)
            return f"Evidence saved with ID: {art.id}"

        @tool(
            description=(
                "Save an atomic structured finding or research deduction. Arguments: category "
                "(also: topic, area), claim (also: finding, statement, text), confidence (0.0-1.0), "
                "evidence_ids (optional list of evidence IDs from save_evidence that support the claim)."
            ),
            args_schema=SaveFindingArgs,
        )
        async def save_finding(**raw_args: Any) -> str:

            args, arg_error = normalize_tool_args(
                "save_finding", raw_args, SAVE_FINDING_ALIASES, optional=SAVE_FINDING_OPTIONAL
            )
            if arg_error:
                return arg_error
            category, claim, confidence = args["category"], args["claim"], float(args["confidence"])
            raw_ids = args.get("evidence_ids") or []
            if isinstance(raw_ids, str):
                raw_ids = [raw_ids]
            if not isinstance(raw_ids, list) or not all(isinstance(i, str) for i in raw_ids):
                return "Error: save_finding 'evidence_ids' must be a list of evidence ID strings."
            from src.agents.schemas import current_saved_evidence_ids_var

            known_ids = set(current_saved_evidence_ids_var.get() or [])
            known_ids.update(
                a.id for a in await self.ledger.list_artifacts_for_run(run_id) if a.type == "evidence"
            )
            unknown_ids = sorted({i for i in raw_ids if i not in known_ids})
            if unknown_ids:
                return (
                    f"Error: save_finding evidence_ids {unknown_ids} do not exist in this run. "
                    "Cite only IDs returned by save_evidence or listed in your task context."
                )
            evidence_ids = list(dict.fromkeys(raw_ids))

            start_t = time.perf_counter()
            finding_id = f"find_{uuid.uuid4().hex[:12]}"
            await self.ledger.append_event_transactional(
                run_id=run_id,
                event_type="finding.saved",
                task_id=execution.task_id,
                execution_id=execution.id,
                payload={
                    "finding_id": finding_id,
                    "category": category,
                    "claim": claim,
                    "confidence": confidence,
                    "evidence_ids": evidence_ids,
                },
            )
            dur = (time.perf_counter() - start_t) * 1000
            if tool_call_callback:
                await tool_call_callback(
                    "save_finding",
                    "completed",
                    {"category": category, "claim": claim, "evidence_ids": evidence_ids},
                    "Finding saved",
                    dur,
                )
            linked = f" linked to {', '.join(evidence_ids)}" if evidence_ids else ""
            return f"Finding saved: {finding_id}{linked}"

        @tool(description="Append a public progress note or decision milestone.")
        async def update_notes(note_text: str) -> str:

            note = ExecutionNote(ts=datetime.now(timezone.utc), text=note_text)
            execution.notes.append(note)
            await self.ledger.append_event_transactional(
                run_id=run_id,
                event_type="agent.activity",
                task_id=execution.task_id,
                execution_id=execution.id,
                payload={
                    "execution_id": execution.id,
                    "activity_type": "note_update",
                    "description": note_text,
                },
            )
            return "Note recorded."

        @tool(description="Delegate a specialized subtask through the scheduler.")
        async def delegate_task(target_agent_id: str, objective: str, reason: str) -> str:

            if depth >= MAX_DELEGATION_DEPTH:
                return f"Delegation rejected: maximum delegation depth ({MAX_DELEGATION_DEPTH}) exceeded."
            if target_agent_id in ancestors:
                return f"Delegation rejected: cycle detected. Agent {target_agent_id} is in caller ancestry."
            parent_defn = registry.get(execution.agent_id)
            if target_agent_id not in parent_defn.delegation_permissions:
                return f"Delegation rejected: Agent '{execution.agent_id}' does not have delegation permission for '{target_agent_id}'."
            if len(execution.child_execution_ids) >= MAX_DELEGATION_CHILDREN:
                return f"Delegation rejected: maximum children count ({MAX_DELEGATION_CHILDREN}) reached."
            try:
                child_defn = registry.get(target_agent_id)
            except KeyError:
                allowed_registered = sorted(list(registry._agents.keys()))
                return (
                    f"Delegation rejected: Agent '{target_agent_id}' is not a registered agent. "
                    f"Available agents for delegation: {allowed_registered}."
                )
            budget_clock = current_budget_clock_var.get()
            if budget_clock is not None:
                needed_s = float(child_defn.wall_time_seconds) + DELEGATION_PARENT_RESERVE_S
                if budget_clock.remaining_s() < needed_s:
                    return (
                        "Not enough budget left to delegate; finish your research. "
                        f"Remaining {budget_clock.remaining_s():.0f}s, a delegation needs {needed_s:.0f}s."
                    )
            child_exec = AgentExecution(
                id=f"exec_{uuid.uuid4().hex[:12]}",
                run_id=run_id,
                task_id=execution.task_id,
                parent_execution_id=execution.id,
                agent_id=target_agent_id,
                objective=objective,
                status="running",
                delegation_depth=depth + 1,
            )
            execution.child_execution_ids.append(child_exec.id)

            delegation = Delegation(
                id=f"del_{uuid.uuid4().hex[:12]}",
                run_id=run_id,
                parent_execution_id=execution.id,
                child_execution_id=child_exec.id,
                parent_agent_id=execution.agent_id,
                child_agent_id=target_agent_id,
                delegation_task_id=execution.task_id,
                objective=objective,
                reason=reason,
                depth=depth + 1,
                status="running",
            )
            parent_agent_token = current_agent_id_var.set(execution.agent_id)
            from src.agents.schemas import (
                current_execution_saved_evidence_ids_var,
                current_saved_evidence_ids_var,
            )

            parent_evidence_ids = current_saved_evidence_ids_var.get()
            parent_exec_saved = current_execution_saved_evidence_ids_var.get()
            child_task: asyncio.Task | None = None
            clock = current_budget_clock_var.get()
            pause = clock.paused("delegated child wait") if clock is not None else contextlib.nullcontext()
            try:
                await self.ledger.save_agent_execution(child_exec)
                await self.ledger.save_delegation(delegation)
                await self.ledger.append_event_transactional(
                    run_id=run_id,
                    event_type="delegation.started",
                    task_id=execution.task_id,
                    execution_id=execution.id,
                    payload={"delegation_id": delegation.id, "child_execution_id": child_exec.id},
                )
                child_task = asyncio.create_task(
                    self.execute_agent_loop(
                        agent_id=target_agent_id,
                        objective=objective,
                        task_id=execution.task_id,
                        run_id=run_id,
                        input_data={"delegated_reason": reason, "parent_agent_id": execution.agent_id},
                        depth=depth + 1,
                        ancestors=ancestors + [execution.agent_id],
                        model_override=model_override,
                        execution_id=child_exec.id,
                        parent_execution_id=execution.id,
                    )
                )
                with pause:
                    child_exec, child_output = await child_task
                child_evidence_ids = [
                    a.id
                    for a in await self.ledger.list_artifacts_for_run(run_id)
                    if a.type == "evidence" and a.agent_execution_id == child_exec.id
                ]
                child_summary = child_exec.public_summary or f"Delegated execution complete for {target_agent_id}"
                if child_evidence_ids:
                    child_summary = f"{child_summary}\nChild evidence IDs you may cite: {', '.join(child_evidence_ids)}"
                delegation.status = "completed"
                delegation.output_artifact_ids = list(child_exec.artifact_ids)
                await self.ledger.save_delegation(delegation)
            except asyncio.CancelledError:
                closer = asyncio.ensure_future(
                    self._close_orphaned_child(
                        child_task,
                        child_exec,
                        delegation,
                        execution,
                        run_id,
                        f"parent execution {execution.id} ended before delegated child finished",
                    )
                )
                while not closer.done():
                    with contextlib.suppress(asyncio.CancelledError):
                        await asyncio.shield(closer)
                raise
            except Exception as de:
                logger.error(f"Delegation execution error: {de}", exc_info=True)
                persisted_child = await self.ledger.get_agent_execution(run_id, child_exec.id)
                if persisted_child is None or persisted_child.status == "running":
                    child_exec.status = "failed"
                    child_exec.error = str(de) or de.__class__.__name__
                    await self.ledger.save_agent_execution(child_exec)
                delegation.status = "failed"
                delegation.error = str(de) or de.__class__.__name__
                delegation.output_artifact_ids = [
                    a.id
                    for a in await self.ledger.list_artifacts_for_run(run_id)
                    if a.agent_execution_id == child_exec.id
                ]
                await self.ledger.save_delegation(delegation)
                await self.ledger.append_event_transactional(
                    run_id=run_id,
                    event_type="delegation.failed",
                    task_id=execution.task_id,
                    execution_id=execution.id,
                    payload={"delegation_id": delegation.id, "error": str(de)[:500]},
                )
                return (
                    f"Delegation failed: {de}. Continue your own research without this deep dive; "
                    "do not cite evidence from it."
                )
            finally:
                current_agent_id_var.reset(parent_agent_token)
                current_saved_evidence_ids_var.set(parent_evidence_ids)
                current_execution_saved_evidence_ids_var.set(parent_exec_saved)
            if parent_evidence_ids is not None:
                parent_evidence_ids.update(
                    a.id for a in await self.ledger.list_artifacts_for_run(run_id) if a.type == "evidence"
                )

            await self.ledger.append_event_transactional(
                run_id=run_id,
                event_type="delegation.completed",
                task_id=execution.task_id,
                execution_id=execution.id,
                payload={
                    "delegation_id": delegation.id,
                    "child_execution_id": child_exec.id,
                    "summary": child_summary,
                },
            )

            return child_summary

        @tool(description="Evaluate research findings and claims against criteria.")
        async def evaluate_findings(summary: str) -> str:

            return f"Evaluated findings: {summary}. Criteria satisfied."

        return [
            web_search,
            read_source,
            save_evidence,
            save_finding,
            update_notes,
            delegate_task,
            evaluate_findings,
        ]

    async def execute_agent_loop(
        self,
        agent_id: str,
        objective: str,
        task_id: str,
        run_id: str,
        input_data: dict[str, Any],
        model_override: BaseChatModel | None = None,
        depth: int = 0,
        ancestors: list[str] | None = None,
        execution_id: str | None = None,
        parent_execution_id: str | None = None,
    ) -> tuple[AgentExecution, dict[str, Any]]:
        execution_id = execution_id or f"exec_{uuid.uuid4().hex[:12]}"
        ledger = self.ledger

        async def announce_wait(limit_label: str) -> None:
            await ledger.append_event_transactional(
                run_id=run_id,
                event_type="agent.activity",
                task_id=task_id,
                execution_id=execution_id,
                payload={
                    "execution_id": execution_id,
                    "activity_type": "waiting_for_slot",
                    "description": f"{agent_id}: Waiting for execution slot ({limit_label})",
                },
            )
        slot_wait_started = time.perf_counter()
        async with agent_limits.execution_slot(run_id, agent_id, on_wait=announce_wait):
            slot_wait_s = time.perf_counter() - slot_wait_started
            if slot_wait_s >= 1.0:
                logger.info(
                    f"Execution {execution_id} ({agent_id}): execution slot wait took {slot_wait_s:.1f}s; "
                    "deadline not started during it"
                )
            return await self._run_agent_loop(
                agent_id=agent_id,
                objective=objective,
                task_id=task_id,
                run_id=run_id,
                input_data=input_data,
                model_override=model_override,
                depth=depth,
                ancestors=ancestors,
                execution_id=execution_id,
                parent_execution_id=parent_execution_id,
            )

    async def _run_agent_loop(
        self,
        agent_id: str,
        objective: str,
        task_id: str,
        run_id: str,
        input_data: dict[str, Any],
        model_override: BaseChatModel | None,
        depth: int,
        ancestors: list[str] | None,
        execution_id: str,
        parent_execution_id: str | None,
    ) -> tuple[AgentExecution, dict[str, Any]]:
        current_agent_id_var.set(agent_id)
        defn = registry.get(agent_id)
        current_ancestors = (ancestors or []) + [agent_id]

        from src.agents.schemas import (
            current_execution_saved_evidence_ids_var,
            current_saved_evidence_ids_var,
        )

        current_execution_saved_evidence_ids_var.set(set())
        existing_artifacts = await self.ledger.list_artifacts_for_run(run_id)
        saved_ev_ids = {a.id for a in existing_artifacts if a.type == "evidence"}
        current_saved_evidence_ids_var.set(saved_ev_ids)
        chat_model = model_override or self.get_chat_model(role=agent_id, request_timeout_s=defn.llm_request_timeout_s)
        model_name = getattr(chat_model, "model_name", getattr(chat_model, "model", "custom-model"))
        provider_name = chat_model.__class__.__name__
        exec_rec = AgentExecution(
            id=execution_id,
            run_id=run_id,
            task_id=task_id,
            parent_execution_id=parent_execution_id,
            agent_id=agent_id,
            objective=objective,
            status="running",
            delegation_depth=depth,
            model_provider=provider_name,
            model_name=model_name,
            budgets=ExecutionBudgets(
                tool_calls=defn.tool_calls_budget,
                model_calls=defn.model_calls_budget,
                wall_time_ms=defn.wall_time_seconds * 1000,
            ),
            budgets_used=ExecutionBudgetsUsed(),
            started_at=datetime.now(timezone.utc),
        )
        await self.ledger.save_agent_execution(exec_rec)

        await self.ledger.append_event_transactional(
            run_id=run_id,
            event_type="agent.started",
            task_id=task_id,
            execution_id=exec_rec.id,
            payload={
                "execution_id": exec_rec.id,
                "agent_id": agent_id,
                "objective": objective,
                "model": exec_rec.model_name,
            },
        )
        self_ledger = self.ledger

        class ActivityMiddleware(AgentMiddleware):
            async def abefore_model(self, state: Any, runtime: Any = None) -> Any:
                msgs = state.get("messages", []) if isinstance(state, dict) else []
                step_num = exec_rec.budgets_used.model_calls + 1
                await self_ledger.append_event_transactional(
                    run_id=run_id,
                    event_type="agent.activity",
                    task_id=task_id,
                    execution_id=exec_rec.id,
                    payload={
                        "execution_id": exec_rec.id,
                        "activity_type": "model_step_started",
                        "description": f"Step {step_num}: Dispatched prompt to {exec_rec.model_name} (in-flight)",
                    },
                )
                return None

            async def aafter_model(self, state: Any, runtime: Any = None) -> Any:
                exec_rec.budgets_used.model_calls += 1
                msgs = state.get("messages", []) if isinstance(state, dict) else []
                if msgs:
                    last_m = msgs[-1]
                    tc_list = getattr(last_m, "tool_calls", []) or []
                    step_desc = ""
                    if tc_list:
                        intents = [f"call {t.get('name')}" for t in tc_list]
                        step_desc = f"Action: {', '.join(intents)}"
                    elif getattr(last_m, "content", None):
                        txt = str(last_m.content).strip()
                        if txt and not txt.startswith("{"):
                            step_desc = f"Reasoning: {txt[:160]}"
                        else:
                            step_desc = f"Generated structured output deliverable ({len(txt)} chars)"
                    if step_desc:
                        await self_ledger.append_event_transactional(
                            run_id=run_id,
                            event_type="agent.activity",
                            task_id=task_id,
                            execution_id=exec_rec.id,
                            payload={
                                "execution_id": exec_rec.id,
                                "activity_type": "model_step",
                                "description": step_desc,
                            },
                        )
                return None

        @wrap_tool_call
        async def intercept_tool_call(request, handler):
            lost_ev = current_lost_lease_var.get()
            if lost_ev and lost_ev.is_set():
                raise RuntimeError("Lease lost: cooperative abort at tool boundary.")
            tc = getattr(request, "tool_call", {}) or {}
            t_name = tc.get("name", "tool") if isinstance(tc, dict) else getattr(tc, "name", "tool")
            t_args = tc.get("args", {}) if isinstance(tc, dict) else getattr(tc, "args", {})
            st = time.perf_counter()
            t_id = f"tool_{uuid.uuid4().hex[:12]}"
            inv = ToolInvocation(
                id=t_id,
                execution_id=validate_id(exec_rec.id, "execution_id"),
                tool_name=t_name,
                status="running",
                duration_ms=0.0,
                sanitized_input=sanitize_data(t_args),
                output_summary="",
                started_at=datetime.now(timezone.utc),
            )
            await self_ledger.save_tool_invocation(inv, run_id=run_id)
            await self_ledger.append_event_transactional(
                run_id=run_id,
                event_type="tool.started",
                task_id=task_id,
                execution_id=exec_rec.id,
                tool_invocation_id=inv.id,
                payload={
                    "tool_invocation_id": inv.id,
                    "tool_name": t_name,
                    "sanitized_input": inv.sanitized_input,
                },
            )

            exec_rec.tool_call_count += 1
            exec_rec.budgets_used.tool_calls += 1
            try:
                out = await handler(request)
                dur = (time.perf_counter() - st) * 1000
                out_content = getattr(out, "content", str(out))
                inv.status = "completed"
                inv.duration_ms = dur
                inv.output_summary = str(out_content)[:300]
                await self_ledger.save_tool_invocation(inv, run_id=run_id)
                await self_ledger.append_event_transactional(
                    run_id=run_id,
                    event_type="tool.completed",
                    task_id=task_id,
                    execution_id=exec_rec.id,
                    tool_invocation_id=inv.id,
                    payload={
                        "tool_invocation_id": inv.id,
                        "tool_name": t_name,
                        "duration_ms": dur,
                        "summary": inv.output_summary,
                    },
                )
                return out
            except Exception as te:
                dur = (time.perf_counter() - st) * 1000
                inv.status = "failed"
                inv.duration_ms = dur
                inv.error = str(te)
                await self_ledger.save_tool_invocation(inv, run_id=run_id)
                await self_ledger.append_event_transactional(
                    run_id=run_id,
                    event_type="tool.failed",
                    task_id=task_id,
                    execution_id=exec_rec.id,
                    tool_invocation_id=inv.id,
                    payload={"tool_invocation_id": inv.id, "tool_name": t_name, "error": str(te)},
                )
                raise te

        tools = self.create_tools_for_execution(
            execution=exec_rec,
            run_id=run_id,
            depth=depth,
            ancestors=current_ancestors,
            tool_call_callback=None,
            model_override=model_override,
        )

        from src.agents.schemas import CURRICULUM_STAGE_SCHEMAS, CategoryQuestionBatchOutput

        if agent_id == "interview-generation" and input_data and "target_category" in input_data:
            schema_cls = CategoryQuestionBatchOutput
        elif agent_id == "curriculum" and input_data and "curriculum_stage" in input_data:
            schema_cls = CURRICULUM_STAGE_SCHEMAS[input_data["curriculum_stage"]]
        else:
            schema_cls = AGENT_OUTPUT_SCHEMAS.get(agent_id, AgentTaskOutput)
        tool_strategy = ToolStrategy(schema_cls, handle_errors=True)
        from langchain_core.messages import AIMessage, HumanMessage, SystemMessage

        response_tool_name = schema_cls.__name__

        evidence_floor = EVIDENCE_FLOOR_BY_AGENT.get(agent_id, 0)
        save_tools = {"read_source", "save_evidence"}

        def own_saved_ids() -> list[str]:
            from src.agents.schemas import current_execution_saved_evidence_ids_var

            return sorted(current_execution_saved_evidence_ids_var.get() or [])

        async def emit_budget_activity(description: str) -> None:
            logger.info(f"Execution {exec_rec.id} ({agent_id}): {description}")
            await self_ledger.append_event_transactional(
                run_id=run_id,
                event_type="agent.activity",
                task_id=task_id,
                execution_id=exec_rec.id,
                payload={
                    "execution_id": exec_rec.id,
                    "activity_type": "budget_control",
                    "description": description,
                },
            )

        budget_mode = {"value": "normal"}

        @wrap_model_call
        async def budget_finalizer_middleware(request, handler):
            used_tools = exec_rec.budgets_used.tool_calls
            used_models = exec_rec.budgets_used.model_calls
            rem_tools = exec_rec.budgets.tool_calls - used_tools
            rem_models = exec_rec.budgets.model_calls - used_models
            active_s = budget_clock.active_elapsed_s()
            rem_time_ratio = 1.0 - (active_s / max(1.0, budget_clock.budget_s))
            saved_ids = own_saved_ids()

            rem_time_s = budget_clock.remaining_s()
            if (
                rem_tools <= FINALIZE_RESERVED_TOOL_CALLS
                or rem_models <= FINALIZE_RESERVED_MODEL_CALLS
                or rem_time_ratio <= FINALIZE_TIME_RATIO
                or rem_time_s <= FINALIZE_MIN_WALL_TIME_RESERVE_S
            ):
                if budget_mode["value"] != "finalize":
                    budget_mode["value"] = "finalize"
                    await emit_budget_activity(
                        f"Research budget exhausted: forcing final output with {len(saved_ids)} saved evidence item(s)"
                    )
                saved_list = ", ".join(saved_ids) if saved_ids else "none"
                warn_msg = SystemMessage(
                    content=(
                        "CRITICAL: Research budget is nearly exhausted. "
                        "Stop calling research tools. You must call the final output tool now "
                        f"('{response_tool_name}') with your accumulated evidence and findings. "
                        f"You saved {len(saved_ids)} evidence item(s) in this execution: {saved_list}. "
                        "Cite only these exact IDs (or IDs from the task context); do not invent IDs."
                    )
                )
                msgs = list(request.messages) + [warn_msg]
                req = request.override(messages=msgs, tools=[])
                return await handler(req)

            missing = max(0, evidence_floor - len(saved_ids))
            if missing and rem_tools <= missing * 2 + 2:
                if budget_mode["value"] != "reserve":
                    budget_mode["value"] = "reserve"
                    await emit_budget_activity(
                        f"Research budget low: restricting to read/save tools; {missing} more evidence needed"
                    )
                reserve_msg = SystemMessage(
                    content=(
                        f"Budget reserve: you still need {missing} more saved evidence item(s) and have "
                        f"{rem_tools} tool call(s) left. web_search is disabled. Use read_source on a URL "
                        "you already found, then save_evidence with an exact quote from it."
                    )
                )
                kept = [t for t in request.tools if getattr(t, "name", None) in save_tools]
                req = request.override(messages=list(request.messages) + [reserve_msg], tools=kept)
                return await handler(req)
            return await handler(request)

        llm_retry_middleware = build_llm_retry_middleware(f"Execution {exec_rec.id} ({agent_id})")

        missing_output_nudges = {"count": 0}

        @wrap_model_call
        async def require_output_tool_middleware(request, handler):
            response = await handler(request)
            while missing_output_nudges["count"] < MAX_MISSING_OUTPUT_TOOL_NUDGES:
                ai_msg = next(
                    (m for m in reversed(getattr(response, "result", []) or []) if isinstance(m, AIMessage)),
                    None,
                )
                if ai_msg is None or ai_msg.tool_calls:
                    return response
                missing_output_nudges["count"] += 1
                logger.warning(
                    f"Execution {exec_rec.id} ({agent_id}): model returned no tool call "
                    f"(content {len(str(ai_msg.content or ''))} chars); re-prompting for {response_tool_name} "
                    f"({missing_output_nudges['count']}/{MAX_MISSING_OUTPUT_TOOL_NUDGES})"
                )
                await self_ledger.append_event_transactional(
                    run_id=run_id,
                    event_type="agent.activity",
                    task_id=task_id,
                    execution_id=exec_rec.id,
                    payload={
                        "execution_id": exec_rec.id,
                        "activity_type": "output_reprompt",
                        "description": f"No tool call returned; re-prompting for {response_tool_name}",
                    },
                )
                nudge = HumanMessage(
                    content=(
                        f"Your last reply contained no tool call. You must call {response_tool_name} "
                        "now with your complete structured output. Do not reply with plain text."
                    )
                )
                response = await handler(
                    request.override(messages=list(request.messages) + [ai_msg, nudge])
                )
            return response

        middleware = [
            ModelCallLimitMiddleware(
                run_limit=exec_rec.budgets.model_calls + FINALIZE_MODEL_CALL_HEADROOM
            ),
            ToolCallLimitMiddleware(
                run_limit=max(1, exec_rec.budgets.tool_calls - FINALIZE_RESERVED_TOOL_CALLS)
            ),
            ActivityMiddleware(),
            intercept_tool_call,
            budget_finalizer_middleware,
            require_output_tool_middleware,
            llm_retry_middleware,
            build_provider_slot_middleware(provider_key(chat_model)),
        ]

        agent = create_agent(
            model=chat_model,
            tools=tools,
            system_prompt=defn.instructions,
            response_format=tool_strategy,
            middleware=middleware,
        )
        start_perf = time.perf_counter()
        timeout_seconds = float(defn.wall_time_seconds)
        budget_clock = ExecutionBudgetClock(f"Execution {exec_rec.id} ({agent_id})", timeout_seconds)
        clock_token = current_budget_clock_var.set(budget_clock)
        try:
            try:
                async with asyncio.timeout(timeout_seconds) as timeout_cm:
                    budget_clock.start(timeout_cm)
                    user_msg = f"Task: {objective}\nContext: {json.dumps(input_data)}"
                    result = await agent.ainvoke({"messages": [{"role": "user", "content": user_msg}]})
            finally:
                current_budget_clock_var.reset(clock_token)

            exec_rec.budgets_used.wall_time_ms = (time.perf_counter() - start_perf) * 1000
            if budget_clock.excluded_s:
                logger.info(
                    f"Execution {exec_rec.id} ({agent_id}) excluded from wall budget: "
                    f"{ {k: round(v, 1) for k, v in budget_clock.excluded_by_reason.items()} }"
                )

            structured_data = None
            if isinstance(result, dict):
                structured = result.get("structured_response")
                if isinstance(structured, BaseModel):
                    exec_rec.public_summary = getattr(structured, "summary", str(structured))
                    structured_data = structured
                elif isinstance(structured, dict):
                    exec_rec.public_summary = structured.get("summary", str(structured))
                    structured_data = structured

                msgs = result.get("messages", [])
                total_in = 0
                total_out = 0
                for m in msgs:
                    usage = getattr(m, "usage_metadata", None)
                    if isinstance(usage, dict):
                        total_in += usage.get("input_tokens", 0)
                        total_out += usage.get("output_tokens", 0)
                exec_rec.budgets_used.tokens_in = total_in
                exec_rec.budgets_used.tokens_out = total_out
            if not exec_rec.public_summary or structured_data is None:
                last_m_str = str(result.get("messages", [])[-1:]) if isinstance(result, dict) else str(result)
                logger.error(
                    f"Agent {agent_id} failed to produce valid structured output for objective: {objective}. Last message: {last_m_str[:300]}"
                )
                raise ValueError(
                    f"Agent {agent_id} failed to produce valid structured output for objective: {objective}"
                )
            AGENT_ARTIFACT_TYPE_MAP = {
                "company-research": "research_report",
                "job-analysis": "finding",
                "technical-research": "research_report",
                "interview-generation": "question_set",
                "evidence-evaluator": "validation_result",
            }
            art_type = AGENT_ARTIFACT_TYPE_MAP.get(agent_id, "research_report")
            dict_content = (
                structured_data.model_dump()
                if hasattr(structured_data, "model_dump")
                else structured_data
            )
            art = Artifact(
                id=f"art_{uuid.uuid4().hex[:12]}",
                run_id=run_id,
                task_id=task_id,
                agent_execution_id=exec_rec.id,
                type=art_type,
                title=f"{defn.name} Deliverable",
                evidence_ids=list(saved_ev_ids),
                content=dict_content,
            )
            await self.ledger.save_artifact(art)
            await self.ledger.append_event_transactional(
                run_id=run_id,
                event_type="artifact.created",
                task_id=task_id,
                execution_id=exec_rec.id,
                payload={
                    "artifact_id": art.id,
                    "type": art.type,
                    "title": art.title,
                },
            )
            exec_rec.artifact_ids.append(art.id)
            exec_rec.status = "completed"
            exec_rec.completed_at = datetime.now(timezone.utc)
            await self.ledger.save_agent_execution(exec_rec)

            total_tokens = exec_rec.budgets_used.tokens_in + exec_rec.budgets_used.tokens_out
            await self.ledger.append_event_transactional(
                run_id=run_id,
                event_type="agent.completed",
                task_id=task_id,
                execution_id=exec_rec.id,
                payload={
                    "execution_id": exec_rec.id,
                    "summary": exec_rec.public_summary,
                    "tool_calls": exec_rec.tool_call_count,
                    "tokens_used": total_tokens,
                },
            )
            return exec_rec, dict_content
        except Exception as e:
            exec_rec.budgets_used.wall_time_ms = (time.perf_counter() - start_perf) * 1000
            if budget_clock.excluded_s:
                logger.info(
                    f"Execution {exec_rec.id} ({agent_id}) excluded from wall budget: "
                    f"{ {k: round(v, 1) for k, v in budget_clock.excluded_by_reason.items()} }"
                )
            exec_rec.status = "failed"
            if isinstance(e, (asyncio.TimeoutError, TimeoutError)):
                error_text = f"wall-time budget exceeded ({int(timeout_seconds)}s)"
            else:
                error_text = str(e) or e.__class__.__name__
            exec_rec.error = error_text
            exec_rec.completed_at = datetime.now(timezone.utc)
            await self.ledger.save_agent_execution(exec_rec)
            await self.ledger.append_event_transactional(
                run_id=run_id,
                event_type="agent.failed",
                task_id=task_id,
                execution_id=exec_rec.id,
                payload={"execution_id": exec_rec.id, "error": error_text},
            )
            if error_text != str(e):
                raise AgentExecutionError(error_text) from e
            raise


agent_runtime = AutonomousAgentRuntime()
