"""Company, job, and technical research nodes for the static workflow."""
import asyncio
import logging
import time
from typing import Any

from src.agents.runtime import agent_runtime
from src.contracts.v1 import WorkflowTask
from src.orchestration.static_support import StaticNodeSupport, StaticWorkflowState

logger = logging.getLogger("mensetsu.orchestration.static_workflow")


def build_research_nodes(support: StaticNodeSupport):
    ledger = support.ledger
    check_lease = support.check_lease
    completed_task_deliverable = support.completed_task_deliverable

    async def company_research_node(state: StaticWorkflowState) -> dict[str, Any]:
        check_lease()
        task_id = f"task_company_{state.run_id[:8]}"
        tasks = await ledger.list_tasks_for_run(state.run_id)
        matching_task = next((t for t in tasks if t.id == task_id), None)
        if matching_task and matching_task.status == "completed":
            comp_existing = state.research_results.get("company")
            if not comp_existing:
                comp_existing = await completed_task_deliverable(
                    state.run_id, task_id, "research_report"
                )
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
            payload={
                "task_id": task.id,
                "attempt": first_attempt,
                "assigned_agent": task.assigned_agent,
            },
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
            payload={
                "task_id": task.id,
                "attempt": first_attempt,
                "assigned_agent": task.assigned_agent,
            },
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
                tech_existing = await completed_task_deliverable(
                    state.run_id, task_id, "research_report"
                )
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
            payload={
                "task_id": task.id,
                "attempt": first_attempt,
                "assigned_agent": task.assigned_agent,
            },
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

    return company_research_node, job_analysis_node, technical_research_node
