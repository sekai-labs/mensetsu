import asyncio
import logging
import time
from datetime import datetime, timezone
from typing import Annotated, Any

from langgraph.graph import END, START, StateGraph
from langgraph.types import Send
from pydantic import BaseModel, Field

from src.agents.runtime import agent_runtime
from src.agents.workflow_router import route_workflow_request
from src.contracts.v1 import (
    Artifact,
    PlanTaskItem,
    TaskFailedPayload,
    WorkflowProgress,
    WorkflowTask,
)
from src.orchestration.dynamic_workflow import (
    PLANNABLE_AGENTS,
    merge_completed,
    merge_results,
    validate_plan_dag,
)
from src.orchestration.ledger import SurrealLedgerRepository


def check_lease() -> None:
    from src.agents.runtime import current_lost_lease_var

    lost_ev = current_lost_lease_var.get()
    if lost_ev and lost_ev.is_set():
        raise RuntimeError("Worker lost redis lease during execution. Aborting.")


logger = logging.getLogger("mensetsu.orchestration.hybrid_workflow")

CURRICULUM_MIN_UNITS = 6
CURRICULUM_UNIT_CONCURRENCY = 3
QUESTION_CATEGORIES = (
    "system_design",
    "target_stack",
    "technical_fundamentals",
    "resume_deep_dive",
    "project_deep_dive",
    "debugging",
    "behavioral",
)
QUESTIONS_PER_BATCH = 3
QUESTION_BATCHES_PER_CATEGORY = 5
QUESTION_GENERATION_CONCURRENCY = 3


def _question_batch_plan() -> list[tuple[str, int, int]]:
    return [
        (category, part, QUESTIONS_PER_BATCH)
        for category in QUESTION_CATEGORIES
        for part in range(1, QUESTION_BATCHES_PER_CATEGORY + 1)
    ]


def _task_failed_payload(task: WorkflowTask, error: str) -> dict[str, Any]:
    return TaskFailedPayload(
        task_id=task.id,
        attempt=task.attempt,
        error=error,
    ).model_dump()


