import asyncio
import collections
import logging
import time
from typing import Annotated, Any

from langgraph.graph import END, START, StateGraph
from langgraph.types import Send
from pydantic import BaseModel, Field

from src.agents.registry import registry
from src.agents.runtime import agent_runtime
from src.contracts.v1 import WorkflowTask
from src.orchestration.id_validator import ID_PATTERN
from src.orchestration.ledger import SurrealLedgerRepository

logger = logging.getLogger("mensetsu.orchestration.dynamic_workflow")

MAX_PLAN_TASKS = 15
MAX_RESEARCH_TASKS_PER_REVISION = 4
MAX_PLAN_DEPTH = 4
MAX_PLAN_TOOL_CALLS = 240
TASK_ATTEMPTS = 2
PLANNABLE_AGENTS: dict[str, str] = {
    "company-research": "company",
    "job-analysis": "job",
    "technical-research": "tech",
}


def merge_completed(a: list[str], b: list[str]) -> list[str]:
    return list(dict.fromkeys((a or []) + (b or [])))


def merge_results(a: dict[str, Any], b: dict[str, Any]) -> dict[str, Any]:
    return {**(a or {}), **(b or {})}


class DynamicWorkflowState(BaseModel):
    run_id: str
    workspace_id: str
    company_name: str
    role_title: str
    company_urls: list[str] = Field(default_factory=list)
    resume_text: str = ""
    resume_pdf_base64: str | None = None
    resume_filename: str | None = None
    job_description: str = ""
    jd_url: str | None = None
    seniority: str | None = None
    parsed_resume: dict[str, Any] = Field(default_factory=dict)
    jd_evidence_id: str | None = None
    plan_revision: int = 1
    revision_evaluated: bool = False
    tasks: list[dict[str, Any]] = Field(default_factory=list)
    completed_task_ids: Annotated[list[str], merge_completed] = Field(default_factory=list)
    research_results: Annotated[dict[str, Any], merge_results] = Field(default_factory=dict)
    failed_task_ids: Annotated[list[str], merge_completed] = Field(default_factory=list)
    missing_research: Annotated[dict[str, str], merge_results] = Field(default_factory=dict)


def validate_plan_dag(tasks: list[dict[str, Any]]) -> None:
    if len(tasks) > MAX_PLAN_TASKS:
        raise ValueError(f"Dynamic plan exceeds maximum task count of {MAX_PLAN_TASKS}.")

    task_ids: set[str] = set()
    graph: dict[str, list[str]] = collections.defaultdict(list)
    in_degree: dict[str, int] = collections.defaultdict(int)

    for t in tasks:
        tid = t["id"]
        if tid in task_ids:
            raise ValueError(f"Duplicate task ID '{tid}' in plan.")
        task_ids.add(tid)
        if not ID_PATTERN.match(tid):
            raise ValueError(f"Task ID '{tid}' must match ^[A-Za-z0-9_\\-]+$.")

        agent_id = t["assigned_agent"]
        try:
            registry.get(agent_id)
        except KeyError:
            raise ValueError(f"Unknown agent '{agent_id}' in planned task '{tid}'.")

    for t in tasks:
        tid = t["id"]
        deps = t.get("dependencies", [])
        if len(set(deps)) != len(deps):
            raise ValueError(f"Task '{tid}' lists a dependency more than once.")
        for d in deps:
            if d not in task_ids:
                raise ValueError(f"Task '{tid}' references nonexistent dependency '{d}'.")
            graph[d].append(tid)
            in_degree[tid] += 1

    queue = [t["id"] for t in tasks if in_degree[t["id"]] == 0]
    depth_map = {tid: 1 for tid in queue}
    visited = 0

    while queue:
        curr = queue.pop(0)
        visited += 1
        curr_depth = depth_map[curr]
        if curr_depth > MAX_PLAN_DEPTH:
            raise ValueError(
                f"Plan exceeds maximum depth of {MAX_PLAN_DEPTH} (task '{curr}' has depth {curr_depth})."
            )
        for neighbor in graph[curr]:
            in_degree[neighbor] -= 1
            depth_map[neighbor] = max(depth_map.get(neighbor, 1), curr_depth + 1)
            if in_degree[neighbor] == 0:
                queue.append(neighbor)

    if visited != len(tasks):
        raise ValueError("Cycle detected in planned task DAG.")

    for t in tasks:
        if t["assigned_agent"] not in PLANNABLE_AGENTS:
            raise ValueError(
                f"Agent '{t['assigned_agent']}' in task '{t['id']}' is not permitted in dynamic plans. "
                f"Plannable agents: {sorted(PLANNABLE_AGENTS)}. Synthesis, curriculum and questions run after the plan."
            )
        if not str(t.get("name", "")).strip():
            raise ValueError(f"Task '{t['id']}' has an empty name.")
        if t["assigned_agent"] == "job-analysis" and not t["required"]:
            raise ValueError(f"Task '{t['id']}' runs job-analysis and must be required.")

    research = [
        t for t in tasks if t["assigned_agent"] in ("company-research", "technical-research")
    ]
    if len(research) > MAX_RESEARCH_TASKS_PER_REVISION:
        raise ValueError(
            f"Plan proposes {len(research)} research tasks; maximum permitted per revision is {MAX_RESEARCH_TASKS_PER_REVISION}."
        )
    if research and not any(t["required"] for t in research):
        raise ValueError(
            "At least one company-research or technical-research task must be required."
        )
    planned_tool_calls = sum(registry.get(t["assigned_agent"]).tool_calls_budget for t in tasks)
    if planned_tool_calls > MAX_PLAN_TOOL_CALLS:
        raise ValueError(
            f"Plan requests {planned_tool_calls} tool calls across tasks; the dynamic budget is {MAX_PLAN_TOOL_CALLS}."
        )


