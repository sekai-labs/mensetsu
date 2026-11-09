"""Persistence and finalization nodes for the static workflow."""
import logging
import time
from datetime import datetime, timezone
from typing import Any
from urllib.parse import urlparse

from src.contracts.v1 import WorkflowTask
from src.generation.persistence import persist_record
from src.orchestration.id_validator import validate_id
from src.orchestration.static_support import (
    TOTAL_MIN_QUESTIONS,
    StaticNodeSupport,
    StaticWorkflowState,
)

logger = logging.getLogger("mensetsu.orchestration.static_workflow")


def build_output_nodes(support: StaticNodeSupport, generate_questions_node):
    ledger = support.ledger
    db = support.db
    check_lease = support.check_lease

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
            payload={
                "task_id": task.id,
                "attempt": first_attempt,
                "assigned_agent": task.assigned_agent,
            },
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
            linked = [
                source_by_evidence[eid]
                for eid in c_dict["evidence_ids"]
                if eid in source_by_evidence
            ]
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

    return persist_outputs_node
