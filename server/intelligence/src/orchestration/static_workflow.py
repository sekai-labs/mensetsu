import asyncio
import logging
import time
from datetime import datetime, timezone
from typing import Annotated, Any
from urllib.parse import urlparse

from langgraph.graph import END, START, StateGraph
from pydantic import BaseModel, Field

from src.agents.runtime import agent_runtime
from src.contracts.v1 import WorkflowTask
from src.db.client import SurrealDirectClient
from src.generation.persistence import persist_record
from src.orchestration.id_validator import validate_id
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
    company_name: str
    role_title: str
    company_urls: list[str] = Field(default_factory=list)
    resume_text: str = ""
    resume_pdf_base64: str | None = None
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


def build_static_workflow(
    checkpointer: Any,
    ledger: SurrealLedgerRepository | None = None,
    db_client: SurrealDirectClient | None = None,
    mode: str = "full",
):
    if mode not in ("full", "normalize_only", "outputs_only"):
        raise ValueError(f"Unknown static workflow mode '{mode}'.")
    if checkpointer is None:
        raise ValueError("Checkpointer is required to compile static workflow.")
    ledger = ledger or SurrealLedgerRepository(db_client=db_client)
    from src.agents.runtime import current_ledger_var
    current_ledger_var.set(ledger)
    db = db_client or ledger.db
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

    async def completed_task_deliverable(run_id: str, task_id: str, artifact_type: str) -> dict[str, Any] | None:
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

    async def normalize_input_node(state: StaticWorkflowState) -> dict[str, Any]:
        check_lease()
        normalize_task_id = f"task_normalize_{state.run_id[:8]}"
        existing = await ledger.get_task(state.run_id, normalize_task_id)
        if existing is not None and existing.status == "completed":
            from src.generation.resume import load_parsed_resume

            stored_cv = existing.output_data.get("parsed_resume")
            restored_cv = (
                state.parsed_resume
                or (stored_cv if isinstance(stored_cv, dict) and stored_cv.get("raw_text") else None)
                or await load_parsed_resume(state.workspace_id, db_client=db)
            )
            if restored_cv is None:
                raise RuntimeError(
                    f"Run {state.run_id}: normalize task is completed but its parsed CV is not recoverable."
                )
            return {
                "parsed_resume": restored_cv,
                "resume_text": state.resume_text.strip() or restored_cv["raw_text"],
                "job_description": state.job_description.strip()
                or await restored_job_description_text(state.run_id, existing),
            }
        await set_phase(state, "analyzing_resume", 5)
        start_t = time.perf_counter()
        task = WorkflowTask(
            id=f"task_normalize_{state.run_id[:8]}",
            run_id=state.run_id,
            task_type="normalization",
            name="Normalize CV and JD inputs",
            assigned_agent="job-analysis",
            status="running",
        )
        first_attempt = await ledger.next_task_attempt(task.run_id, task.id)
        task.attempt = first_attempt
        await ledger.save_task(task)
        await ledger.append_event_transactional(
            run_id=state.run_id,
            event_type="task.started",
            task_id=task.id,
            payload={"task_id": task.id, "attempt": first_attempt, "assigned_agent": task.assigned_agent},
        )

        from src.generation.documents import extract_resume_pdf_text
        from src.generation.resume import parse_resume_activity

        try:
            resume_text = state.resume_text.strip()
            resume_source = "text"
            if not resume_text and state.resume_pdf_base64:
                resume_text = extract_resume_pdf_text(state.resume_pdf_base64, state.resume_filename)
                resume_source = "pdf"
            if not resume_text:
                raise ValueError(
                    "No resume provided: resume_text is empty and no resume PDF was uploaded."
                )

            job_description = state.job_description.strip()
            jd_evidence_id = None
            if not job_description and state.jd_url:
                job_description, jd_evidence_id = await fetch_job_description(state, task.id)

            parsed_cv = await parse_resume_activity(
                workspace_id=state.workspace_id,
                resume_text=resume_text,
                db_client=db,
            )
        except Exception as e:
            task.status = "failed"
            task.error = str(e)
            await ledger.save_task(task)
            await ledger.append_event_transactional(
                run_id=state.run_id,
                event_type="task.failed",
                task_id=task.id,
                payload={"task_id": task.id, "attempt": first_attempt, "error": str(e), "can_retry": False},
            )
            raise

        skills_count = len(parsed_cv.get("extracted_skills", []))
        dur = (time.perf_counter() - start_t) * 1000
        task.status = "completed"
        task.output_data = {
            "skills_count": skills_count,
            "resume_source": resume_source,
            "job_description_source": "url" if jd_evidence_id else "text",
            "jd_evidence_id": jd_evidence_id,
            "parsed_resume": parsed_cv,
            "job_description_text": job_description,
        }
        await ledger.save_task(task)
        await ledger.append_event_transactional(
            run_id=state.run_id,
            event_type="task.completed",
            task_id=task.id,
            payload={
                "task_id": task.id,
                "duration_ms": dur,
                "output_summary": f"CV ({resume_source}) extracted {skills_count} skills.",
            },
        )
        await set_phase(state, "researching_company", 15)
        return {
            "parsed_resume": parsed_cv,
            "resume_text": resume_text,
            "job_description": job_description,
        }

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

    async def company_research_node(state: StaticWorkflowState) -> dict[str, Any]:
        check_lease()
        task_id = f"task_company_{state.run_id[:8]}"
        tasks = await ledger.list_tasks_for_run(state.run_id)
        matching_task = next((t for t in tasks if t.id == task_id), None)
        if matching_task and matching_task.status == "completed":
            comp_existing = state.research_results.get("company")
            if not comp_existing:
                comp_existing = await completed_task_deliverable(state.run_id, task_id, "research_report")
            return {"research_results": {"company": comp_existing or {}}}

        start_t = time.perf_counter()
        task = WorkflowTask(
            id=task_id,
            run_id=state.run_id,
            task_type="research",
            name="Research Target Company",
            assigned_agent="company-research",
            dependencies=[f"task_normalize_{state.run_id[:8]}"],
            status="running",
        )
        first_attempt = await ledger.next_task_attempt(task.run_id, task.id)
        task.attempt = first_attempt
        await ledger.save_task(task)
        await ledger.append_event_transactional(
            run_id=state.run_id,
            event_type="task.started",
            task_id=task.id,
            payload={"task_id": task.id, "attempt": first_attempt, "assigned_agent": task.assigned_agent},
        )

        max_task_attempts = 2
        for attempt in range(first_attempt, first_attempt - 1 + max_task_attempts + 1):
            task.attempt = attempt
            await ledger.save_task(task)
            try:
                prior_artifacts = await ledger.list_artifacts_for_run(state.run_id)
                prior_ev_ids = [a.id for a in prior_artifacts if a.type == "evidence"]
                input_payload = {"company": state.company_name, "urls": state.company_urls}
                if attempt > first_attempt:
                    input_payload["prior_evidence_ids"] = prior_ev_ids
                    input_payload["retry_attempt"] = attempt

                exec_rec, struct_data = await agent_runtime.execute_agent_loop(
                    agent_id="company-research",
                    objective=f"Analyze company {state.company_name} culture and tech stack",
                    task_id=task.id,
                    run_id=state.run_id,
                    input_data=input_payload,
                )
                dur = (time.perf_counter() - start_t) * 1000
                task.status = "completed"
                task.execution_ids.append(exec_rec.id)
                await ledger.save_task(task)
                await ledger.append_event_transactional(
                    run_id=state.run_id,
                    event_type="task.completed",
                    task_id=task.id,
                    payload={
                        "task_id": task.id,
                        "duration_ms": dur,
                        "output_summary": exec_rec.public_summary,
                    },
                )
                return {
                    "research_results": {
                        "company": struct_data
                        if isinstance(struct_data, dict)
                        else (
                            struct_data.model_dump()
                            if hasattr(struct_data, "model_dump")
                            else exec_rec.public_summary
                        )
                    }
                }
            except Exception as e:
                logger.warning(f"Company research task attempt {attempt} failed: {e}")
                if attempt < first_attempt - 1 + max_task_attempts:
                    delay_ms = 1000 * (attempt - first_attempt + 1)
                    await ledger.append_event_transactional(
                        run_id=state.run_id,
                        event_type="task.retrying",
                        task_id=task.id,
                        payload={
                            "task_id": task.id,
                            "next_attempt": attempt + 1,
                            "delay_ms": delay_ms,
                        },
                    )
                    await asyncio.sleep(delay_ms / 1000.0)
                else:
                    task.status = "failed"
                    task.error = str(e) or e.__class__.__name__
                    task.required = False
                    await ledger.save_task(task)
                    await ledger.append_event_transactional(
                        run_id=state.run_id,
                        event_type="task.failed",
                        task_id=task.id,
                        payload={
                            "task_id": task.id,
                            "attempt": attempt,
                            "error": task.error,
                            "can_retry": False,
                        },
                    )
                    return {"missing_research": {"company": task.error}}

    async def job_analysis_node(state: StaticWorkflowState) -> dict[str, Any]:
        check_lease()
        task_id = f"task_job_{state.run_id[:8]}"
        tasks = await ledger.list_tasks_for_run(state.run_id)
        matching_task = next((t for t in tasks if t.id == task_id), None)
        if matching_task and matching_task.status == "completed":
            job_existing = state.research_results.get("job")
            if not job_existing:
                job_existing = await completed_task_deliverable(state.run_id, task_id, "finding")
            return {"research_results": {"job": job_existing or {}}}

        start_t = time.perf_counter()
        task = WorkflowTask(
            id=task_id,
            run_id=state.run_id,
            task_type="research",
            name="Analyze Job Description & Skills",
            assigned_agent="job-analysis",
            dependencies=[f"task_normalize_{state.run_id[:8]}"],
            status="running",
        )
        first_attempt = await ledger.next_task_attempt(task.run_id, task.id)
        task.attempt = first_attempt
        await ledger.save_task(task)
        await ledger.append_event_transactional(
            run_id=state.run_id,
            event_type="task.started",
            task_id=task.id,
            payload={"task_id": task.id, "attempt": first_attempt, "assigned_agent": task.assigned_agent},
        )

        max_task_attempts = 2
        for attempt in range(first_attempt, first_attempt - 1 + max_task_attempts + 1):
            task.attempt = attempt
            await ledger.save_task(task)
            try:
                prior_artifacts = await ledger.list_artifacts_for_run(state.run_id)
                prior_ev_ids = [a.id for a in prior_artifacts if a.type == "evidence"]
                input_payload = {"role": state.role_title, "job_description": state.job_description}
                jd_evidence_id = f"art_jd_{state.run_id[:8]}"
                if any(a.id == jd_evidence_id for a in prior_artifacts):
                    input_payload["job_description_source"] = {
                        "url": state.jd_url,
                        "evidence_id": jd_evidence_id,
                    }
                if attempt > first_attempt:
                    input_payload["prior_evidence_ids"] = prior_ev_ids
                    input_payload["retry_attempt"] = attempt

                exec_rec, struct_job = await agent_runtime.execute_agent_loop(
                    agent_id="job-analysis",
                    objective=f"Extract key requirements for role {state.role_title}",
                    task_id=task.id,
                    run_id=state.run_id,
                    input_data=input_payload,
                )
                dur = (time.perf_counter() - start_t) * 1000
                task.status = "completed"
                task.execution_ids.append(exec_rec.id)
                await ledger.save_task(task)
                await ledger.append_event_transactional(
                    run_id=state.run_id,
                    event_type="task.completed",
                    task_id=task.id,
                    payload={
                        "task_id": task.id,
                        "duration_ms": dur,
                        "output_summary": exec_rec.public_summary,
                    },
                )
                return {
                    "research_results": {
                        "job": struct_job
                        if isinstance(struct_job, dict)
                        else (
                            struct_job.model_dump()
                            if hasattr(struct_job, "model_dump")
                            else exec_rec.public_summary
                        )
                    }
                }
            except Exception as e:
                logger.warning(f"Job analysis task attempt {attempt} failed: {e}")
                if attempt < first_attempt - 1 + max_task_attempts:
                    delay_ms = 1000 * (attempt - first_attempt + 1)
                    await ledger.append_event_transactional(
                        run_id=state.run_id,
                        event_type="task.retrying",
                        task_id=task.id,
                        payload={
                            "task_id": task.id,
                            "next_attempt": attempt + 1,
                            "delay_ms": delay_ms,
                        },
                    )
                    await asyncio.sleep(delay_ms / 1000.0)
                else:
                    task.status = "failed"
                    task.error = str(e)
                    await ledger.save_task(task)
                    await ledger.append_event_transactional(
                        run_id=state.run_id,
                        event_type="task.failed",
                        task_id=task.id,
                        payload={
                            "task_id": task.id,
                            "attempt": attempt,
                            "error": str(e),
                            "can_retry": False,
                        },
                    )
                    raise

    async def technical_research_node(state: StaticWorkflowState) -> dict[str, Any]:
        check_lease()
        task_id = f"task_tech_{state.run_id[:8]}"
        tasks = await ledger.list_tasks_for_run(state.run_id)
        matching_task = next((t for t in tasks if t.id == task_id), None)
        if matching_task and matching_task.status == "completed":
            tech_existing = state.research_results.get("tech")
            if not tech_existing:
                tech_existing = await completed_task_deliverable(state.run_id, task_id, "research_report")
            return {"research_results": {"tech": tech_existing or {}}}

        start_t = time.perf_counter()
        task = WorkflowTask(
            id=task_id,
            run_id=state.run_id,
            task_type="research",
            name="Research Technical Stack & Architecture",
            assigned_agent="technical-research",
            dependencies=[f"task_normalize_{state.run_id[:8]}"],
            status="running",
        )
        first_attempt = await ledger.next_task_attempt(task.run_id, task.id)
        task.attempt = first_attempt
        await ledger.save_task(task)
        await ledger.append_event_transactional(
            run_id=state.run_id,
            event_type="task.started",
            task_id=task.id,
            payload={"task_id": task.id, "attempt": first_attempt, "assigned_agent": task.assigned_agent},
        )
        max_task_attempts = 2
        for attempt in range(first_attempt, first_attempt - 1 + max_task_attempts + 1):
            task.attempt = attempt
            await ledger.save_task(task)
            try:
                prior_artifacts = await ledger.list_artifacts_for_run(state.run_id)
                prior_ev_ids = [a.id for a in prior_artifacts if a.type == "evidence"]
                input_payload = {"company": state.company_name}
                if attempt > first_attempt:
                    input_payload["prior_evidence_ids"] = prior_ev_ids
                    input_payload["retry_attempt"] = attempt
                exec_rec, struct_tech = await agent_runtime.execute_agent_loop(
                    agent_id="technical-research",
                    objective="Extract architectural patterns and distributed systems signals",
                    task_id=task.id,
                    run_id=state.run_id,
                    input_data=input_payload,
                )
                dur = (time.perf_counter() - start_t) * 1000
                task.status = "completed"
                task.execution_ids.append(exec_rec.id)
                await ledger.save_task(task)
                await ledger.append_event_transactional(
                    run_id=state.run_id,
                    event_type="task.completed",
                    task_id=task.id,
                    payload={
                        "task_id": task.id,
                        "duration_ms": dur,
                        "output_summary": exec_rec.public_summary,
                    },
                )
                return {
                    "research_results": {
                        "tech": struct_tech
                        if isinstance(struct_tech, dict)
                        else (
                            struct_tech.model_dump()
                            if hasattr(struct_tech, "model_dump")
                            else exec_rec.public_summary
                        )
                    }
                }
            except Exception as e:
                logger.warning(f"Technical research task attempt {attempt} failed: {e}")
                if attempt < first_attempt - 1 + max_task_attempts:
                    delay_ms = 1000 * (attempt - first_attempt + 1)
                    await ledger.append_event_transactional(
                        run_id=state.run_id,
                        event_type="task.retrying",
                        task_id=task.id,
                        payload={
                            "task_id": task.id,
                            "next_attempt": attempt + 1,
                            "delay_ms": delay_ms,
                        },
                    )
                    await asyncio.sleep(delay_ms / 1000.0)
                else:
                    task.status = "failed"
                    task.error = str(e) or e.__class__.__name__
                    task.required = False
                    await ledger.save_task(task)
                    await ledger.append_event_transactional(
                        run_id=state.run_id,
                        event_type="task.failed",
                        task_id=task.id,
                        payload={
                            "task_id": task.id,
                            "attempt": attempt,
                            "error": task.error,
                            "can_retry": False,
                        },
                    )
                    return {"missing_research": {"tech": task.error}}

    async def synthesize_node(state: StaticWorkflowState) -> dict[str, Any]:

        check_lease()
        task_id = f"task_synth_{state.run_id[:8]}"
        tasks = await ledger.list_tasks_for_run(state.run_id)
        matching_task = next((t for t in tasks if t.id == task_id), None)
        if matching_task and matching_task.status == "completed":
            match_existing = state.match_analysis
            if not match_existing:
                arts = await ledger.list_artifacts_for_run(state.run_id)
                task_art = next(
                    (a for a in arts if a.task_id == task_id or a.type == "validation_result"), None
                )
                if task_art:
                    match_existing = task_art.content
            return {"match_analysis": match_existing or {}}

        await set_phase(state, "mapping_resume_evidence", 45)
        start_t = time.perf_counter()
        task = WorkflowTask(
            id=task_id,
            run_id=state.run_id,
            task_type="synthesis",
            name="Synthesize Match Analysis & Evidence Evaluation",
            assigned_agent="evidence-evaluator",
            dependencies=list(state.upstream_task_ids)
            or [
                f"task_company_{state.run_id[:8]}",
                f"task_job_{state.run_id[:8]}",
                f"task_tech_{state.run_id[:8]}",
            ],
            status="running",
        )
        first_attempt = await ledger.next_task_attempt(task.run_id, task.id)
        task.attempt = first_attempt
        await ledger.save_task(task)
        await ledger.append_event_transactional(
            run_id=state.run_id,
            event_type="task.started",
            task_id=task.id,
            payload={"task_id": task.id, "attempt": first_attempt, "assigned_agent": task.assigned_agent},
        )

        missing = [key for key in ("company", "job", "tech") if key not in state.research_results]
        if "job" in missing:
            raise ValueError("Synthesis cannot run: job analysis did not produce results.")
        if "company" in missing and "tech" in missing:
            raise ValueError(
                "Synthesis cannot run: neither company research nor technical research produced results."
            )
        synthesis_input = {
            "candidate_cv": state.parsed_resume,
            "job_requirements": state.research_results["job"],
            "company_intelligence": state.research_results.get("company"),
            "technical_stack": state.research_results.get("tech"),
            "missing_research": [
                {"area": key, "reason": state.missing_research.get(key, "not run")} for key in missing
            ],
        }

        for attempt in range(first_attempt, first_attempt + 2):
            task.attempt = attempt
            await ledger.save_task(task)
            try:
                exec_rec, eval_data = await agent_runtime.execute_agent_loop(
                    agent_id="evidence-evaluator",
                    objective="Evaluate candidate CV and JD match, verify claims, and identify skill gaps",
                    task_id=task.id,
                    run_id=state.run_id,
                    input_data=synthesis_input,
                )

                dur = (time.perf_counter() - start_t) * 1000
                task.status = "completed"
                task.execution_ids.append(exec_rec.id)
                await ledger.save_task(task)
                await ledger.append_event_transactional(
                    run_id=state.run_id,
                    event_type="task.completed",
                    task_id=task.id,
                    payload={
                        "task_id": task.id,
                        "duration_ms": dur,
                        "output_summary": exec_rec.public_summary,
                    },
                )
                return {
                    "match_analysis": eval_data
                    if isinstance(eval_data, dict)
                    else eval_data.model_dump()
                }
            except Exception as e:
                logger.warning(f"Synthesize match analysis attempt {attempt} failed: {e}")
                if attempt < first_attempt - 1 + 2:
                    delay_ms = 1000 * (attempt - first_attempt + 1)
                    await ledger.append_event_transactional(
                        run_id=state.run_id,
                        event_type="task.retrying",
                        task_id=task.id,
                        payload={
                            "task_id": task.id,
                            "next_attempt": attempt + 1,
                            "delay_ms": delay_ms,
                        },
                    )
                    await asyncio.sleep(delay_ms / 1000.0)
                else:
                    task.status = "failed"
                    task.error = str(e)
                    await ledger.save_task(task)
                    await ledger.append_event_transactional(
                        run_id=state.run_id,
                        event_type="task.failed",
                        task_id=task.id,
                        payload={
                            "task_id": task.id,
                            "attempt": attempt,
                            "error": str(e),
                            "can_retry": False,
                        },
                    )
                    raise

    async def curriculum_node(state: StaticWorkflowState) -> dict[str, Any]:

        check_lease()
        task_id = f"task_curriculum_{state.run_id[:8]}"
        tasks = await ledger.list_tasks_for_run(state.run_id)
        matching_task = next((t for t in tasks if t.id == task_id), None)
        if matching_task and matching_task.status == "completed":
            curr_existing = state.curriculum
            if not curr_existing:
                arts = await ledger.list_artifacts_for_run(state.run_id)
                task_art = next(
                    (
                        a
                        for a in arts
                        if a.task_id == task_id
                        or (
                            a.type == "research_report"
                            and a.agent_execution_id
                            and "curriculum" in a.title.lower()
                        )
                    ),
                    None,
                )
                if task_art:
                    curr_existing = task_art.content
            return {"curriculum": curr_existing or {}}

        await set_phase(state, "constructing_knowledge_graph", 60)
        start_t = time.perf_counter()
        task = WorkflowTask(
            id=task_id,
            run_id=state.run_id,
            task_type="curriculum",
            name="Construct Knowledge Graph & Curriculum",
            assigned_agent="curriculum",
            dependencies=[f"task_synth_{state.run_id[:8]}"],
            status="running",
        )
        first_attempt = await ledger.next_task_attempt(task.run_id, task.id)
        task.attempt = first_attempt
        await ledger.save_task(task)
        await ledger.append_event_transactional(
            run_id=state.run_id,
            event_type="task.started",
            task_id=task.id,
            payload={"task_id": task.id, "attempt": first_attempt, "assigned_agent": task.assigned_agent},
        )

        match_gaps = (state.match_analysis or {}).get("gaps", [])
        job_skills = [
            r.get("skill")
            for r in (state.research_results.get("job", {}) or {}).get("requirements", [])
            if isinstance(r, dict)
        ]
        tech_topics = [
            t.get("name")
            for t in (state.research_results.get("tech", {}) or {}).get("topics", [])
            if isinstance(t, dict)
        ]

        all_artifacts = await ledger.list_artifacts_for_run(state.run_id)
        evidence_items = [
            {
                "id": a.id,
                "title": a.title,
                "url": a.evidence.source_url
                if a.evidence
                else (a.source_links[0].url if a.source_links else ""),
                "snippet": a.evidence.snippet
                if a.evidence
                else (
                    a.content.get("snippet", "") if isinstance(a.content, dict) else str(a.content)
                ),
            }
            for a in all_artifacts
            if a.type == "evidence"
        ]

        curriculum_input = {
            "target_company": state.company_name,
            "target_role": state.role_title,
            "identified_gaps": match_gaps[:5],
            "job_skills": job_skills[:8],
            "architecture_topics": tech_topics[:6],
            "evidence_citations": evidence_items,
        }

        async def generate_unit_batch(
            skill_batch: list[dict[str, Any]], all_skill_names: list[str]
        ) -> tuple[Any, list[dict[str, Any]]]:
            target_names = [s["name"] for s in skill_batch]
            rec, data = await agent_runtime.execute_agent_loop(
                agent_id="curriculum",
                objective=f"Write one prerequisite-aware learning unit for each of: {', '.join(target_names)}",
                task_id=task.id,
                run_id=state.run_id,
                input_data={
                    **curriculum_input,
                    "curriculum_stage": "units",
                    "target_skills": skill_batch,
                    "knowledge_graph_skills": all_skill_names,
                },
            )
            batch_dict = data if isinstance(data, dict) else data.model_dump()
            return rec, batch_dict["learning_units"]

        for attempt in range(first_attempt, first_attempt + 2):
            task.attempt = attempt
            await ledger.save_task(task)
            try:
                graph_rec, graph_data = await agent_runtime.execute_agent_loop(
                    agent_id="curriculum",
                    objective="Construct the knowledge graph of skills and prerequisite relations",
                    task_id=task.id,
                    run_id=state.run_id,
                    input_data={**curriculum_input, "curriculum_stage": "graph"},
                )
                graph_dict = graph_data if isinstance(graph_data, dict) else graph_data.model_dump()
                skills = graph_dict["skills"]
                skill_names = [s["name"] for s in skills]
                unit_targets = [s for s in skills if s["candidate_status"] != "verified"]
                if len(unit_targets) < CURRICULUM_MIN_UNITS:
                    unit_targets += [s for s in skills if s not in unit_targets]
                unit_targets = unit_targets[:CURRICULUM_MIN_UNITS]
                batches = [
                    unit_targets[i : i + CURRICULUM_UNIT_BATCH_SIZE]
                    for i in range(0, len(unit_targets), CURRICULUM_UNIT_BATCH_SIZE)
                ]
                unit_results = await asyncio.gather(
                    *(generate_unit_batch(b, skill_names) for b in batches)
                )

                learning_units: list[dict[str, Any]] = []
                task.execution_ids.append(graph_rec.id)
                for rec, units in unit_results:
                    task.execution_ids.append(rec.id)
                    learning_units.extend(units)

                dur = (time.perf_counter() - start_t) * 1000
                task.status = "completed"
                await ledger.save_task(task)
                await ledger.append_event_transactional(
                    run_id=state.run_id,
                    event_type="task.completed",
                    task_id=task.id,
                    payload={
                        "task_id": task.id,
                        "duration_ms": dur,
                        "output_summary": f"{graph_dict['summary']} Generated {len(learning_units)} learning units in {len(batches)} parallel batches.",
                    },
                )
                return {
                    "curriculum": {
                        "skills": skills,
                        "relations": graph_dict["relations"],
                        "learning_units": learning_units,
                        "summary": graph_dict["summary"],
                    }
                }
            except Exception as e:
                logger.warning(f"Curriculum generation task attempt {attempt} failed: {e}")
                if attempt < first_attempt - 1 + 2:
                    delay_ms = 1000 * (attempt - first_attempt + 1)
                    await ledger.append_event_transactional(
                        run_id=state.run_id,
                        event_type="task.retrying",
                        task_id=task.id,
                        payload={
                            "task_id": task.id,
                            "next_attempt": attempt + 1,
                            "delay_ms": delay_ms,
                        },
                    )
                    await asyncio.sleep(delay_ms / 1000.0)
                else:
                    task.status = "failed"
                    task.error = str(e)
                    await ledger.save_task(task)
                    await ledger.append_event_transactional(
                        run_id=state.run_id,
                        event_type="task.failed",
                        task_id=task.id,
                        payload={
                            "task_id": task.id,
                            "attempt": attempt,
                            "error": str(e),
                            "can_retry": False,
                        },
                    )
                    raise

    async def generate_questions_node(state: StaticWorkflowState) -> dict[str, Any]:

        check_lease()
        task_id = f"task_qgen_{state.run_id[:8]}"
        existing = await ledger.get_task(state.run_id, task_id)
        if existing is not None and existing.status == "completed" and not state.questions:
            stored_questions = existing.output_data.get("questions")
            if isinstance(stored_questions, list) and stored_questions:
                return {
                    "questions": stored_questions,
                    "validation_loop_count": state.validation_loop_count + 1,
                }
        await set_phase(state, "building_interview_question_book", 75)
        start_t = time.perf_counter()
        task = WorkflowTask(
            id=task_id,
            run_id=state.run_id,
            task_type="generation",
            name="Generate Question Book & Rubrics",
            assigned_agent="interview-generation",
            dependencies=[f"task_curriculum_{state.run_id[:8]}"],
            status="running",
        )
        first_attempt = await ledger.next_task_attempt(task.run_id, task.id)
        task.attempt = first_attempt
        await ledger.save_task(task)
        await ledger.append_event_transactional(
            run_id=state.run_id,
            event_type="task.started",
            task_id=task.id,
            payload={"task_id": task.id, "attempt": first_attempt, "assigned_agent": task.assigned_agent},
        )

        gaps_list = (state.match_analysis or {}).get("gaps", [])
        tech_top = [
            t.get("name")
            for t in (state.research_results.get("tech", {}) or {}).get("topics", [])
            if isinstance(t, dict)
        ]
        culture_notes = (state.research_results.get("company", {}) or {}).get(
            "engineering_culture", []
        )

        all_artifacts = await ledger.list_artifacts_for_run(state.run_id)
        evidence_items = [
            {
                "id": a.id,
                "title": a.title,
                "url": a.evidence.source_url
                if a.evidence
                else (a.source_links[0].url if a.source_links else ""),
                "snippet": a.evidence.snippet
                if a.evidence
                else (
                    a.content.get("snippet", "") if isinstance(a.content, dict) else str(a.content)
                ),
            }
            for a in all_artifacts
            if a.type == "evidence"
        ]

        qgen_input_base = {
            "company_name": state.company_name,
            "role_title": state.role_title,
            "target_topics": tech_top[:5],
            "candidate_gaps": gaps_list[:4],
            "culture_signals": culture_notes[:3],
            "evidence_citations": evidence_items,
        }

        batch_configs = [
            (cat, part, QUESTIONS_PER_BATCH)
            for cat in QUESTION_CATEGORIES
            for part in range(1, BATCHES_PER_CATEGORY + 1)
        ]

        for attempt in range(first_attempt, first_attempt + 2):
            task.attempt = attempt
            await ledger.save_task(task)
            try:

                async def generate_category(category_name: str, part: int, count: int):
                    cat_input = {
                        **qgen_input_base,
                        "target_topics": tech_top[part - 1 :: BATCHES_PER_CATEGORY][:3] or tech_top[:3],
                        "target_category": category_name,
                        "count": count,
                        "batch_part": part,
                        "batches_per_category": BATCHES_PER_CATEGORY,
                    }
                    rec, data = await agent_runtime.execute_agent_loop(
                        agent_id="interview-generation",
                        objective=(
                            f"Generate exactly {count} {category_name} interview questions with rigorous rubrics "
                            f"for {state.role_title} at {state.company_name} "
                            f"(batch {part} of {BATCHES_PER_CATEGORY}; focus on the given target_topics)"
                        ),
                        task_id=task.id,
                        run_id=state.run_id,
                        input_data=cat_input,
                    )
                    return rec, data

                async def safe_generate(cat: str, part: int, count: int):
                    for cat_att in range(1, 3):
                        try:
                            rec, data = await generate_category(cat, part, count)
                            return cat, rec, data
                        except Exception as ce:
                            logger.warning(f"Category {cat} batch {part} attempt {cat_att} error: {ce}")
                            if cat_att == 2:
                                raise
                    raise RuntimeError(f"Failed to generate questions for category {cat} batch {part}")

                results = await asyncio.gather(
                    *(safe_generate(cat, part, count) for cat, part, count in batch_configs)
                )

                all_raw_questions = []
                per_category: dict[str, int] = {cat: 0 for cat in QUESTION_CATEGORIES}
                for cat, rec, q_data in results:
                    task.execution_ids.append(rec.id)
                    raw_list = (
                        q_data.get("questions", [])
                        if isinstance(q_data, dict)
                        else getattr(q_data, "questions", [])
                    )
                    for item in raw_list:
                        all_raw_questions.append((cat, item))
                    per_category[cat] += len(raw_list)
                short = {c: n for c, n in per_category.items() if n < MIN_QUESTIONS_PER_CATEGORY}
                if short:
                    raise ValueError(
                        f"Question batches under-delivered (need >= {MIN_QUESTIONS_PER_CATEGORY} per category): {short}"
                    )

                dur = (time.perf_counter() - start_t) * 1000
                formatted_questions = list(state.questions)
                start_offset = len(formatted_questions)
                for idx, (batch_cat, q) in enumerate(all_raw_questions):
                    q_dict = q if isinstance(q, dict) else q.model_dump()
                    formatted_questions.append(
                        {
                            "id": f"q_{state.workspace_id}_{start_offset + idx + 1}",
                            "category": batch_cat,
                            "question": q_dict["question"],
                            "difficulty": q_dict["difficulty"],
                            "skills_tested": q_dict["skills_tested"],
                            "interviewer_intent": q_dict["interviewer_intent"],
                            "answer_outline": q_dict["answer_outline"],
                            "strong_answer_rubric": q_dict["expected_signals"],
                            "common_weaknesses": q_dict["pitfalls"],
                            "follow_up_questions": q_dict["follow_ups"],
                        }
                    )

                task.status = "completed"
                task.output_data = {"questions": formatted_questions}
                await ledger.save_task(task)
                await ledger.append_event_transactional(
                    run_id=state.run_id,
                    event_type="task.completed",
                    task_id=task.id,
                    payload={
                        "task_id": task.id,
                        "duration_ms": dur,
                        "output_summary": f"Generated {len(formatted_questions)} questions across {len(QUESTION_CATEGORIES)} categories in {len(batch_configs)} parallel batches.",
                    },
                )
                return {
                    "questions": formatted_questions,
                    "validation_loop_count": state.validation_loop_count + 1,
                }
            except Exception as e:
                logger.warning(f"Question generation task attempt {attempt} failed: {e}")
                if attempt < first_attempt - 1 + 2:
                    delay_ms = 1000 * (attempt - first_attempt + 1)
                    await ledger.append_event_transactional(
                        run_id=state.run_id,
                        event_type="task.retrying",
                        task_id=task.id,
                        payload={
                            "task_id": task.id,
                            "next_attempt": attempt + 1,
                            "delay_ms": delay_ms,
                        },
                    )
                    await asyncio.sleep(delay_ms / 1000.0)
                else:
                    task.status = "failed"
                    task.error = str(e)
                    await ledger.save_task(task)
                    await ledger.append_event_transactional(
                        run_id=state.run_id,
                        event_type="task.failed",
                        task_id=task.id,
                        payload={
                            "task_id": task.id,
                            "attempt": attempt,
                            "error": str(e),
                            "can_retry": False,
                        },
                    )
                    raise

    async def validate_questions_node(state: StaticWorkflowState) -> dict[str, Any]:

        check_lease()
        task_id = f"task_qval_{state.run_id[:8]}"
        existing = await ledger.get_task(state.run_id, task_id)
        if existing is not None and existing.status == "completed":
            stored_valid = existing.output_data.get("valid_questions")
            if isinstance(stored_valid, list) and len(stored_valid) >= 15:
                return {
                    "questions": stored_valid,
                    "validation_loop_count": state.validation_loop_count + 1,
                }
        start_t = time.perf_counter()
        task = WorkflowTask(
            id=task_id,
            run_id=state.run_id,
            task_type="validation",
            name="Audit Question Book & Evaluation Rubrics",
            assigned_agent="evidence-evaluator",
            dependencies=[f"task_qgen_{state.run_id[:8]}"],
            status="running",
        )
        first_attempt = await ledger.next_task_attempt(task.run_id, task.id)
        task.attempt = first_attempt
        await ledger.save_task(task)
        await ledger.append_event_transactional(
            run_id=state.run_id,
            event_type="task.started",
            task_id=task.id,
            payload={"task_id": task.id, "attempt": first_attempt, "assigned_agent": task.assigned_agent},
        )

        valid_questions = []
        for q in state.questions:
            q_text = q.get("question", "")
            outline = q.get("answer_outline", [])
            signals = q.get("strong_answer_rubric", [])
            if q_text and len(q_text) >= 10 and (outline or signals):
                valid_questions.append(q)

        dur = (time.perf_counter() - start_t) * 1000
        task.status = "completed"
        task.output_data = {
            "valid_count": len(valid_questions),
            "total_count": len(state.questions),
            "valid_questions": valid_questions,
        }
        await ledger.save_task(task)
        await ledger.append_event_transactional(
            run_id=state.run_id,
            event_type="task.completed",
            task_id=task.id,
            payload={
                "task_id": task.id,
                "duration_ms": dur,
                "output_summary": f"Audit accepted {len(valid_questions)} of {len(state.questions)} questions.",
            },
        )
        return {
            "questions": valid_questions,
            "validation_loop_count": state.validation_loop_count + 1,
        }

    def validate_condition(state: StaticWorkflowState) -> str:
        if len(state.questions) < 15 and state.validation_loop_count < 4:
            return "generate_questions"
        return "persist_outputs"

    async def persist_outputs_node(state: StaticWorkflowState) -> dict[str, Any]:

        check_lease()
        ws_id = state.workspace_id
        persist_task_id = f"task_persist_{state.run_id[:8]}"
        existing_persist = await ledger.get_task(state.run_id, persist_task_id)
        if existing_persist is None or existing_persist.status != "completed":
            await persist_domain_entities(state, persist_task_id)
        await finalize_run(state, ws_id)
        return {}

    async def persist_domain_entities(state: StaticWorkflowState, persist_task_id: str) -> None:
        while len(state.questions) < TOTAL_MIN_QUESTIONS:
            q_res = await generate_questions_node(state)
            state.questions = q_res.get("questions", state.questions)
        start_t = time.perf_counter()
        task = WorkflowTask(
            id=persist_task_id,
            run_id=state.run_id,
            task_type="persistence",
            name="Persist Domain Entities to SurrealDB",
            assigned_agent="curriculum",
            dependencies=[f"task_qgen_{state.run_id[:8]}"],
            status="running",
        )
        first_attempt = await ledger.next_task_attempt(task.run_id, task.id)
        task.attempt = first_attempt
        await ledger.save_task(task)
        await ledger.append_event_transactional(
            run_id=state.run_id,
            event_type="task.started",
            task_id=task.id,
            payload={"task_id": task.id, "attempt": first_attempt, "assigned_agent": task.assigned_agent},
        )

        ws_id = state.workspace_id

        cv_data = state.parsed_resume
        extracted_skills = cv_data["extracted_skills"]
        await persist_record(
            db_client=db,
            table="resume",
            record_id=f"res_{ws_id}",
            fields={
                "workspace_id": ws_id,
                "candidate_name": cv_data.get("candidate_name"),
                "summary": cv_data.get("summary"),
                "raw_text": state.resume_text,
                "parsed_skills": [s["name"] for s in extracted_skills],
                "extracted_skills": extracted_skills,
                "experience": [],
                "experience_highlights": cv_data.get("experience_highlights", []),
                "education": [],
            },
        )

        jd_res = state.research_results.get("job", {})
        jd_reqs = jd_res.get("requirements", []) if isinstance(jd_res, dict) else []
        jd_resps = jd_res.get("responsibilities", []) if isinstance(jd_res, dict) else []
        await persist_record(
            db_client=db,
            table="job_description",
            record_id=f"jd_{ws_id}",
            fields={
                "workspace_id": ws_id,
                "title": state.role_title,
                "role_summary": jd_res["role_summary"],
                "raw_text": state.job_description,
                "requirements": [r if isinstance(r, dict) else r.model_dump() for r in jd_reqs],
                "responsibilities": jd_resps,
                "technical_skills": [
                    r["skill"] if isinstance(r, dict) else r.skill for r in jd_reqs
                ],
            },
        )

        comp_data = state.research_results.get("company")
        if comp_data is not None and not isinstance(comp_data, dict):
            raise ValueError(
                f"Company research output for run {state.run_id} is not a structured dossier: {type(comp_data).__name__}"
            )
        all_run_artifacts = await ledger.list_artifacts_for_run(state.run_id)
        evidence_arts = [a for a in all_run_artifacts if a.type == "evidence" and a.evidence]
        source_by_evidence: dict[str, tuple[str, str, str]] = {}
        dossier_sources: list[dict[str, Any]] = []
        for ea in evidence_arts:
            prov = ea.evidence
            src_rec_id = f"src_{ea.id}"
            src_title = prov.title or ea.title
            await persist_record(
                db_client=db,
                table="source",
                record_id=src_rec_id,
                fields={
                    "workspace_id": ws_id,
                    "url": prov.source_url,
                    "title": src_title,
                    "domain": urlparse(prov.source_url).hostname or "",
                    "snippet": prov.snippet,
                    "source_type": "web_evidence",
                    "confidence": float(prov.confidence),
                    "retrieved_at": prov.retrieved_at.isoformat(),
                    "retrieval_method": "read_source",
                    "access_status": "ok",
                },
            )
            source_by_evidence[ea.id] = (src_rec_id, prov.source_url, prov.snippet)
            dossier_sources.append(
                {
                    "id": src_rec_id,
                    "source_url": prov.source_url,
                    "title": src_title,
                    "retrieved_at": prov.retrieved_at.isoformat(),
                    "snippet": prov.snippet,
                    "confidence": float(prov.confidence),
                }
            )

        if comp_data is not None:
            await persist_record(
                db_client=db,
                table="company_research",
                record_id=f"res_{ws_id}",
                fields={
                    "workspace_id": ws_id,
                    "company_name": state.company_name,
                    "dossier": {**comp_data, "sources": dossier_sources},
                },
            )

        company_claims = comp_data["claims"] if comp_data is not None else []
        for idx, claim in enumerate(company_claims):
            c_dict = claim if isinstance(claim, dict) else claim.model_dump()
            linked = [source_by_evidence[eid] for eid in c_dict["evidence_ids"] if eid in source_by_evidence]
            if not linked:
                continue
            src_rec_id, src_url, src_snippet = linked[0]
            await persist_record(
                db_client=db,
                table="claim",
                record_id=f"clm_{state.run_id[:8]}_{idx + 1}",
                fields={
                    "workspace_id": ws_id,
                    "statement": c_dict["statement"],
                    "claim_type": c_dict["category"],
                    "confidence": float(c_dict["confidence"]),
                    "evidence_snippet": src_snippet,
                    "source_id": src_rec_id,
                    "source_url": src_url,
                },
            )

        match_obj = state.match_analysis or {}
        await persist_record(
            db_client=db,
            table="match_analysis",
            record_id=f"match_{ws_id}",
            fields={
                "workspace_id": ws_id,
                "overall_match_ratio": float(match_obj["overall_match_ratio"]),
                "strengths": [
                    s if isinstance(s, dict) else s.model_dump()
                    for s in match_obj.get("strengths", [])
                ],
                "gaps": [
                    g if isinstance(g, dict) else g.model_dump() for g in match_obj.get("gaps", [])
                ],
                "risks": match_obj.get("risks", []),
                "wording_improvements": [
                    {
                        "original_text": w.get("original_text", w.get("original_bullet", "")),
                        "suggested_revision": w.get(
                            "suggested_revision", w.get("improved_bullet", "")
                        ),
                        "rationale": w.get("rationale", ""),
                    }
                    for w in match_obj.get("wording_improvements", [])
                ],
            },
        )

        curr_obj = state.curriculum or {}
        if not curr_obj.get("skills"):
            arts = await ledger.list_artifacts_for_run(state.run_id)
            c_art = next(
                (
                    a
                    for a in arts
                    if a.task_id == f"task_curriculum_{state.run_id[:8]}"
                    or (a.type == "research_report" and "curriculum" in a.title.lower())
                ),
                None,
            )
            if c_art and isinstance(c_art.content, dict):
                curr_obj = c_art.content

        skill_nodes = curr_obj.get("skills", [])
        if not skill_nodes:
            logger.error(
                f"Persist outputs for {state.run_id}: curr_obj has no skills! curr_obj keys: {list(curr_obj.keys())}, state.curriculum: {bool(state.curriculum)}"
            )
        for s in skill_nodes:
            s_dict = s if isinstance(s, dict) else s.model_dump()
            s_name = s_dict["name"]
            s_slug = s_dict["slug"]
            await persist_record(
                db_client=db,
                table="skill",
                record_id=f"skill_{s_slug}_{ws_id[:8]}",
                fields={
                    "workspace_id": ws_id,
                    "name": s_name,
                    "slug": s_slug,
                    "category": s_dict["category"],
                    "candidate_status": s_dict["candidate_status"],
                    "importance": s_dict["importance"],
                },
            )
        skill_edges = curr_obj.get("relations", [])
        for e in skill_edges:
            e_dict = e if isinstance(e, dict) else e.model_dump()
            src = e_dict["source"].lower().replace(" ", "-")
            tgt = e_dict["target"].lower().replace(" ", "-")
            await persist_record(
                db_client=db,
                table="skill_relation",
                record_id=f"rel_{src}_{tgt}_{ws_id[:8]}",
                fields={
                    "workspace_id": ws_id,
                    "source": e_dict["source"],
                    "target": e_dict["target"],
                    "relation": e_dict["relation"],
                },
            )

        learning_units = curr_obj.get("learning_units", [])
        for idx, lu in enumerate(learning_units):
            lu_dict = lu if isinstance(lu, dict) else lu.model_dump()
            await persist_record(
                db_client=db,
                table="learning_unit",
                record_id=f"lu_{ws_id}_{idx + 1}",
                fields={
                    "workspace_id": ws_id,
                    "skill": lu_dict["skill"],
                    "title": lu_dict["title"],
                    "prerequisites": lu_dict["prerequisites"],
                    "estimated_minutes": int(lu_dict["estimated_minutes"]),
                    "summary": lu_dict["summary"],
                    "content_markdown": lu_dict["deep_dive_markdown"],
                    "chapters": [
                        {
                            "id": s.get("session_id") or f"ses_{s_idx + 1}",
                            "title": s.get("title", ""),
                            "timestamp": f"{s.get('estimated_minutes', 15)}m",
                            "focus_area": s.get("focus_area", ""),
                            "content": s.get("content_markdown") or s.get("content", ""),
                            "summary": s.get("focus_area") or s.get("summary", ""),
                        }
                        for s_idx, s in enumerate(lu_dict.get("sessions", []))
                    ]
                    if lu_dict.get("sessions")
                    else None,
                },
            )

        for q in state.questions:
            await persist_record(
                db_client=db,
                table="interview_question",
                record_id=q["id"],
                fields={"workspace_id": ws_id, **q},
            )

        dur = (time.perf_counter() - start_t) * 1000
        task.status = "completed"
        await ledger.save_task(task)
        await ledger.append_event_transactional(
            run_id=state.run_id,
            event_type="task.completed",
            task_id=task.id,
            payload={
                "task_id": task.id,
                "duration_ms": dur,
                "output_summary": "All domain entities persisted to SurrealDB.",
            },
        )

    async def finalize_run(state: StaticWorkflowState, ws_id: str) -> None:
        def extract_result_rows(res: Any) -> list:
            if not isinstance(res, list):
                return []
            for item in res:
                if (
                    isinstance(item, dict)
                    and item.get("status") == "OK"
                    and isinstance(item.get("result"), list)
                ):
                    return item["result"]
            return []

        persisted_skills = await db.execute_query(
            "SELECT * FROM skill WHERE workspace_id = type::record('workspace', $ws);",
            vars={"ws": ws_id},
        )
        persisted_skill_count = len(extract_result_rows(persisted_skills))

        persisted_relations = await db.execute_query(
            "SELECT * FROM skill_relation WHERE workspace_id = type::record('workspace', $ws);",
            vars={"ws": ws_id},
        )
        persisted_rel_count = len(extract_result_rows(persisted_relations))

        persisted_units = await db.execute_query(
            "SELECT * FROM learning_unit WHERE workspace_id = type::record('workspace', $ws);",
            vars={"ws": ws_id},
        )
        persisted_unit_count = len(extract_result_rows(persisted_units))

        persisted_qs = await db.execute_query(
            "SELECT * FROM interview_question WHERE workspace_id = type::record('workspace', $ws);",
            vars={"ws": ws_id},
        )
        persisted_q_count = len(extract_result_rows(persisted_qs))

        if persisted_skill_count < 12:
            raise ValueError(
                f"Curriculum persistence check failed: expected >= 12 skills, got {persisted_skill_count}"
            )
        if persisted_rel_count < 10:
            raise ValueError(
                f"Curriculum persistence check failed: expected >= 10 relations, got {persisted_rel_count}"
            )
        if persisted_unit_count < 6:
            raise ValueError(
                f"Curriculum persistence check failed: expected >= 6 learning units, got {persisted_unit_count}"
            )
        if persisted_q_count < 15:
            raise ValueError(
                f"Question book persistence check failed: expected >= 15 questions, got {persisted_q_count}"
            )

        clean_run_id = validate_id(state.run_id, "run_id")
        progress = await ledger.refresh_run_progress(clean_run_id)
        run_record = await ledger.get_workflow_run(clean_run_id)
        now_completed_at = datetime.now(timezone.utc)
        started_at = run_record.started_at if run_record else None
        calculated_duration_ms = (
            (now_completed_at - started_at).total_seconds() * 1000 if started_at else 0.0
        )

        degraded_reasons = [
            f"{area} research unavailable: {reason}" if reason else f"{area} research unavailable"
            for area, reason in sorted(state.missing_research.items())
        ]
        await ledger.update_workflow_run(
            clean_run_id,
            {
                "status": "completed",
                "current_phase": "completed",
                "completed_at": now_completed_at,
                "duration_ms": calculated_duration_ms,
                "progress": progress.model_copy(update={"percentage": 100, "estimate": False}),
                "degraded": bool(degraded_reasons),
                "degraded_reasons": degraded_reasons,
            },
        )
        await ledger.sweep_run_to_terminal(clean_run_id, reason="run completed")

        from src.generation.progress import update_workspace_progress

        try:
            await update_workspace_progress(workspace_id=ws_id, phase="ready", percent=100, db=db)
        except Exception as pe:
            logger.warning(f"Could not update workspace progress: {pe}")

        total_tasks_count = progress.total_tasks

        artifacts_list = await ledger.list_artifacts_for_run(state.run_id)
        await ledger.append_event_transactional(
            run_id=state.run_id,
            event_type="workflow.completed",
            payload={
                "duration_ms": calculated_duration_ms,
                "total_tasks": total_tasks_count,
                "artifacts_count": len(artifacts_list),
                "degraded": bool(degraded_reasons),
                "degraded_reasons": degraded_reasons,
            },
        )

    builder = StateGraph(StaticWorkflowState)
    builder.add_node("synthesize", synthesize_node)
    builder.add_node("curriculum", curriculum_node)
    builder.add_node("generate_questions", generate_questions_node)
    builder.add_node("validate_questions", validate_questions_node)
    builder.add_node("persist_outputs", persist_outputs_node)
    if mode == "normalize_only":
        normalize_builder = StateGraph(StaticWorkflowState)
        normalize_builder.add_node("normalize", normalize_input_node)
        normalize_builder.add_edge(START, "normalize")
        normalize_builder.add_edge("normalize", END)
        return normalize_builder.compile(checkpointer=checkpointer)
    if mode == "outputs_only":
        builder.add_edge(START, "synthesize")
    else:
        builder.add_node("normalize", normalize_input_node)
        builder.add_node("company_research", company_research_node)
        builder.add_node("job_analysis", job_analysis_node)
        builder.add_node("technical_research", technical_research_node)
        builder.add_edge(START, "normalize")
        builder.add_edge("normalize", "company_research")
        builder.add_edge("normalize", "job_analysis")
        builder.add_edge("normalize", "technical_research")
        builder.add_edge("company_research", "synthesize")
        builder.add_edge("job_analysis", "synthesize")
        builder.add_edge("technical_research", "synthesize")
    builder.add_edge("synthesize", "curriculum")
    builder.add_edge("curriculum", "generate_questions")
    builder.add_edge("generate_questions", "validate_questions")
    builder.add_conditional_edges("validate_questions", validate_condition)
    builder.add_edge("persist_outputs", END)

    return builder.compile(checkpointer=checkpointer)