def build_dynamic_workflow(checkpointer: Any, ledger: SurrealLedgerRepository | None = None):
    if checkpointer is None:
        raise ValueError("Checkpointer is required to compile dynamic workflow.")

    ledger = ledger or SurrealLedgerRepository()

    def check_lease() -> None:
        from src.agents.runtime import current_lost_lease_var

        lost_ev = current_lost_lease_var.get()
        if lost_ev and lost_ev.is_set():
            raise RuntimeError("Lease lost: cooperative abort at node boundary.")

    async def set_run_phase(run_id: str, phase: str, extra: dict[str, Any] | None = None) -> None:
        await ledger.update_workflow_run(run_id, {"current_phase": phase, **(extra or {})})
        await ledger.refresh_run_progress(run_id)

    def agent_input(state: dict[str, Any], task_info: dict[str, Any]) -> dict[str, Any]:
        agent_id = task_info["assigned_agent"]
        base = {"task_name": task_info["name"]}
        if agent_id == "company-research":
            return {**base, "company": state["company_name"], "urls": state["company_urls"]}
        if agent_id == "job-analysis":
            job_input = {
                **base,
                "role": state["role_title"],
                "job_description": state["job_description"],
            }
            if state.get("jd_evidence_id"):
                job_input["job_description_source"] = {
                    "url": state["jd_url"],
                    "evidence_id": state["jd_evidence_id"],
                }
            return job_input
        return {**base, "company": state["company_name"], "role": state["role_title"]}

    async def planner_node(state: DynamicWorkflowState) -> dict[str, Any]:
        check_lease()
        existing_plan = await ledger.get_task(state.run_id, f"task_plan_{state.run_id[:8]}")
        if existing_plan is not None and existing_plan.status == "completed":
            restored = existing_plan.output_data.get("planned_tasks")
            if isinstance(restored, list) and restored:
                return {"tasks": restored}
        await set_run_phase(state.run_id, "planning")
        start_t = time.perf_counter()
        plan_task = WorkflowTask(
            id=f"task_plan_{state.run_id[:8]}",
            run_id=state.run_id,
            task_type="planning",
            name="Plan research DAG",
            assigned_agent="planner",
            status="running",
        )
        first_attempt = await ledger.next_task_attempt(plan_task.run_id, plan_task.id)
        plan_task.attempt = first_attempt
        await ledger.save_task(plan_task)
        await ledger.append_event_transactional(
            run_id=state.run_id,
            event_type="task.started",
            task_id=plan_task.id,
            payload={
                "task_id": plan_task.id,
                "attempt": first_attempt,
                "assigned_agent": "planner",
            },
        )

        planner_input = {
            "company_name": state.company_name,
            "role_title": state.role_title,
            "company_urls": state.company_urls,
            "job_description": state.job_description[:4000],
            "seniority": state.seniority,
            "plannable_agents": {
                agent_id: registry.get(agent_id).description for agent_id in PLANNABLE_AGENTS
            },
            "limits": {
                "max_tasks": MAX_PLAN_TASKS,
                "max_research_tasks": MAX_RESEARCH_TASKS_PER_REVISION,
                "max_depth": MAX_PLAN_DEPTH,
                "max_total_tool_calls": MAX_PLAN_TOOL_CALLS,
                "tool_calls_per_agent": {
                    agent_id: registry.get(agent_id).tool_calls_budget
                    for agent_id in PLANNABLE_AGENTS
                },
            },
            "plan_revision": state.plan_revision,
        }
        last_error: Exception | None = None
        for attempt in range(first_attempt, first_attempt - 1 + TASK_ATTEMPTS + 1):
            plan_task.attempt = attempt
            await ledger.save_task(plan_task)
            try:
                if last_error is not None:
                    planner_input["previous_plan_rejected"] = str(last_error)
                exec_rec, plan_data = await agent_runtime.execute_agent_loop(
                    agent_id="planner",
                    objective=f"Plan the research DAG for {state.role_title} at {state.company_name}",
                    task_id=plan_task.id,
                    run_id=state.run_id,
                    input_data=planner_input,
                )
                raw_tasks = plan_data["tasks"] if isinstance(plan_data, dict) else plan_data.tasks
                planned = [t if isinstance(t, dict) else t.model_dump() for t in raw_tasks]
                validate_plan_dag(planned)
                break
            except ValueError as ve:
                last_error = ve
                logger.warning(f"Plan attempt {attempt} rejected for {state.run_id}: {ve}")
                if attempt == first_attempt - 1 + TASK_ATTEMPTS:
                    plan_task.status = "failed"
                    plan_task.error = str(ve)
                    await ledger.save_task(plan_task)
                    await ledger.append_event_transactional(
                        run_id=state.run_id,
                        event_type="task.failed",
                        task_id=plan_task.id,
                        payload={
                            "task_id": plan_task.id,
                            "attempt": attempt,
                            "error": str(ve),
                            "can_retry": False,
                        },
                    )
                    raise

        plan_task.status = "completed"
        plan_task.execution_ids.append(exec_rec.id)
        plan_task.output_data = {"task_ids": [t["id"] for t in planned], "planned_tasks": planned}
        await ledger.save_task(plan_task)
        await ledger.append_event_transactional(
            run_id=state.run_id,
            event_type="task.completed",
            task_id=plan_task.id,
            payload={
                "task_id": plan_task.id,
                "duration_ms": (time.perf_counter() - start_t) * 1000,
                "output_summary": exec_rec.public_summary,
            },
        )
        for t in planned:
            await ledger.save_task(
                WorkflowTask(
                    id=t["id"],
                    run_id=state.run_id,
                    task_type="dynamic",
                    name=t["name"],
                    assigned_agent=t["assigned_agent"],
                    dependencies=t.get("dependencies", []),
                    required=t["required"],
                    added_in_revision=state.plan_revision,
                    attempt=0,
                    status="pending",
                )
            )
        await ledger.refresh_run_progress(state.run_id)
        await ledger.append_event_transactional(
            run_id=state.run_id,
            event_type="plan.created",
            payload={"revision": state.plan_revision, "tasks": planned},
        )
        await set_run_phase(state.run_id, "executing_plan")
        return {"tasks": planned}

    async def join_node(state: DynamicWorkflowState) -> dict[str, Any]:
        return {}

    def route_from_join(state: DynamicWorkflowState):
        completed = set(state.completed_task_ids)
        failed = set(state.failed_task_ids)
        finished = completed | failed
        blocked = [
            t
            for t in state.tasks
            if t["id"] not in finished and any(d in failed for d in t.get("dependencies", []))
        ]
        finished |= {t["id"] for t in blocked}
        remaining = [t for t in state.tasks if t["id"] not in finished]
        roots_done = all(t["id"] in finished for t in state.tasks if not t.get("dependencies"))
        if not state.revision_evaluated and roots_done:
            return "revise"
        if not remaining:
            return "outputs"
        ready = [t for t in remaining if all(d in completed for d in t.get("dependencies", []))]
        if not ready:
            raise RuntimeError(
                f"Dynamic plan for {state.run_id} stalled: {[t['id'] for t in remaining]} have unmet dependencies."
            )
        payload = state.model_dump(
            include={
                "run_id",
                "company_name",
                "role_title",
                "company_urls",
                "job_description",
                "jd_url",
                "jd_evidence_id",
            }
        )
        return [Send("execute_dynamic_task", {**payload, "task": t}) for t in ready]

    async def execute_task_node(arg: dict[str, Any]) -> dict[str, Any]:
        check_lease()
        task_info = arg["task"]
        run_id = arg["run_id"]
        result_key = PLANNABLE_AGENTS[task_info["assigned_agent"]]
        existing = await ledger.get_task(run_id, task_info["id"])
        if existing is None:
            raise RuntimeError(
                f"Dynamic task {task_info['id']} was dispatched for {run_id} without a registered task record."
            )
        if existing and existing.status == "completed" and existing.output_data.get("result"):
            return {
                "completed_task_ids": [task_info["id"]],
                "research_results": {result_key: existing.output_data["result"]},
            }

        start_t = time.perf_counter()
        task = WorkflowTask(
            id=task_info["id"],
            run_id=run_id,
            task_type="dynamic",
            name=task_info["name"],
            assigned_agent=task_info["assigned_agent"],
            dependencies=task_info.get("dependencies", []),
            required=task_info["required"],
            added_in_revision=existing.added_in_revision,
            status="running",
        )
        first_attempt = await ledger.next_task_attempt(task.run_id, task.id)
        task.attempt = first_attempt
        await ledger.save_task(task)
        await ledger.append_event_transactional(
            run_id=run_id,
            event_type="task.started",
            task_id=task.id,
            payload={
                "task_id": task.id,
                "attempt": first_attempt,
                "assigned_agent": task.assigned_agent,
            },
        )

        for attempt in range(first_attempt, first_attempt - 1 + TASK_ATTEMPTS + 1):
            task.attempt = attempt
            await ledger.save_task(task)
            try:
                exec_rec, structured = await agent_runtime.execute_agent_loop(
                    agent_id=task.assigned_agent,
                    objective=f"{task.name} ({arg['role_title']} at {arg['company_name']})",
                    task_id=task.id,
                    run_id=run_id,
                    input_data=agent_input(arg, task_info),
                )
                break
            except Exception as e:
                logger.warning(f"Dynamic task {task.id} attempt {attempt} failed: {e}")
                if attempt < first_attempt - 1 + TASK_ATTEMPTS:
                    delay_ms = 1000 * (attempt - first_attempt + 1)
                    await ledger.append_event_transactional(
                        run_id=run_id,
                        event_type="task.retrying",
                        task_id=task.id,
                        payload={
                            "task_id": task.id,
                            "next_attempt": attempt + 1,
                            "delay_ms": delay_ms,
                        },
                    )
                    await asyncio.sleep(delay_ms / 1000.0)
                    continue
                task.status = "failed"
                task.error = str(e) or e.__class__.__name__
                task.required = task_info["required"]
                await ledger.save_task(task)
                await ledger.append_event_transactional(
                    run_id=run_id,
                    event_type="task.failed",
                    task_id=task.id,
                    payload={
                        "task_id": task.id,
                        "attempt": attempt,
                        "error": task.error,
                        "can_retry": False,
                    },
                )
                if task_info["required"]:
                    raise
                return {
                    "failed_task_ids": [task.id],
                    "missing_research": {result_key: f"{task.id}: {task.error}"},
                }

        result = structured if isinstance(structured, dict) else structured.model_dump()
        task.status = "completed"
        task.execution_ids.append(exec_rec.id)
        task.output_data = {"result": result}
        await ledger.save_task(task)
        await ledger.append_event_transactional(
            run_id=run_id,
            event_type="task.completed",
            task_id=task.id,
            payload={
                "task_id": task.id,
                "duration_ms": (time.perf_counter() - start_t) * 1000,
                "output_summary": exec_rec.public_summary,
            },
        )
        return {"completed_task_ids": [task.id], "research_results": {result_key: result}}

    async def revise_node(state: DynamicWorkflowState) -> dict[str, Any]:
        check_lease()
        completed = set(state.completed_task_ids)
        failed = set(state.failed_task_ids)
        completed_agents = {t["assigned_agent"] for t in state.tasks if t["id"] in completed}
        pending = [t for t in state.tasks if t["id"] not in completed and t["id"] not in failed]
        depended_on = {d for t in pending for d in t.get("dependencies", [])}
        removed = [
            t
            for t in pending
            if t["assigned_agent"] in completed_agents
            and t["id"] not in depended_on
            and not t["required"]
        ]
        removed_ids = {t["id"] for t in removed}
        kept = [t for t in state.tasks if t["id"] not in removed_ids]
        covered = {t["assigned_agent"] for t in kept}
        next_revision = state.plan_revision + 1
        added = [
            {
                "id": f"rev{next_revision}_{agent_id.replace('-', '_')}_{state.run_id[:8]}",
                "name": f"{registry.get(agent_id).name}: {state.company_name}",
                "assigned_agent": agent_id,
                "dependencies": [],
                "required": agent_id == "job-analysis",
            }
            for agent_id in PLANNABLE_AGENTS
            if agent_id not in covered
        ]
        revised = kept + added
        try:
            validate_plan_dag(revised)
        except ValueError as ve:
            logger.warning(
                f"Plan revision for {state.run_id} rejected; keeping revision {state.plan_revision} unchanged: {ve}"
            )
            await ledger.append_execution_log(
                run_id=state.run_id,
                message=f"Plan revision skipped: candidate plan invalid ({ve}); original plan kept.",
                severity="warn",
                component="plan",
            )
            await set_run_phase(state.run_id, "plan_final")
            return {"revision_evaluated": True}

        if added or removed:
            reasons = []
            if added:
                reasons.append(f"added missing coverage for {[t['assigned_agent'] for t in added]}")
            if removed:
                reasons.append(
                    f"dropped duplicate tasks for agents already completed {sorted(removed_ids)}"
                )
            for t in removed:
                task = await ledger.get_task(state.run_id, t["id"])
                if task:
                    task.status = "skipped"
                    await ledger.save_task(task)
            for t in added:
                await ledger.save_task(
                    WorkflowTask(
                        id=t["id"],
                        run_id=state.run_id,
                        task_type="dynamic",
                        name=t["name"],
                        assigned_agent=t["assigned_agent"],
                        dependencies=t["dependencies"],
                        required=t["required"],
                        added_in_revision=next_revision,
                        attempt=0,
                        status="pending",
                    )
                )
            await ledger.update_workflow_run(state.run_id, {"plan_revision": next_revision})
            await ledger.append_event_transactional(
                run_id=state.run_id,
                event_type="plan.revised",
                payload={
                    "revision": next_revision,
                    "added_tasks": [t["id"] for t in added],
                    "removed_tasks": sorted(removed_ids),
                    "reason": "; ".join(reasons),
                },
            )
            for t in removed:
                await ledger.append_event_transactional(
                    run_id=state.run_id,
                    event_type="task.removed",
                    task_id=t["id"],
                    payload={
                        "task_id": t["id"],
                        "reason": f"{t['assigned_agent']} already completed in this run",
                    },
                )
            for t in added:
                await ledger.append_event_transactional(
                    run_id=state.run_id,
                    event_type="task.added",
                    task_id=t["id"],
                    payload={
                        "task_id": t["id"],
                        "name": t["name"],
                        "assigned_agent": t["assigned_agent"],
                        "dependencies": t["dependencies"],
                        "added_in_revision": next_revision,
                    },
                )
            await set_run_phase(state.run_id, "plan_final", {"plan_revision": next_revision})
            return {"tasks": revised, "plan_revision": next_revision, "revision_evaluated": True}
        await set_run_phase(state.run_id, "plan_final")
        return {"revision_evaluated": True}

    async def outputs_node(state: DynamicWorkflowState) -> dict[str, Any]:
        check_lease()
        from src.orchestration.static_workflow import StaticWorkflowState, build_static_workflow

        if "job" not in state.research_results:
            raise RuntimeError(
                f"Dynamic run {state.run_id} reached outputs without job analysis results."
            )
        if "company" not in state.research_results and "tech" not in state.research_results:
            raise RuntimeError(
                f"Dynamic run {state.run_id} has neither company nor technical research results."
            )
        output_graph = build_static_workflow(
            checkpointer=checkpointer,
            ledger=ledger,
            db_client=ledger.db,
            mode="outputs_only",
        )
        await output_graph.ainvoke(
            StaticWorkflowState(
                run_id=state.run_id,
                workspace_id=state.workspace_id,
                company_name=state.company_name,
                role_title=state.role_title,
                company_urls=state.company_urls,
                resume_text=state.resume_text,
                job_description=state.job_description,
                seniority=state.seniority,
                parsed_resume=state.parsed_resume,
                research_results=state.research_results,
                missing_research=state.missing_research,
                upstream_task_ids=list(state.completed_task_ids),
            ),
            config={"configurable": {"thread_id": f"{state.run_id}__outputs"}},
        )
        return {}

    async def normalize_node(state: DynamicWorkflowState) -> dict[str, Any]:
        check_lease()
        from src.orchestration.static_workflow import StaticWorkflowState, build_static_workflow

        normalize_graph = build_static_workflow(
            checkpointer=checkpointer, ledger=ledger, db_client=ledger.db, mode="normalize_only"
        )
        normalized = await normalize_graph.ainvoke(
            StaticWorkflowState(
                run_id=state.run_id,
                workspace_id=state.workspace_id,
                company_name=state.company_name,
                role_title=state.role_title,
                company_urls=state.company_urls,
                resume_text=state.resume_text,
                resume_pdf_base64=state.resume_pdf_base64,
                resume_filename=state.resume_filename,
                job_description=state.job_description,
                jd_url=state.jd_url,
                seniority=state.seniority,
            ),
            config={"configurable": {"thread_id": f"{state.run_id}__normalize"}},
        )
        jd_evidence_id = (
            f"art_jd_{state.run_id[:8]}"
            if state.jd_url and not state.job_description.strip()
            else None
        )
        return {
            "resume_text": normalized["resume_text"],
            "job_description": normalized["job_description"],
            "parsed_resume": normalized["parsed_resume"],
            "jd_evidence_id": jd_evidence_id,
            "resume_pdf_base64": None,
        }

    builder = StateGraph(DynamicWorkflowState)
    builder.add_node("normalize", normalize_node)
    builder.add_node("plan", planner_node)
    builder.add_node("join", join_node)
    builder.add_node("execute_dynamic_task", execute_task_node)
    builder.add_node("revise", revise_node)
    builder.add_node("outputs", outputs_node)

    builder.add_edge(START, "normalize")
    builder.add_edge("normalize", "plan")
    builder.add_edge("plan", "join")
    builder.add_edge("execute_dynamic_task", "join")
    builder.add_edge("revise", "join")
    builder.add_conditional_edges(
        "join", route_from_join, ["execute_dynamic_task", "revise", "outputs"]
    )
    builder.add_edge("outputs", END)

    return builder.compile(checkpointer=checkpointer)
