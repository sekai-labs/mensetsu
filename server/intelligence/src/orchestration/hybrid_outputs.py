"""Curriculum hand-off, question generation, and finalization nodes for the hybrid workflow."""
import asyncio
import logging
import time
from datetime import datetime, timezone
from typing import Any

from src.agents.runtime import agent_runtime
from src.agents.runtime_support import AgentRecoveryContext, build_agent_recovery_context
from src.agents.schemas import (
    current_knowledge_graph_skill_names_var,
    validate_question_skill_names,
)
from src.contracts.v1 import WorkflowProgress, WorkflowTask
from src.orchestration.hybrid_curriculum import execute_curriculum_node
from src.orchestration.hybrid_support import (
    QUESTION_BATCHES_PER_CATEGORY,
    QUESTION_CATEGORIES,
    QUESTION_GENERATION_CONCURRENCY,
    QUESTIONS_PER_BATCH,
    HybridWorkflowState,
    _persist_hybrid_outputs,
    _question_batch_plan,
    _task_failed_payload,
    _validate_hybrid_run_outputs,
    check_lease,
)
from src.orchestration.ledger import SurrealLedgerRepository

logger = logging.getLogger("mensetsu.orchestration.hybrid_workflow")


def build_hybrid_output_nodes(ledger: SurrealLedgerRepository, set_run_phase):
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
        first_attempt = await ledger.next_task_attempt(state.run_id, task.id)
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
        knowledge_graph_skills = [
            skill["name"]
            for skill in state.curriculum.get("skills", [])
            if isinstance(skill, dict) and skill.get("name")
        ]
        input_base = {
            "company_name": state.company_name,
            "role_title": state.role_title,
            "target_topics": topics[:5],
            "candidate_gaps": state.match_analysis.get("gaps", [])[:4],
            "culture_signals": company.get("engineering_culture", [])[:3],
            "candidate_background": candidate_background,
            "knowledge_graph_skills": knowledge_graph_skills,
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
                batch_recovery_context: AgentRecoveryContext | None = None
                for attempt in range(1, 3):
                    try:
                        skill_names_token = current_knowledge_graph_skill_names_var.set(
                            frozenset(knowledge_graph_skills)
                        )
                        try:
                            execution, result = await agent_runtime.execute_agent_loop(
                                agent_id="interview-generation",
                                objective=objective,
                                task_id=task.id,
                                run_id=state.run_id,
                                input_data=batch_input,
                                recovery_context=batch_recovery_context,
                            )
                        finally:
                            current_knowledge_graph_skill_names_var.reset(skill_names_token)
                        result_dict = result if isinstance(result, dict) else result.model_dump()
                        questions = result_dict.get("questions", [])
                        for question in questions:
                            question_data = (
                                question
                                if isinstance(question, dict)
                                else question.model_dump()
                            )
                            validate_question_skill_names(
                                question_data.get("skills_tested", []),
                                frozenset(knowledge_graph_skills),
                            )
                        if len(questions) != count:
                            raise ValueError(
                                f"{category} batch {part} returned {len(questions)} questions; expected {count}"
                            )
                        return category, part, execution, questions, None
                    except Exception as exc:
                        if attempt == 2:
                            return category, part, None, [], str(exc)
                        batch_recovery_context = build_agent_recovery_context(
                            "interview-generation", exc, attempt
                        )
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
        check_lease()
        now_completed = datetime.now(timezone.utc)
        all_tasks = await ledger.list_tasks_for_run(state.run_id)
        completed_tasks = [t for t in all_tasks if t.status == "completed"]
        total_tasks_count = max(len(all_tasks), len(completed_tasks), 6)

        run_rec = await ledger.get_workflow_run(state.run_id)
        if run_rec is not None and run_rec.status == "cancelled":
            logger.info(f"Run {state.run_id} was cancelled before finalization; skipping completion.")
            return {}
        started_at = run_rec.started_at if run_rec else None
        calc_duration_ms = (
            (now_completed - started_at).total_seconds() * 1000 if started_at else 0.0
        )
        all_artifacts = await ledger.list_artifacts_for_run(state.run_id)
        resolved_outputs = _validate_hybrid_run_outputs(state, all_artifacts)
        await _persist_hybrid_outputs(state, ledger, all_artifacts, resolved_outputs)
        degraded_reasons = [
            f"{area} research failed: {reason}"
            for area, reason in sorted(state.missing_research.items())
        ] + [
            f"{area} generation failed: {reason}"
            for area, reason in sorted(state.missing_outputs.items())
        ]
        completed = await ledger.transition_workflow_run_status(
            state.run_id,
            "completed",
            from_statuses=("pending", "running"),
            updates={
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
        if not completed:
            logger.info(
                f"Run {state.run_id} is no longer active; skipping completion finalization."
            )
            return {}
        await ledger.sweep_run_to_terminal(state.run_id, reason="run completed")
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

    return {
        "curriculum_node": curriculum_node,
        "interview_node": interview_node,
        "finalize_node": finalize_node,
    }
