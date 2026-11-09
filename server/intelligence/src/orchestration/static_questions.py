"""Question generation and validation nodes for the static workflow."""
import asyncio
import logging
import time
from typing import Any

from src.agents.runtime import agent_runtime
from src.contracts.v1 import WorkflowTask
from src.orchestration.static_support import (
    BATCHES_PER_CATEGORY,
    MIN_QUESTIONS_PER_CATEGORY,
    QUESTION_CATEGORIES,
    QUESTIONS_PER_BATCH,
    StaticNodeSupport,
    StaticWorkflowState,
)

logger = logging.getLogger("mensetsu.orchestration.static_workflow")


def build_question_nodes(support: StaticNodeSupport):
    ledger = support.ledger
    check_lease = support.check_lease
    set_phase = support.set_phase

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
            payload={
                "task_id": task.id,
                "attempt": first_attempt,
                "assigned_agent": task.assigned_agent,
            },
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
                        "target_topics": tech_top[part - 1 :: BATCHES_PER_CATEGORY][:3]
                        or tech_top[:3],
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
                            logger.warning(
                                f"Category {cat} batch {part} attempt {cat_att} error: {ce}"
                            )
                            if cat_att == 2:
                                raise
                    raise RuntimeError(
                        f"Failed to generate questions for category {cat} batch {part}"
                    )

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
            payload={
                "task_id": task.id,
                "attempt": first_attempt,
                "assigned_agent": task.assigned_agent,
            },
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

    return generate_questions_node, validate_questions_node, validate_condition
