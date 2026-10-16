import logging
import uuid
from typing import Any

from src.contracts.v1 import (
    AgentExecution,
    Artifact,
    Delegation,
    DelegationGraphResponse,
    EventsHistoryPageResponse,
    LogsPageResponse,
    RunsPageResponse,
    TaskGraphResponse,
    ToolInvocationsPageResponse,
    ToolOutputResponse,
    WorkflowDefinition,
    WorkflowRun,
    WorkflowTask,
)
from src.orchestration.shared import (
    DEFINITIONS,
    get_ledger,
    job_queue,
)

logger = logging.getLogger("mensetsu.orchestration.service")


class WorkflowService:
    @staticmethod
    async def list_definitions() -> list[WorkflowDefinition]:
        return DEFINITIONS

    @staticmethod
    async def create_run(
        workspace_id: str,
        user_id: str,
        workflow_id: str = "interview-preparation-hybrid",
        company_name: str = "Target Company",
        role_title: str = "Target Role",
        company_urls: list[str] | None = None,
        resume_text: str = "",
        resume_pdf_base64: str | None = None,
        resume_filename: str | None = None,
        job_description: str = "",
        jd_url: str | None = None,
        seniority: str | None = None,
        user_prompt: str | None = None,
        workflow_mode: str | None = None,
        llm_config: dict[str, Any] | None = None,
    ) -> WorkflowRun:
        wf_def = next((d for d in DEFINITIONS if d.id == workflow_id), None)
        if wf_def is None:
            raise ValueError(
                f"Unknown workflow_id '{workflow_id}'. Known: {[d.id for d in DEFINITIONS]}"
            )

        run_id = f"run_{uuid.uuid4().hex}"
        run = WorkflowRun(
            id=run_id,
            workflow_id=wf_def.id,
            workflow_name=wf_def.name,
            workspace_id=workspace_id,
            user_id=user_id,
            type=wf_def.type,
            status="pending",
            current_phase="queued",
            plan_revision=1,
            last_seq=0,
        )

        active_ledger = get_ledger()
        await active_ledger.create_workflow_run(run)

        await active_ledger.append_event_transactional(
            run_id=run.id,
            event_type="workflow.created",
            payload={
                "workflow_id": run.workflow_id,
                "workspace_id": run.workspace_id,
                "user_id": run.user_id,
                "type": run.type,
            },
        )

        await job_queue.enqueue_job(
            run_id=run.id,
            payload={
                "run_id": run.id,
                "workflow_id": run.workflow_id,
                "workspace_id": workspace_id,
                "user_id": user_id,
                "company_name": company_name,
                "role_title": role_title,
                "company_urls": company_urls or [],
                "resume_text": resume_text,
                "resume_pdf_base64": resume_pdf_base64,
                "resume_filename": resume_filename,
                "job_description": job_description,
                "jd_url": jd_url,
                "seniority": seniority,
                "user_prompt": user_prompt,
                "workflow_mode": workflow_mode,
                "llm_config": llm_config,
            },
        )
        run.last_seq = 1
        return run

    @staticmethod
    async def resume_plan(run_id: str, prompt: str) -> dict[str, Any]:
        active_ledger = get_ledger()
        run = await active_ledger.get_workflow_run(run_id)
        if not run:
            raise KeyError(f"Workflow run '{run_id}' not found")

        new_revision = run.plan_revision + 1
        await active_ledger.update_workflow_run(run_id, {"plan_revision": new_revision})
        await active_ledger.append_event_transactional(
            run_id=run_id,
            event_type="plan.revised",
            payload={"run_id": run_id, "prompt": prompt, "plan_revision": new_revision},
        )

        await job_queue.enqueue_job(
            run_id=run.id,
            payload={
                "run_id": run.id,
                "workflow_id": run.workflow_id,
                "workspace_id": run.workspace_id,
                "user_id": run.user_id,
                "company_name": "Target Company",
                "role_title": "Target Role",
                "user_prompt": prompt,
                "plan_revision": new_revision,
                "resume_plan": True,
            },
        )

        return {"status": "plan_resumed", "run_id": run_id, "plan_revision": new_revision}

    @staticmethod
    async def list_runs(
        run_ids: list[str] | None = None,
        workspace_id: str | None = None,
        status: str | None = None,
        run_type: str | None = None,
        limit: int = 50,
        cursor: str | None = None,
    ) -> RunsPageResponse:
        active_ledger = get_ledger()
        if run_ids:
            if len(run_ids) > 100:
                raise ValueError("Maximum 100 run_ids allowed.")
            runs = []
            for r_id in run_ids:
                run = await active_ledger.get_workflow_run(r_id)
                if run:
                    runs.append(run)
            return RunsPageResponse(items=runs, next_cursor=None)

        return await active_ledger.list_workflow_runs(
            workspace_id=workspace_id,
            status=status,
            run_type=run_type,
            limit=limit,
            cursor=cursor,
        )

    @staticmethod
    async def get_run(run_id: str) -> WorkflowRun | None:
        active_ledger = get_ledger()
        return await active_ledger.get_workflow_run(run_id)

    @staticmethod
    async def cancel_run(run_id: str) -> WorkflowRun:
        active_ledger = get_ledger()
        run = await active_ledger.get_workflow_run(run_id)
        if not run:
            raise KeyError("Run not found")
        if run.status != "cancelled":
            run.status = "cancelled"
            await active_ledger.update_workflow_run(run_id, {"status": "cancelled"})
            try:
                await active_ledger.sweep_run_to_terminal(run_id, reason="run cancelled")
            except Exception as se:
                logger.error(f"Run-end sweep failed for cancelled run {run_id}: {se}")
            await active_ledger.append_event_transactional(
                run_id=run_id,
                event_type="workflow.cancelled",
                payload={"reason": "User cancellation requested", "cancelled_by": "api"},
            )
        return run

    @staticmethod
    async def get_task(run_id: str, task_id: str) -> WorkflowTask | None:
        active_ledger = get_ledger()
        return await active_ledger.get_task(run_id, task_id)

    @staticmethod
    async def list_agents(run_id: str) -> list[AgentExecution]:
        active_ledger = get_ledger()
        return await active_ledger.list_agent_executions_for_run(run_id)

    @staticmethod
    async def get_agent(run_id: str, execution_id: str) -> AgentExecution | None:
        active_ledger = get_ledger()
        return await active_ledger.get_agent_execution(run_id, execution_id)

    @staticmethod
    async def list_agent_tools(
        run_id: str,
        execution_id: str,
        cursor: str | None = None,
        limit: int = 50,
    ) -> ToolInvocationsPageResponse:
        active_ledger = get_ledger()
        return await active_ledger.list_tools_for_execution(
            execution_id=execution_id, cursor=cursor, limit=limit
        )

    @staticmethod
    async def get_tool_output(
        run_id: str,
        tool_invocation_id: str,
        offset: int = 0,
        limit: int = 1000,
    ) -> ToolOutputResponse | None:
        active_ledger = get_ledger()
        tool = await active_ledger.get_tool_invocation(run_id, tool_invocation_id)
        if not tool:
            return None
        summary = tool.output_summary or ""
        chunk = summary[offset : offset + limit]
        return ToolOutputResponse(
            tool_invocation_id=tool_invocation_id,
            offset=offset,
            limit=limit,
            total_bytes=len(summary),
            has_more=(offset + limit) < len(summary),
            content=chunk,
        )

    @staticmethod
    async def list_delegations(run_id: str) -> list[Delegation]:
        active_ledger = get_ledger()
        return await active_ledger.list_delegations_for_run(run_id)

    @staticmethod
    async def list_artifacts(run_id: str) -> list[Artifact]:
        active_ledger = get_ledger()
        return await active_ledger.list_artifacts_for_run(run_id)

    @staticmethod
    async def get_logs(
        run_id: str,
        cursor: str | None = None,
        limit: int = 100,
        severity: str | None = None,
        task_id: str | None = None,
        execution_id: str | None = None,
    ) -> LogsPageResponse:
        active_ledger = get_ledger()
        return await active_ledger.get_logs_page(
            run_id=run_id,
            cursor=cursor,
            limit=limit,
            severity=severity,
            task_id=task_id,
            execution_id=execution_id,
        )

    @staticmethod
    async def get_task_graph(run_id: str) -> TaskGraphResponse:
        active_ledger = get_ledger()
        return await active_ledger.get_task_graph(run_id)

    @staticmethod
    async def get_delegation_graph(run_id: str) -> DelegationGraphResponse:
        active_ledger = get_ledger()
        return await active_ledger.get_delegation_graph(run_id)

    @staticmethod
    async def get_events_history(
        run_id: str,
        after_seq: int = 0,
        limit: int = 50,
    ) -> EventsHistoryPageResponse:
        active_ledger = get_ledger()
        return await active_ledger.get_events_history(
            run_id=run_id, after_seq=after_seq, limit=limit
        )
