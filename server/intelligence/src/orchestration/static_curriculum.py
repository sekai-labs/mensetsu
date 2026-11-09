"""Curriculum and knowledge-graph node for the static workflow."""
import asyncio
import logging
import time
from typing import Any

from src.agents.runtime import agent_runtime
from src.contracts.v1 import WorkflowTask
from src.orchestration.static_support import (
    CURRICULUM_MIN_UNITS,
    CURRICULUM_UNIT_BATCH_SIZE,
    StaticNodeSupport,
    StaticWorkflowState,
)

logger = logging.getLogger("mensetsu.orchestration.static_workflow")


def build_curriculum_nodes(support: StaticNodeSupport):
    ledger = support.ledger
    check_lease = support.check_lease
    set_phase = support.set_phase

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
            payload={
                "task_id": task.id,
                "attempt": first_attempt,
                "assigned_agent": task.assigned_agent,
            },
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
            batch_dict = (
                data
                if isinstance(data, dict)
                else (data.model_dump() if hasattr(data, "model_dump") else {})
            )
            return rec, batch_dict.get("learning_units", [])

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
                skills = graph_dict.get("skills", [])
                skill_names = [s.get("name", "") for s in skills if isinstance(s, dict)]
                unit_targets = [
                    s
                    for s in skills
                    if isinstance(s, dict) and s.get("candidate_status") != "verified"
                ]
                if len(unit_targets) < CURRICULUM_MIN_UNITS:
                    unit_targets += [
                        s for s in skills if isinstance(s, dict) and s not in unit_targets
                    ]
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
                        "output_summary": f"{graph_dict.get('summary', '')} Generated {len(learning_units)} learning units in {len(batches)} parallel batches.",
                    },
                )
                return {
                    "curriculum": {
                        "skills": skills,
                        "relations": graph_dict.get("relations", []),
                        "learning_units": learning_units,
                        "summary": graph_dict.get("summary", ""),
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

    return curriculum_node
