"""Curriculum and learning-unit generation for the hybrid workflow."""
import asyncio
import logging
import time
from typing import Any

from src.agents.runtime import agent_runtime
from src.contracts.v1 import WorkflowTask
from src.orchestration.hybrid_support import (
    CURRICULUM_MIN_UNITS,
    CURRICULUM_UNIT_CONCURRENCY,
    HybridWorkflowState,
    _task_failed_payload,
    check_lease,
)
from src.orchestration.ledger import SurrealLedgerRepository

logger = logging.getLogger("mensetsu.orchestration.hybrid_workflow")


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
