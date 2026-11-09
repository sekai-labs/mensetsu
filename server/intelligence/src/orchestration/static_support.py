"""Shared state, constants, and node support helpers for the static workflow."""
import logging
from dataclasses import dataclass
from datetime import datetime
from typing import Annotated, Any, Awaitable, Callable

from pydantic import BaseModel, Field

from src.contracts.v1 import WorkflowTask
from src.orchestration.ledger import SurrealLedgerRepository

logger = logging.getLogger("mensetsu.orchestration.static_workflow")

CURRICULUM_MIN_UNITS = 6
CURRICULUM_UNIT_BATCH_SIZE = 2
QUESTION_CATEGORIES = (
    "system_design",
    "target_stack",
    "technical_fundamentals",
    "resume_deep_dive",
    "project_deep_dive",
    "debugging",
    "behavioral",
)
QUESTIONS_PER_BATCH = 5
BATCHES_PER_CATEGORY = 3
MIN_QUESTIONS_PER_CATEGORY = QUESTIONS_PER_BATCH * BATCHES_PER_CATEGORY
TOTAL_MIN_QUESTIONS = 100


def merge_dict(a: dict[str, Any], b: dict[str, Any]) -> dict[str, Any]:
    res = dict(a or {})
    for k, v in (b or {}).items():
        if isinstance(v, dict) and isinstance(res.get(k), dict):
            res[k] = {**res[k], **v}
        else:
            res[k] = v
    return res


class StaticWorkflowState(BaseModel):
    run_id: str
    workspace_id: str
    user_id: str = ""
    company_name: str
    role_title: str
    company_urls: list[str] = Field(default_factory=list)
    resume_text: str = ""
    resume_upload_id: str | None = None
    resume_filename: str | None = None
    job_description: str = ""
    jd_url: str | None = None
    seniority: str | None = None
    validation_loop_count: int = 0
    parsed_resume: dict[str, Any] = Field(default_factory=dict)
    research_results: Annotated[dict[str, Any], merge_dict] = Field(default_factory=dict)
    match_analysis: dict[str, Any] = Field(default_factory=dict)
    missing_research: Annotated[dict[str, str], merge_dict] = Field(default_factory=dict)
    questions: list[dict[str, Any]] = Field(default_factory=list)
    curriculum: dict[str, Any] = Field(default_factory=dict)
    upstream_task_ids: list[str] = Field(default_factory=list)


@dataclass(slots=True)
class StaticNodeSupport:
    """Shared collaborators injected into every static workflow node factory."""

    ledger: SurrealLedgerRepository
    db: Any
    check_lease: Callable[[], None]
    set_phase: Callable[[StaticWorkflowState, str, int], Awaitable[None]]
    completed_task_deliverable: Callable[[str, str, str], Awaitable[dict[str, Any] | None]]
    restored_job_description_text: Callable[[str, WorkflowTask], Awaitable[str]]
    fetch_job_description: Callable[[StaticWorkflowState, str], Awaitable[tuple[str, str]]]


def build_support(ledger: SurrealLedgerRepository, db: Any) -> StaticNodeSupport:
    def check_lease():
        from src.agents.runtime import current_lost_lease_var

        lost_ev = current_lost_lease_var.get()
        if lost_ev and lost_ev.is_set():
            raise RuntimeError("Lease lost: cooperative abort at node boundary.")

    async def set_phase(state: StaticWorkflowState, phase: str, percent: int) -> None:
        try:
            await ledger.update_workflow_run(state.run_id, {"current_phase": phase})
        except Exception as re:
            logger.warning(f"Could not update run phase for {state.run_id}: {re}")
        from src.generation.progress import update_workspace_progress

        try:
            await update_workspace_progress(
                workspace_id=state.workspace_id, phase=phase, percent=percent, db=db
            )
        except Exception as pe:
            logger.warning(f"Could not update workspace progress: {pe}")

    async def completed_task_deliverable(
        run_id: str, task_id: str, artifact_type: str
    ) -> dict[str, Any] | None:
        arts = await ledger.list_artifacts_for_run(run_id)
        deliverables = [
            a
            for a in arts
            if a.task_id == task_id
            and a.type == artifact_type
            and a.agent_execution_id
            and isinstance(a.content, dict)
        ]
        if not deliverables:
            return None
        return max(deliverables, key=lambda a: a.created_at).content

    async def restored_job_description_text(run_id: str, normalize_task: WorkflowTask) -> str:
        restored = normalize_task.output_data.get("job_description_text")
        if isinstance(restored, str) and restored.strip():
            return restored
        raise RuntimeError(
            f"Run {run_id}: normalize task is completed but its job description text is not recoverable."
        )

    async def fetch_job_description(state: StaticWorkflowState, task_id: str) -> tuple[str, str]:
        from src.contracts.v1 import Artifact, EvidenceProvenance, SourceLink
        from src.fetcher.reader import SafeReaderFetcher
        from src.fetcher.ssrf_client import reject_placeholder_domain, validate_url_for_ssrf
        from src.orchestration.sanitizer import sanitize_text

        reject_placeholder_domain(state.jd_url)
        validate_url_for_ssrf(state.jd_url)
        doc = await SafeReaderFetcher(timeout_seconds=15.0).fetch_document(state.jd_url)
        text = doc.markdown.strip()
        if doc.access_status != "success" or not text:
            raise ValueError(
                f"Job description URL {state.jd_url} could not be read ({doc.access_status}); paste the job description text instead."
            )
        evidence = Artifact(
            id=f"art_jd_{state.run_id[:8]}",
            run_id=state.run_id,
            task_id=task_id,
            agent_execution_id=f"normalize_{state.run_id[:8]}",
            type="evidence",
            title=doc.title or "Job description",
            source_links=[SourceLink(title=doc.title or "Job description", url=state.jd_url)],
            evidence=EvidenceProvenance(
                source_url=state.jd_url,
                title=doc.title or "Job description",
                retrieved_at=datetime.fromisoformat(doc.retrieved_at),
                snippet=sanitize_text(text[:1500]),
                confidence=1.0,
            ),
            content={"snippet": sanitize_text(text[:1500]), "full_text_chars": len(text)},
        )
        await ledger.save_artifact(evidence)
        await ledger.append_event_transactional(
            run_id=state.run_id,
            event_type="evidence.saved",
            task_id=task_id,
            payload={
                "source_id": evidence.id,
                "url": state.jd_url,
                "title": evidence.title,
                "chunk_count": 1,
            },
        )
        return text, evidence.id

    return StaticNodeSupport(
        ledger=ledger,
        db=db,
        check_lease=check_lease,
        set_phase=set_phase,
        completed_task_deliverable=completed_task_deliverable,
        restored_job_description_text=restored_job_description_text,
        fetch_job_description=fetch_job_description,
    )