async def _persist_hybrid_outputs(
    state: "HybridWorkflowState",
    ledger: SurrealLedgerRepository,
    artifacts: list[Artifact],
) -> None:
    from urllib.parse import urlparse

    from src.generation.persistence import persist_record

    ws_id = state.workspace_id
    prefix = state.run_id[:8]

    def deliverable(task_id: str, artifact_type: str) -> dict[str, Any]:
        candidates = [
            artifact
            for artifact in artifacts
            if artifact.task_id == task_id and artifact.type == artifact_type
        ]
        if not candidates:
            return {}
        content = max(candidates, key=lambda artifact: artifact.created_at).content
        return content if isinstance(content, dict) else {}

    company = state.research_results.get("company") or deliverable(
        f"task_company_{prefix}", "research_report"
    )
    job = state.research_results.get("job") or deliverable(f"task_job_{prefix}", "finding")
    match = deliverable(f"task_synthesize_{prefix}", "validation_result") or state.match_analysis
    curriculum_artifact = deliverable(f"task_curriculum_{prefix}", "research_report")
    curriculum = {**curriculum_artifact, **state.curriculum}
    questions = state.questions or deliverable(f"task_questions_{prefix}", "question_set").get(
        "questions", []
    )

    if state.parsed_resume:
        resume = state.parsed_resume
        skills = resume.get("extracted_skills", [])
        await persist_record(
            ledger.db,
            "resume",
            f"res_{ws_id}",
            {
                "workspace_id": ws_id,
                "candidate_name": resume.get("candidate_name"),
                "summary": resume.get("summary"),
                "raw_text": state.resume_text,
                "parsed_skills": [skill["name"] for skill in skills],
                "extracted_skills": skills,
                "experience": [],
                "experience_highlights": resume.get("experience_highlights", []),
                "education": [],
            },
        )
    if job:
        requirements = job.get("requirements", [])
        await persist_record(
            ledger.db,
            "job_description",
            f"jd_{ws_id}",
            {
                "workspace_id": ws_id,
                "title": state.role_title,
                "role_summary": job.get("role_summary", ""),
                "raw_text": state.job_description,
                "requirements": requirements,
                "responsibilities": job.get("responsibilities", []),
                "technical_skills": [
                    item.get("skill", "") if isinstance(item, dict) else str(item)
                    for item in requirements
                ],
            },
        )

    evidence_artifacts = [
        artifact for artifact in artifacts if artifact.type == "evidence" and artifact.evidence
    ]
    source_by_id: dict[str, tuple[str, str, str]] = {}
    company_evidence_ids = {
        evidence_id
        for claim in company.get("claims", [])
        for evidence_id in claim.get("evidence_ids", [])
    } | {
        evidence_id
        for item in company.get("tech_stack", [])
        for evidence_id in item.get("evidence_ids", [])
    }
    dossier_sources = []
    for evidence in evidence_artifacts:
        if evidence.id not in company_evidence_ids:
            continue
        provenance = evidence.evidence
        source_id = f"src_{evidence.id}"
        title = provenance.title or evidence.title
        await persist_record(
            ledger.db,
            "source",
            source_id,
            {
                "workspace_id": ws_id,
                "url": provenance.source_url,
                "title": title,
                "domain": urlparse(provenance.source_url).hostname or "",
                "snippet": provenance.snippet,
                "source_type": "web_evidence",
                "confidence": float(provenance.confidence),
                "retrieved_at": provenance.retrieved_at.isoformat(),
                "retrieval_method": "read_source",
                "access_status": "ok",
            },
        )
        source_by_id[evidence.id] = (source_id, provenance.source_url, provenance.snippet)
        dossier_sources.append(
            {
                "id": source_id,
                "source_url": provenance.source_url,
                "title": title,
                "retrieved_at": provenance.retrieved_at.isoformat(),
                "snippet": provenance.snippet,
                "confidence": float(provenance.confidence),
            }
        )

    if company:
        await persist_record(
            ledger.db,
            "company_research",
            f"res_{ws_id}",
            {
                "workspace_id": ws_id,
                "company_name": state.company_name,
                "dossier": {**company, "sources": dossier_sources},
            },
        )
        for index, claim in enumerate(company.get("claims", [])):
            linked = [
                source_by_id[evidence_id]
                for evidence_id in claim.get("evidence_ids", [])
                if evidence_id in source_by_id
            ]
            if linked:
                source_id, source_url, snippet = linked[0]
                await persist_record(
                    ledger.db,
                    "claim",
                    f"clm_{prefix}_{index + 1}",
                    {
                        "workspace_id": ws_id,
                        "statement": claim["statement"],
                        "claim_type": claim["category"],
                        "confidence": float(claim["confidence"]),
                        "evidence_snippet": snippet,
                        "source_id": source_id,
                        "source_url": source_url,
                    },
                )

    if match:
        await persist_record(
            ledger.db,
            "match_analysis",
            f"match_{ws_id}",
            {
                "workspace_id": ws_id,
                "overall_match_ratio": float(match["overall_match_ratio"]),
                "strengths": match.get("strengths", []),
                "gaps": match.get("gaps", []),
                "risks": match.get("risks", []),
                "wording_improvements": [
                    {
                        "original_text": item.get("original_text", item.get("original_bullet", "")),
                        "suggested_revision": item.get(
                            "suggested_revision", item.get("improved_bullet", "")
                        ),
                        "rationale": item.get("rationale", ""),
                    }
                    for item in match.get("wording_improvements", [])
                ],
            },
        )

    for skill in curriculum.get("skills", []):
        await persist_record(
            ledger.db,
            "skill",
            f"skill_{skill['slug']}_{ws_id[:8]}",
            {
                "workspace_id": ws_id,
                "name": skill["name"],
                "slug": skill["slug"],
                "category": skill["category"],
                "candidate_status": skill["candidate_status"],
                "importance": skill["importance"],
            },
        )
    for relation in curriculum.get("relations", []):
        source = relation["source"]
        target = relation["target"]
        relation_id = (
            f"rel_{source.lower().replace(' ', '-')}_{target.lower().replace(' ', '-')}_{ws_id[:8]}"
        )
        await persist_record(
            ledger.db,
            "skill_relation",
            relation_id,
            {"workspace_id": ws_id, **relation},
        )
    for index, unit in enumerate(curriculum.get("learning_units", [])):
        await persist_record(
            ledger.db,
            "learning_unit",
            f"lu_{ws_id}_{index + 1}",
            {
                "workspace_id": ws_id,
                "skill": unit["skill"],
                "title": unit["title"],
                "prerequisites": unit.get("prerequisites", []),
                "estimated_minutes": int(unit.get("estimated_minutes", 60)),
                "summary": unit["summary"],
                "content_markdown": unit["deep_dive_markdown"],
                "chapters": [
                    {
                        "id": session.get("session_id") or f"ses_{session_index + 1}",
                        "title": session.get("title", ""),
                        "timestamp": f"{session.get('estimated_minutes', 15)}m",
                        "focus_area": session.get("focus_area", ""),
                        "content": session.get("content_markdown") or session.get("content", ""),
                        "summary": session.get("focus_area") or session.get("summary", ""),
                    }
                    for session_index, session in enumerate(unit.get("sessions", []))
                ]
                if unit.get("sessions")
                else None,
            },
        )

    for index, question in enumerate(questions):
        question_id = question.get("id") or f"iq_{ws_id}_{index + 1}"
        await persist_record(
            ledger.db,
            "interview_question",
            question_id,
            {
                "workspace_id": ws_id,
                "category": question["category"],
                "question": question["question"],
                "difficulty": question["difficulty"],
                "skills_tested": question.get("skills_tested", []),
                "interviewer_intent": question.get("interviewer_intent", ""),
                "answer_outline": question.get("answer_outline", []),
                "strong_answer_rubric": question.get(
                    "strong_answer_rubric", question.get("expected_signals", [])
                ),
                "common_weaknesses": question.get(
                    "common_weaknesses", question.get("pitfalls", [])
                ),
                "follow_up_questions": question.get(
                    "follow_up_questions", question.get("follow_ups", [])
                ),
            },
        )


