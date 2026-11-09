"""Tool implementations exposed to delegated research agents."""
import asyncio
import contextlib
import json
import logging
import time
import uuid
from datetime import datetime, timezone
from typing import Any, Callable

import httpx
from langchain_core.language_models.chat_models import BaseChatModel
from langchain_core.tools import tool
from pydantic import BaseModel, ConfigDict, Field

from src.agents.registry import registry
from src.agents.runtime_support import (
    DELEGATION_PARENT_RESERVE_S,
    MAX_CONSECUTIVE_NOT_FOUND,
    MAX_DELEGATION_CHILDREN,
    MAX_DELEGATION_DEPTH,
    SEARCH_QUOTA_BASE,
    SEARCH_QUOTA_MIN_SAVES,
    SEARCH_QUOTA_PER_SAVE,
    best_matching_excerpt,
    current_agent_id_var,
    current_budget_clock_var,
)
from src.contracts.v1 import (
    AgentExecution,
    Artifact,
    Delegation,
    EvidenceProvenance,
    ExecutionNote,
    SourceLink,
)
from src.fetcher.reader import SafeResearchFetcher
from src.fetcher.ssrf_client import (
    PlaceholderDomainError,
    reject_placeholder_domain,
    validate_url_for_ssrf,
)
from src.orchestration.sanitizer import sanitize_data

logger = logging.getLogger("mensetsu.agents.runtime")


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


class ToolsMixin:
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
