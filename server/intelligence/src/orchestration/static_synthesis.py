"""Match-analysis synthesis node for the static workflow."""
import asyncio
import logging
import time
from typing import Any

from src.agents.runtime import agent_runtime
from src.contracts.v1 import WorkflowTask
from src.orchestration.static_support import StaticNodeSupport, StaticWorkflowState

logger = logging.getLogger("mensetsu.orchestration.static_workflow")


def build_synthesis_nodes(support: StaticNodeSupport):
    ledger = support.ledger
    check_lease = support.check_lease
    set_phase = support.set_phase

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
            payload={
                "task_id": task.id,
                "attempt": first_attempt,
                "assigned_agent": task.assigned_agent,
            },
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
                {"area": key, "reason": state.missing_research.get(key, "not run")}
                for key in missing
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

    return synthesize_node