class HybridWorkflowState(BaseModel):
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
    user_prompt: str | None = None
    workflow_mode: str = "hybrid"
    llm_config: dict[str, Any] | None = None

    parsed_resume: dict[str, Any] = Field(default_factory=dict)
    jd_evidence_id: str | None = None
    plan_revision: int = 1
    tasks: list[dict[str, Any]] = Field(default_factory=list)
    completed_task_ids: Annotated[list[str], merge_completed] = Field(default_factory=list)
    research_results: Annotated[dict[str, Any], merge_results] = Field(default_factory=dict)
    failed_task_ids: Annotated[list[str], merge_completed] = Field(default_factory=list)
    missing_research: Annotated[dict[str, str], merge_results] = Field(default_factory=dict)

    match_analysis: dict[str, Any] = Field(default_factory=dict)
    curriculum: dict[str, Any] = Field(default_factory=dict)
    questions: list[dict[str, Any]] = Field(default_factory=list)
    missing_outputs: Annotated[dict[str, str], merge_results] = Field(default_factory=dict)


def build_hybrid_workflow(checkpointer=None, ledger: SurrealLedgerRepository | None = None):
    if ledger is None:
        from src.orchestration.shared import default_ledger

        ledger = default_ledger

    async def set_run_phase(
        run_id: str, phase: str, progress: int | None = None, completed: int = 0, total: int = 6
    ) -> None:
        all_tasks = await ledger.list_tasks_for_run(run_id)
        completed_count = len([t for t in all_tasks if t.status == "completed"])
        total_count = max(total, len(all_tasks))
        updates: dict[str, Any] = {
            "current_phase": phase,
            "progress": WorkflowProgress(
                total_tasks=total_count,
                completed=completed_count,
                running=1 if progress and progress < 100 else 0,
                percentage=progress
                or (int((completed_count / total_count) * 100) if total_count else 0),
            ),
        }
        await ledger.update_workflow_run(run_id, updates)

        from src.generation.progress import update_workspace_progress

        ws_rec = await ledger.get_workflow_run(run_id)
        if ws_rec and ws_rec.workspace_id:
            try:
                await update_workspace_progress(
                    workspace_id=ws_rec.workspace_id,
                    phase=phase,
                    percent=updates["progress"].percentage or 0,
                    db=ledger.db,
                )
            except Exception as pe:
                logger.warning(
                    f"Could not update workspace progress for {ws_rec.workspace_id}: {pe}"
                )

    async def route_and_normalize_node(state: HybridWorkflowState) -> dict[str, Any]:
        check_lease()
        await set_run_phase(state.run_id, "analyzing_input", 5)

        prompt = state.user_prompt or ""
        routing = await route_workflow_request(
            user_prompt=prompt,
            company_name=state.company_name,
            role_title=state.role_title,
            resume_text=state.resume_text,
            job_description=state.job_description,
            seniority=state.seniority,
            llm_config=state.llm_config,
        )

        resolved_company = routing.extracted_company or state.company_name
        resolved_role = routing.extracted_role or state.role_title

        routing_art = Artifact(
            id=f"art_routing_{state.run_id[:8]}",
            run_id=state.run_id,
            task_id="task_route",
            agent_execution_id="workflow_router",
            type="routing_decision",
            title="Workflow Routing Decision",
            content=routing.model_dump(),
        )
        await ledger.save_artifact(routing_art)
        initial_plan_items = [
            PlanTaskItem(
                id=f"task_company_{state.run_id[:8]}",
                name="Company Research",
                assigned_agent="company-research",
                dependencies=[],
                required=True,
            ),
            PlanTaskItem(
                id=f"task_tech_{state.run_id[:8]}",
                name="Technical Research",
                assigned_agent="technical-research",
                dependencies=[],
                required=True,
            ),
            PlanTaskItem(
                id=f"task_job_{state.run_id[:8]}",
                name="Job Analysis",
                assigned_agent="job-analysis",
                dependencies=[],
                required=True,
            ),
        ]
        await ledger.append_event_transactional(
            run_id=state.run_id,
            event_type="plan.created",
            payload={
                "revision": 1,
                "tasks": [t.model_dump() for t in initial_plan_items],
            },
        )

        from src.orchestration.static_workflow import StaticWorkflowState, build_static_workflow

        normalize_graph = build_static_workflow(
            checkpointer=checkpointer, ledger=ledger, db_client=ledger.db, mode="normalize_only"
        )
        normalized = await normalize_graph.ainvoke(
            StaticWorkflowState(
                run_id=state.run_id,
                workspace_id=state.workspace_id,
                user_id=state.user_id,
                company_name=resolved_company,
                role_title=resolved_role,
                company_urls=state.company_urls,
                resume_text=state.resume_text,
                resume_upload_id=state.resume_upload_id,
                resume_filename=state.resume_filename,
                job_description=state.job_description,
                jd_url=state.jd_url,
                seniority=state.seniority,
            ),
            config={"configurable": {"thread_id": f"{state.run_id}__normalize"}},
        )

        return {
            "company_name": resolved_company,
            "role_title": resolved_role,
            "resume_text": normalized.get("resume_text", state.resume_text),
            "job_description": normalized.get("job_description", state.job_description),
            "parsed_resume": normalized.get("parsed_resume", {}),
            "jd_evidence_id": f"art_jd_{state.run_id[:8]}"
            if state.jd_url and not state.job_description.strip()
            else None,
            "resume_upload_id": None,
        }

    async def planner_node(state: HybridWorkflowState) -> dict[str, Any]:
        check_lease()
        await set_run_phase(state.run_id, "planning", 15)

        task_id = f"task_plan_{state.run_id[:8]}_v{state.plan_revision}"
        start_t = time.perf_counter()

        plan_task = WorkflowTask(
            id=task_id,
            run_id=state.run_id,
            task_type="planning",
            name=f"Synthesize Dynamic Research Plan (Revision {state.plan_revision})",
            assigned_agent="planner",
            status="running",
        )
        await ledger.save_task(plan_task)
        await ledger.append_event_transactional(
            run_id=state.run_id,
            event_type="task.started",
            task_id=plan_task.id,
            payload={"task_id": plan_task.id, "attempt": 1, "assigned_agent": "planner"},
        )

        _plan_task_id = f"task_plan_{state.run_id[:8]}_v{state.plan_revision}"
        planned_tasks = [
            {
                "id": f"task_company_{state.run_id[:8]}",
                "agent": "company-research",
                "assigned_agent": "company-research",
                "name": "Company Research",
                "objective": f"Investigate {state.company_name} engineering culture and stack, focusing on: {state.user_prompt or 'general architecture'}",
                "dependencies": [],
                "required": True,
            },
            {
                "id": f"task_tech_{state.run_id[:8]}",
                "agent": "technical-research",
                "assigned_agent": "technical-research",
                "name": "Technical Research",
                "objective": f"Deep dive into {state.company_name}'s production architecture, technologies, and concerns relevant to {state.role_title}",
                "dependencies": [],
                "required": True,
            },
            {
                "id": f"task_job_{state.run_id[:8]}",
                "agent": "job-analysis",
                "assigned_agent": "job-analysis",
                "name": "Job Analysis",
                "objective": f"Extract role requirements, constraints, and seniority expectations for {state.role_title}",
                "dependencies": [],
                "required": True,
            },
        ]

        new_tasks = [t for t in planned_tasks if t["id"] not in state.completed_task_ids]

        validate_plan_dag(new_tasks)

        dur = (time.perf_counter() - start_t) * 1000
        plan_task.status = "completed"
        await ledger.save_task(plan_task)
        await ledger.append_event_transactional(
            run_id=state.run_id,
            event_type="task.completed",
            task_id=plan_task.id,
            payload={
                "task_id": plan_task.id,
                "duration_ms": dur,
                "output_summary": f"Planned {len(new_tasks)} research tasks",
            },
        )

        return {"tasks": new_tasks}

    def dynamic_router(state: HybridWorkflowState):
        ready = [
            t
            for t in state.tasks
            if t["id"] not in state.completed_task_ids
            and t["id"] not in state.failed_task_ids
            and all(dep in state.completed_task_ids for dep in t.get("dependencies", []))
        ]
        if ready:
            return [
                Send(
                    "execute_dynamic_task",
                    {
                        "task_info": t,
                        "run_id": state.run_id,
                        "workspace_id": state.workspace_id,
                        "company_name": state.company_name,
                        "role_title": state.role_title,
                        "company_urls": state.company_urls,
                        "parsed_resume": state.parsed_resume,
                        "job_description": state.job_description,
                        "user_prompt": state.user_prompt,
                    },
                )
                for t in ready
            ]
        return "synthesize"

    async def execute_dynamic_task_node(inputs: dict[str, Any]) -> dict[str, Any]:
        check_lease()
        t_info = inputs["task_info"]
        run_id = inputs["run_id"]
        task_id = t_info["id"]
        agent_id = t_info.get("assigned_agent") or t_info.get("agent")
        objective = t_info["objective"]

        start_t = time.perf_counter()
        task = WorkflowTask(
            id=task_id,
            run_id=run_id,
            task_type="research",
            name=f"{agent_id}: {objective[:60]}",
            assigned_agent=agent_id,
            dependencies=t_info.get("dependencies", []),
            status="running",
        )
        await ledger.save_task(task)
        await ledger.append_event_transactional(
            run_id=run_id,
            event_type="task.started",
            task_id=task.id,
            payload={"task_id": task.id, "attempt": 1, "assigned_agent": agent_id},
        )

        task_input = {
            "company_name": inputs["company_name"],
            "role_title": inputs["role_title"],
            "company_urls": inputs.get("company_urls", []),
            "candidate_cv": inputs["parsed_resume"],
            "job_description": inputs["job_description"],
            "user_prompt": inputs.get("user_prompt", ""),
            "objective": objective,
        }

        try:
            exec_rec, res_data = await agent_runtime.execute_agent_loop(
                agent_id=agent_id,
                objective=objective,
                task_id=task.id,
                run_id=run_id,
                input_data=task_input,
            )
            dur = (time.perf_counter() - start_t) * 1000
            task.status = "completed"
            task.execution_ids.append(exec_rec.id)
            await ledger.save_task(task)
            await ledger.append_event_transactional(
                run_id=run_id,
                event_type="task.completed",
                task_id=task.id,
                payload={
                    "task_id": task.id,
                    "duration_ms": dur,
                    "output_summary": exec_rec.public_summary,
                },
            )

            res_dict = (
                res_data
                if isinstance(res_data, dict)
                else (res_data.model_dump() if hasattr(res_data, "model_dump") else {})
            )
            cat = PLANNABLE_AGENTS.get(agent_id, agent_id)
            return {
                "completed_task_ids": [task.id],
                "research_results": {cat: res_dict},
            }
        except Exception as e:
            task.status = "failed"
            task.error = str(e)
            await ledger.save_task(task)
            await ledger.append_event_transactional(
                run_id=run_id,
                event_type="task.failed",
                task_id=task.id,
                payload=_task_failed_payload(task, str(e)),
            )
            cat = PLANNABLE_AGENTS.get(agent_id, agent_id)
            return {
                "failed_task_ids": [task.id],
                "missing_research": {cat: str(e)},
            }

    async def synthesize_node(state: HybridWorkflowState) -> dict[str, Any]:
        check_lease()
        await set_run_phase(state.run_id, "synthesizing_match", 60)

        task_id = f"task_synthesize_{state.run_id[:8]}"
        start_t = time.perf_counter()
        task = WorkflowTask(
            id=task_id,
            run_id=state.run_id,
            task_type="synthesis",
            name="Synthesize Match Analysis & Evidence Evaluation",
            assigned_agent="evidence-evaluator",
            dependencies=[t["id"] for t in state.tasks],
            status="running",
        )
        await ledger.save_task(task)
        await ledger.append_event_transactional(
            run_id=state.run_id,
            event_type="task.started",
            task_id=task.id,
            payload={"task_id": task.id, "attempt": 1, "assigned_agent": task.assigned_agent},
        )

        synthesis_input = {
            "candidate_cv": state.parsed_resume,
            "job_requirements": state.research_results.get("job", {"requirements": []}),
            "company_intelligence": state.research_results.get("company", {}),
            "technical_stack": state.research_results.get("tech", {}),
            "missing_research": [
                {"area": k, "reason": v} for k, v in state.missing_research.items()
            ],
            "user_prompt": state.user_prompt or "",
        }
        await ledger.append_event_transactional(
            run_id=state.run_id,
            event_type="agent.activity",
            task_id=task.id,
            execution_id=task.id,
            payload={
                "execution_id": task.id,
                "activity_type": "evaluator_synthesizing",
                "description": f"Auditing {len(state.research_results)} research sources and candidate CV against {state.role_title} requirements...",
            },
        )

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
            dict_eval = eval_data if isinstance(eval_data, dict) else eval_data.model_dump()
            return {"match_analysis": dict_eval, "completed_task_ids": [task.id]}
        except Exception as e:
            logger.warning(f"Synthesis failed ({e}).")
            task.status = "failed"
            task.error = str(e)
            await ledger.save_task(task)
            await ledger.append_event_transactional(
                run_id=state.run_id,
                event_type="task.failed",
                task_id=task.id,
                payload=_task_failed_payload(task, str(e)),
            )
            return {
                "match_analysis": {},
                "failed_task_ids": [task.id],
                "missing_outputs": {"match analysis": str(e)},
            }

    async def curriculum_node(state: HybridWorkflowState) -> dict[str, Any]:
        return await execute_curriculum_node(state, ledger, set_run_phase)

    async def interview_node(state: HybridWorkflowState) -> dict[str, Any]:
        check_lease()
        await set_run_phase(state.run_id, "building_interview_question_book", 90)
        start_t = time.perf_counter()

        task_id = f"task_questions_{state.run_id[:8]}"
        task = WorkflowTask(
            id=task_id,
            run_id=state.run_id,
            task_type="generation",
            name="Generate Interview Question Book",
            assigned_agent="interview-generation",
            dependencies=[f"task_curriculum_{state.run_id[:8]}"],
            status="running",
        )
        await ledger.save_task(task)
        await ledger.append_event_transactional(
            run_id=state.run_id,
            event_type="task.started",
            task_id=task.id,
            payload={"task_id": task.id, "attempt": 1, "assigned_agent": task.assigned_agent},
        )

        artifacts = await ledger.list_artifacts_for_run(state.run_id)

        def artifact_content(task_id: str, artifact_type: str) -> dict[str, Any]:
            candidates = [
                artifact
                for artifact in artifacts
                if artifact.task_id == task_id and artifact.type == artifact_type
            ]
            if not candidates:
                return {}
            content = max(candidates, key=lambda artifact: artifact.created_at).content
            return content if isinstance(content, dict) else {}

        prefix = state.run_id[:8]
        company = state.research_results.get("company") or artifact_content(
            f"task_company_{prefix}", "research_report"
        )
        technical = state.research_results.get("tech") or artifact_content(
            f"task_tech_{prefix}", "research_report"
        )
        topics = [
            item.get("name", "")
            for item in technical.get("topics", [])
            if isinstance(item, dict) and item.get("name")
        ]
        topics.extend(
            item.get("name", "")
            for item in company.get("tech_stack", [])
            if isinstance(item, dict) and item.get("name")
        )
        evidence_citations = [
            {
                "id": artifact.id,
                "title": artifact.evidence.title or artifact.title,
                "url": artifact.evidence.source_url,
                "snippet": artifact.evidence.snippet,
            }
            for artifact in artifacts
            if artifact.type == "evidence" and artifact.evidence
        ][:10]
        candidate_background = {
            "summary": state.parsed_resume.get("summary", ""),
            "skills": [
                skill.get("name", "")
                for skill in state.parsed_resume.get("extracted_skills", [])
                if isinstance(skill, dict) and skill.get("name")
            ][:12],
            "experience_highlights": state.parsed_resume.get("experience_highlights", [])[:5],
        }
        input_base = {
            "company_name": state.company_name,
            "role_title": state.role_title,
            "target_topics": topics[:5],
            "candidate_gaps": state.match_analysis.get("gaps", [])[:4],
            "culture_signals": company.get("engineering_culture", [])[:3],
            "candidate_background": candidate_background,
            "evidence_citations": evidence_citations,
            "user_prompt": state.user_prompt or "",
        }

        semaphore = asyncio.Semaphore(QUESTION_GENERATION_CONCURRENCY)

        async def generate_batch(
            category: str, part: int, count: int
        ) -> tuple[str, int, Any | None, list[Any], str | None]:
            objective = (
                f"Generate exactly {count} distinct {category} interview questions with complete "
                f"evaluation rubrics for {state.role_title} at {state.company_name} "
                f"(batch {part} of {QUESTION_BATCHES_PER_CATEGORY})."
            )
            batch_input = {
                **input_base,
                "target_category": category,
                "count": count,
                "batch_part": part,
                "batches_per_category": QUESTION_BATCHES_PER_CATEGORY,
            }
            async with semaphore:
                for attempt in range(2):
                    try:
                        execution, result = await agent_runtime.execute_agent_loop(
                            agent_id="interview-generation",
                            objective=objective,
                            task_id=task.id,
                            run_id=state.run_id,
                            input_data=batch_input,
                        )
                        result_dict = result if isinstance(result, dict) else result.model_dump()
                        questions = result_dict.get("questions", [])
                        if len(questions) != count:
                            raise ValueError(
                                f"{category} batch {part} returned {len(questions)} questions; expected {count}"
                            )
                        return category, part, execution, questions, None
                    except Exception as exc:
                        if attempt == 1:
                            return category, part, None, [], str(exc)
                        logger.warning(
                            f"Question batch {category}/{part} retrying after failure: {exc}"
                        )
            return category, part, None, [], "question batch exhausted retries"

        results = await asyncio.gather(
            *(
                generate_batch(category, part, count)
                for category, part, count in _question_batch_plan()
            )
        )
        per_category = {category: 0 for category in QUESTION_CATEGORIES}
        formatted_questions = []
        failures = []
        seen_questions = set()
        for category, part, execution, batch, error in results:
            if execution is not None:
                task.execution_ids.append(execution.id)
            if error is not None:
                failures.append(f"{category} batch {part}: {error}")
                continue
            for question in batch:
                item = question if isinstance(question, dict) else question.model_dump()
                normalized = " ".join(item["question"].casefold().split())
                if normalized in seen_questions:
                    failures.append(f"{category} batch {part}: duplicate question")
                    continue
                seen_questions.add(normalized)
                per_category[category] += 1
                index = len(formatted_questions) + 1
                formatted_questions.append(
                    {
                        "id": f"q_{state.workspace_id}_{index}",
                        "category": category,
                        "question": item["question"],
                        "difficulty": item["difficulty"],
                        "skills_tested": item["skills_tested"],
                        "interviewer_intent": item["interviewer_intent"],
                        "answer_outline": item["answer_outline"],
                        "strong_answer_rubric": item["expected_signals"],
                        "common_weaknesses": item["pitfalls"],
                        "follow_up_questions": item["follow_ups"],
                    }
                )

        missing_categories = {
            category: count
            for category, count in per_category.items()
            if count < QUESTIONS_PER_BATCH * QUESTION_BATCHES_PER_CATEGORY
        }
        complete = len(formatted_questions) >= 100 and not missing_categories
        task.output_data = {"questions": formatted_questions}
        task.status = "completed" if complete else "failed"
        if not complete:
            failures.append(
                f"question bank incomplete: {len(formatted_questions)}/100 questions; "
                f"category counts {missing_categories}"
            )
            task.error = "; ".join(failures[:8])
        await ledger.save_task(task)
        duration_ms = (time.perf_counter() - start_t) * 1000
        if complete:
            await ledger.append_event_transactional(
                run_id=state.run_id,
                event_type="task.completed",
                task_id=task.id,
                payload={
                    "task_id": task.id,
                    "duration_ms": duration_ms,
                    "output_summary": (
                        f"Generated {len(formatted_questions)} interview questions "
                        f"across {len(QUESTION_CATEGORIES)} categories."
                    ),
                },
            )
            return {
                "questions": formatted_questions,
                "completed_task_ids": [task.id],
            }

        await ledger.append_event_transactional(
            run_id=state.run_id,
            event_type="task.failed",
            task_id=task.id,
            payload=_task_failed_payload(task, task.error or "incomplete question bank"),
        )
        return {
            "questions": formatted_questions,
            "failed_task_ids": [task.id],
            "missing_outputs": {"interview question bank": task.error or "incomplete output"},
        }

    async def finalize_node(state: HybridWorkflowState) -> dict[str, Any]:
        now_completed = datetime.now(timezone.utc)
        all_tasks = await ledger.list_tasks_for_run(state.run_id)
        completed_tasks = [t for t in all_tasks if t.status == "completed"]
        total_tasks_count = max(len(all_tasks), len(completed_tasks), 6)

        run_rec = await ledger.get_workflow_run(state.run_id)
        started_at = run_rec.started_at if run_rec else None
        calc_duration_ms = (
            (now_completed - started_at).total_seconds() * 1000 if started_at else 0.0
        )
        all_artifacts = await ledger.list_artifacts_for_run(state.run_id)
        await _persist_hybrid_outputs(state, ledger, all_artifacts)
        degraded_reasons = [
            f"{area} research failed: {reason}"
            for area, reason in sorted(state.missing_research.items())
        ] + [
            f"{area} generation failed: {reason}"
            for area, reason in sorted(state.missing_outputs.items())
        ]
        await ledger.update_workflow_run(
            state.run_id,
            {
                "status": "completed",
                "current_phase": "completed",
                "completed_at": now_completed,
                "duration_ms": calc_duration_ms,
                "degraded": bool(degraded_reasons),
                "degraded_reasons": degraded_reasons,
                "progress": WorkflowProgress(
                    total_tasks=total_tasks_count,
                    completed=len(completed_tasks),
                    running=0,
                    percentage=100,
                ),
            },
        )
        await ledger.append_event_transactional(
            run_id=state.run_id,
            event_type="workflow.completed",
            payload={
                "duration_ms": calc_duration_ms,
                "total_tasks": total_tasks_count,
                "artifacts_count": len(all_artifacts),
                "degraded": bool(degraded_reasons),
                "degraded_reasons": degraded_reasons,
            },
        )
        from src.generation.progress import update_workspace_progress

        try:
            await update_workspace_progress(
                workspace_id=state.workspace_id,
                phase="ready",
                percent=100,
                db=ledger.db,
            )
        except Exception as pe:
            logger.warning(
                f"Could not update workspace progress ready for {state.workspace_id}: {pe}"
            )
        return {}

    builder = StateGraph(HybridWorkflowState)
    builder.add_node("route_and_normalize", route_and_normalize_node)
    builder.add_node("planner", planner_node)
    builder.add_node("execute_dynamic_task", execute_dynamic_task_node)
    builder.add_node("synthesize", synthesize_node)
    builder.add_node("curriculum", curriculum_node)
    builder.add_node("interview", interview_node)
    builder.add_node("finalize", finalize_node)

    builder.add_edge(START, "route_and_normalize")
    builder.add_edge("route_and_normalize", "planner")
    builder.add_conditional_edges("planner", dynamic_router, ["execute_dynamic_task", "synthesize"])
    builder.add_conditional_edges(
        "execute_dynamic_task", dynamic_router, ["execute_dynamic_task", "synthesize"]
    )
    builder.add_edge("synthesize", "curriculum")
    builder.add_edge("curriculum", "interview")
    builder.add_edge("interview", "finalize")
    builder.add_edge("finalize", END)

    return builder.compile(checkpointer=checkpointer)


