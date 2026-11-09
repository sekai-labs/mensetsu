"""Shared state, routing helpers, and persistence for the hybrid workflow."""
import logging
from typing import Annotated, Any

from pydantic import BaseModel, Field

from src.contracts.v1 import Artifact, TaskFailedPayload, WorkflowTask
from src.orchestration.dynamic_workflow import merge_completed, merge_results
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
