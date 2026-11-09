"""Routing, planning, and per-agent execution nodes for the hybrid workflow."""
import logging
import time
from typing import Any

from langgraph.types import Send

from src.agents.runtime import agent_runtime
from src.agents.workflow_router import route_workflow_request
from src.contracts.v1 import Artifact, PlanTaskItem, WorkflowProgress, WorkflowTask
from src.orchestration.dynamic_workflow import PLANNABLE_AGENTS, validate_plan_dag
from src.orchestration.hybrid_support import (
    HybridWorkflowState,
    _task_failed_payload,
    check_lease,
)
from src.orchestration.ledger import SurrealLedgerRepository

logger = logging.getLogger("mensetsu.orchestration.hybrid_workflow")


def build_hybrid_nodes(checkpointer, ledger: SurrealLedgerRepository):
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

    return {
        "set_run_phase": set_run_phase,
        "route_and_normalize_node": route_and_normalize_node,
        "planner_node": planner_node,
        "dynamic_router": dynamic_router,
        "execute_dynamic_task_node": execute_dynamic_task_node,
        "synthesize_node": synthesize_node,
    }