async def execute_curriculum_node(
    state: HybridWorkflowState, ledger: SurrealLedgerRepository, set_run_phase=None
) -> dict[str, Any]:
    check_lease()
    if set_run_phase:
        await set_run_phase(state.run_id, "constructing_knowledge_graph", 75)
    start_t = time.perf_counter()

    task_id = f"task_curriculum_{state.run_id[:8]}"
    task = WorkflowTask(
        id=task_id,
        run_id=state.run_id,
        task_type="curriculum",
        name="Construct Knowledge Graph & Curriculum",
        assigned_agent="curriculum",
        dependencies=[f"task_synthesize_{state.run_id[:8]}"],
        status="running",
    )
    await ledger.save_task(task)
    await ledger.append_event_transactional(
        run_id=state.run_id,
        event_type="task.started",
        task_id=task.id,
        payload={"task_id": task.id, "attempt": 1, "assigned_agent": task.assigned_agent},
    )

    match_gaps = (state.match_analysis or {}).get("gaps", [])
    job = state.research_results.get("job", {}) or {}
    tech = state.research_results.get("tech", {}) or {}
    all_artifacts = await ledger.list_artifacts_for_run(state.run_id)
    evidence_items = [
        {
            "id": artifact.id,
            "title": artifact.evidence.title or artifact.title,
            "url": artifact.evidence.source_url,
            "snippet": artifact.evidence.snippet,
        }
        for artifact in all_artifacts
        if artifact.type == "evidence" and artifact.evidence
    ][:10]
    curriculum_input = {
        "target_company": state.company_name,
        "target_role": state.role_title,
        "identified_gaps": match_gaps[:5],
        "job_skills": [
            item.get("skill", "") for item in job.get("requirements", []) if isinstance(item, dict)
        ][:8],
        "architecture_topics": [
            item.get("name", "")
            for item in tech.get("topics", [])
            if isinstance(item, dict) and item.get("name")
        ][:6],
        "evidence_citations": evidence_items,
        "user_prompt": state.user_prompt or "",
    }
    await ledger.append_event_transactional(
        run_id=state.run_id,
        event_type="agent.activity",
        task_id=task.id,
        execution_id=task.id,
        payload={
            "execution_id": task.id,
            "activity_type": "curriculum_graph",
            "description": f"Building the prerequisite graph for {state.role_title}.",
        },
    )

    try:
        graph_rec, graph_data = await agent_runtime.execute_agent_loop(
            agent_id="curriculum",
            objective="Construct the knowledge graph of skills and prerequisite relations",
            task_id=task.id,
            run_id=state.run_id,
            input_data={**curriculum_input, "curriculum_stage": "graph"},
        )
        graph_dict = graph_data if isinstance(graph_data, dict) else graph_data.model_dump()
    except Exception as exc:
        logger.warning(f"Knowledge graph generation failed ({exc}).")
        task.status = "failed"
        task.error = str(exc)
        await ledger.save_task(task)
        await ledger.append_event_transactional(
            run_id=state.run_id,
            event_type="task.failed",
            task_id=task.id,
            payload=_task_failed_payload(task, str(exc)),
        )
        return {
            "curriculum": {},
            "failed_task_ids": [task.id],
            "missing_outputs": {"knowledge graph": str(exc)},
        }

    task.execution_ids.append(graph_rec.id)
    skills = graph_dict.get("skills", [])
    skill_names = [skill.get("name", "") for skill in skills if isinstance(skill, dict)]
    targets = [
        skill
        for skill in skills
        if isinstance(skill, dict) and skill.get("candidate_status") != "verified"
    ]
    if len(targets) < CURRICULUM_MIN_UNITS:
        targets.extend(skill for skill in skills if skill not in targets)
    targets = targets[:CURRICULUM_MIN_UNITS]
    if set_run_phase:
        await set_run_phase(state.run_id, "generating_lessons", 80)

    await ledger.append_event_transactional(
        run_id=state.run_id,
        event_type="agent.activity",
        task_id=task.id,
        execution_id=task.id,
        payload={
            "execution_id": task.id,
            "activity_type": "curriculum_units",
            "description": (
                f"Generating one evidence-grounded lesson for each of "
                f"{len(targets)} prioritized skills."
            ),
        },
    )
    semaphore = asyncio.Semaphore(CURRICULUM_UNIT_CONCURRENCY)

    async def generate_unit(
        target: dict[str, Any],
    ) -> tuple[Any | None, dict[str, Any] | None, str | None]:
        objective = f"Write one prerequisite-aware learning unit for {target['name']}."
        unit_input = {
            **curriculum_input,
            "curriculum_stage": "units",
            "target_skills": [target],
            "knowledge_graph_skills": skill_names,
        }
        async with semaphore:
            for attempt in range(2):
                try:
                    execution, result = await agent_runtime.execute_agent_loop(
                        agent_id="curriculum",
                        objective=objective,
                        task_id=task.id,
                        run_id=state.run_id,
                        input_data=unit_input,
                    )
                    result_dict = result if isinstance(result, dict) else result.model_dump()
                    units = result_dict.get("learning_units", [])
                    if len(units) != 1:
                        raise ValueError(
                            f"Expected one learning unit for {target['name']}, got {len(units)}"
                        )
                    unit = units[0] if isinstance(units[0], dict) else units[0].model_dump()
                    if unit.get("skill") != target["name"]:
                        raise ValueError(
                            f"Generated learning unit for {unit.get('skill')!r}; "
                            f"expected {target['name']!r}"
                        )
                    return execution, unit, None
                except Exception as exc:
                    if attempt == 1:
                        return None, None, f"{target['name']}: {exc}"
                    logger.warning(f"Learning unit generation for {target['name']} retrying: {exc}")
        return None, None, f"{target['name']}: generation retries exhausted"

    unit_results = await asyncio.gather(*(generate_unit(target) for target in targets))
    learning_units = []
    unit_failures = []
    for execution, unit, error in unit_results:
        if execution is not None:
            task.execution_ids.append(execution.id)
        if unit is not None:
            learning_units.append(unit)
        if error is not None:
            unit_failures.append(error)

    curriculum = {
        "skills": skills,
        "relations": graph_dict.get("relations", []),
        "learning_units": learning_units,
        "summary": graph_dict.get("summary", ""),
    }
    complete = len(learning_units) >= CURRICULUM_MIN_UNITS
    task.output_data = {"curriculum": curriculum}
    task.status = "completed" if complete else "failed"
    if not complete:
        task.error = (
            f"Generated {len(learning_units)} of {CURRICULUM_MIN_UNITS} required learning units. "
            + "; ".join(unit_failures[:6])
        )
    await ledger.save_task(task)
    duration_ms = (time.perf_counter() - start_t) * 1000
    if complete:
        await ledger.append_event_transactional(
            run_id=state.run_id,
            event_type="task.completed",
            task_id=task.id,
            payload={
                "task_id": task.id,
                "duration_ms": duration_ms,
                "output_summary": (
                    f"{graph_dict.get('summary', '')} Generated "
                    f"{len(learning_units)} prerequisite-ordered learning units."
                ),
            },
        )
        return {"curriculum": curriculum, "completed_task_ids": [task.id]}

    await ledger.append_event_transactional(
        run_id=state.run_id,
        event_type="task.failed",
        task_id=task.id,
        payload=_task_failed_payload(task, task.error or "incomplete learning plan"),
    )
    return {
        "curriculum": curriculum,
        "failed_task_ids": [task.id],
        "missing_outputs": {"learning plan": task.error or "incomplete learning plan"},
    }
