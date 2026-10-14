import asyncio
import contextlib
import json
import logging
import uuid
from typing import Any, AsyncGenerator

from fastapi import APIRouter, Depends, Header, HTTPException, Query, Request, status
from fastapi.responses import StreamingResponse
from pydantic import BaseModel, Field

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
from src.orchestration.auth import verify_internal_auth
from src.orchestration.ledger import SurrealLedgerRepository, set_event_publisher
from src.orchestration.redis_queue import RedisStreamJobQueue

logger = logging.getLogger("mensetsu.orchestration.internal_router")
SSE_PING_INTERVAL_S = 15.0
SSE_IDLE_REPLAY_S = 5.0

router = APIRouter(
    prefix="/internal/v1/workflows",
    dependencies=[Depends(verify_internal_auth)],
    tags=["workflows-internal"],
)

default_ledger = SurrealLedgerRepository()
job_queue = RedisStreamJobQueue()
set_event_publisher(job_queue.publish_event)


class _LedgerProxy:
    def __getattr__(self, name):
        from src.agents.runtime import current_ledger_var
        ctx_ledger = current_ledger_var.get()
        target = ctx_ledger if ctx_ledger is not None else default_ledger
        return getattr(target, name)


ledger = _LedgerProxy()


def get_ledger() -> SurrealLedgerRepository:
    from src.agents.runtime import current_ledger_var
    ctx_ledger = current_ledger_var.get()
    return ctx_ledger if ctx_ledger is not None else default_ledger


DEFINITIONS = [
    WorkflowDefinition(
        id="interview-preparation-static",
        name="Static Interview Preparation Workflow",
        type="static",
        version="1.0.0",
        description="Deterministic fan-out/fan-in research and question generation pipeline",
        default_max_retries=3,
        supported_agents=[
            "company-research",
            "job-analysis",
            "technical-research",
            "systems-deep-dive",
            "evidence-evaluator",
            "curriculum",
            "interview-generation",
        ],
    ),
    WorkflowDefinition(
        id="interview-preparation-dynamic",
        name="Dynamic Interview Preparation Workflow",
        type="dynamic",
        version="1.0.0",
        description="Adaptive planner-driven DAG research and study path compiler",
        default_max_retries=3,
        supported_agents=[
            "planner",
            "company-research",
            "job-analysis",
            "technical-research",
            "systems-deep-dive",
            "evidence-evaluator",
            "curriculum",
            "interview-generation",
        ],
    ),
]


class CreateRunRequest(BaseModel):
    workspace_id: str
    user_id: str
    workflow_id: str = "interview-preparation-static"
    company_name: str
    role_title: str
    company_urls: list[str] = Field(default_factory=list)
    resume_text: str = ""
    resume_pdf_base64: str | None = None
    resume_filename: str | None = None
    job_description: str = ""
    jd_url: str | None = None
    seniority: str | None = None
    llm_config: dict[str, Any] | None = None


@router.get("", response_model=list[WorkflowDefinition])
async def list_workflow_definitions() -> list[WorkflowDefinition]:
    return DEFINITIONS


@router.post("/runs", response_model=WorkflowRun, status_code=status.HTTP_201_CREATED)
async def create_workflow_run(req: CreateRunRequest) -> WorkflowRun:

    wf_def = next((d for d in DEFINITIONS if d.id == req.workflow_id), None)
    if wf_def is None:
        raise HTTPException(
            status_code=400,
            detail=f"Unknown workflow_id '{req.workflow_id}'. Known: {[d.id for d in DEFINITIONS]}",
        )
    try:
        run_id = f"run_{uuid.uuid4().hex}"
        run = WorkflowRun(
            id=run_id,
            workflow_id=wf_def.id,
            workflow_name=wf_def.name,
            workspace_id=req.workspace_id,
            user_id=req.user_id,
            type=wf_def.type,
            status="pending",
            current_phase="queued",
            plan_revision=1,
            last_seq=0,
        )

        await ledger.create_workflow_run(run)

        await ledger.append_event_transactional(
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
                "workspace_id": req.workspace_id,
                "user_id": req.user_id,
                "company_name": req.company_name,
                "role_title": req.role_title,
                "company_urls": req.company_urls,
                "resume_text": req.resume_text,
                "resume_pdf_base64": req.resume_pdf_base64,
                "resume_filename": req.resume_filename,
                "job_description": req.job_description,
                "jd_url": req.jd_url,
                "seniority": req.seniority,
                "llm_config": req.llm_config,
            },
        )

        run.last_seq = 1
        return run
    except Exception as e:
        logger.error(f"Failed to create workflow run: {e}")
        raise HTTPException(status_code=503, detail=f"Failed to initialize run in storage: {e}")


@router.get("/runs", response_model=RunsPageResponse)
async def list_runs(
    run_ids: str | None = Query(None),
    workspace_id: str | None = Query(None),
    status: str | None = Query(None),
    type: str | None = Query(None),
    limit: int = Query(50, ge=1, le=100),
    cursor: str | None = Query(None),
) -> RunsPageResponse:
    if run_ids:
        ids = [i.strip() for i in run_ids.split(",") if i.strip()]
        if len(ids) > 100:
            raise HTTPException(status_code=400, detail="Maximum 100 run_ids allowed.")
        runs = []
        for r_id in ids:
            run = await ledger.get_workflow_run(r_id)
            if run:
                runs.append(run)
        return RunsPageResponse(items=runs, next_cursor=None)

    return await ledger.list_workflow_runs(
        workspace_id=workspace_id, status=status, run_type=type, limit=limit, cursor=cursor
    )


@router.get("/runs/{run_id}", response_model=WorkflowRun)
async def get_run(run_id: str) -> WorkflowRun:
    run = await ledger.get_workflow_run(run_id)
    if not run:
        raise HTTPException(status_code=404, detail="Run not found")
    return run


@router.get("/runs/{run_id}/tasks/{task_id}", response_model=WorkflowTask)
async def get_task(run_id: str, task_id: str) -> WorkflowTask:
    task = await ledger.get_task(run_id, task_id)
    if not task:
        raise HTTPException(status_code=404, detail="Task not found")
    return task


@router.get("/runs/{run_id}/agents", response_model=list[AgentExecution])
async def list_agents(run_id: str) -> list[AgentExecution]:
    return await ledger.list_agent_executions_for_run(run_id)


@router.get("/runs/{run_id}/agents/{execution_id}", response_model=AgentExecution)
async def get_agent(run_id: str, execution_id: str) -> AgentExecution:
    ledger = get_ledger()
    agent = await ledger.get_agent_execution(run_id, execution_id)
    if not agent or agent.run_id != run_id:
        raise HTTPException(status_code=404, detail="Agent execution not found")
    return agent


@router.get(
    "/runs/{run_id}/agents/{execution_id}/tools", response_model=ToolInvocationsPageResponse
)
async def list_agent_tools(
    run_id: str,
    execution_id: str,
    cursor: str | None = Query(None),
    limit: int = Query(50, ge=1, le=100),
) -> ToolInvocationsPageResponse:
    return await ledger.list_tools_for_execution(
        execution_id=execution_id, cursor=cursor, limit=limit
    )


@router.get("/runs/{run_id}/tools/{tool_invocation_id}/output", response_model=ToolOutputResponse)
async def get_tool_output(
    run_id: str,
    tool_invocation_id: str,
    offset: int = Query(0, ge=0),
    limit: int = Query(1000, ge=1, le=10000),
) -> ToolOutputResponse:
    ledger = get_ledger()
    tool = await ledger.get_tool_invocation(run_id, tool_invocation_id)
    if not tool:
        raise HTTPException(status_code=404, detail="Tool invocation not found")
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


@router.get("/runs/{run_id}/delegations", response_model=list[Delegation])
async def list_delegations(run_id: str) -> list[Delegation]:
    return await ledger.list_delegations_for_run(run_id)


@router.get("/runs/{run_id}/artifacts", response_model=list[Artifact])
async def list_artifacts(run_id: str) -> list[Artifact]:
    return await ledger.list_artifacts_for_run(run_id)


@router.get("/runs/{run_id}/logs", response_model=LogsPageResponse)
async def get_logs(
    run_id: str,
    cursor: str | None = Query(None),
    limit: int = Query(100, ge=1, le=500),
    severity: str | None = Query(None),
    task_id: str | None = Query(None),
    execution_id: str | None = Query(None),
) -> LogsPageResponse:
    if cursor is not None and not cursor.isdigit():
        raise HTTPException(status_code=400, detail="Invalid logs cursor: must be a numeric log id.")
    ledger = get_ledger()
    return await ledger.get_logs_page(
        run_id=run_id,
        cursor=cursor,
        limit=limit,
        severity=severity,
        task_id=task_id,
        execution_id=execution_id,
    )


@router.get("/runs/{run_id}/graph", response_model=TaskGraphResponse)
async def get_task_graph(run_id: str) -> TaskGraphResponse:
    return await ledger.get_task_graph(run_id)


@router.get("/runs/{run_id}/delegations/graph", response_model=DelegationGraphResponse)
async def get_delegations_graph(run_id: str) -> DelegationGraphResponse:
    return await ledger.get_delegation_graph(run_id)


@router.post("/runs/{run_id}/cancel", response_model=WorkflowRun)
async def cancel_run(run_id: str) -> WorkflowRun:

    run = await ledger.get_workflow_run(run_id)
    if not run:
        raise HTTPException(status_code=404, detail="Run not found")
    if run.status != "cancelled":
        run.status = "cancelled"
        await ledger.update_workflow_run(run_id, {"status": "cancelled"})
        try:
            await ledger.sweep_run_to_terminal(run_id, reason="run cancelled")
        except Exception as se:
            logger.error(f"Run-end sweep failed for cancelled run {run_id}: {se}")
        await ledger.append_event_transactional(
            run_id=run_id,
            event_type="workflow.cancelled",
            payload={"reason": "User cancellation requested", "cancelled_by": "api"},
        )
    return run


@router.get("/runs/{run_id}/events/history", response_model=EventsHistoryPageResponse)
async def get_events_history(
    run_id: str,
    after_seq: int = Query(0, ge=0),
    limit: int = Query(50, ge=1, le=100),
) -> EventsHistoryPageResponse:
    return await ledger.get_events_history(run_id=run_id, after_seq=after_seq, limit=limit)


@router.get("/runs/{run_id}/events")
async def stream_events(
    request: Request,
    run_id: str,
    after_seq: int | None = Query(None),
    last_event_id: str | None = Header(None, alias="Last-Event-ID"),
):

    start_seq = (
        after_seq
        if after_seq is not None
        else (int(last_event_id) if last_event_id and last_event_id.isdigit() else 0)
    )
    terminal_types = ("workflow.completed", "workflow.failed", "workflow.cancelled")

    async def sse_generator() -> AsyncGenerator[str, None]:
        curr_seq = start_seq

        async def missed_events() -> AsyncGenerator[tuple[int, str, str], None]:
            nonlocal curr_seq
            while True:
                history = await ledger.get_events_history(run_id=run_id, after_seq=curr_seq, limit=100)
                for evt in history.events:
                    if evt.seq > curr_seq:
                        curr_seq = evt.seq
                        yield evt.seq, evt.type, evt.model_dump_json()
                if history.next_after_seq is None or not history.events:
                    return

        loop = asyncio.get_running_loop()
        live_gen = job_queue.subscribe_events(run_id)
        next_event: asyncio.Future | None = asyncio.ensure_future(live_gen.__anext__())
        next_ping_at = loop.time() + SSE_PING_INTERVAL_S
        try:
            needs_replay = True
            while not await request.is_disconnected():
                if needs_replay:
                    needs_replay = False
                    async for seq, evt_type, data in missed_events():
                        yield f"id: {seq}\nevent: {evt_type}\ndata: {data}\n\n"
                        if evt_type in terminal_types:
                            return
                        if await request.is_disconnected():
                            return
                if loop.time() >= next_ping_at:
                    yield ": ping\n\n"
                    next_ping_at = loop.time() + SSE_PING_INTERVAL_S
                    needs_replay = True
                    continue
                if next_event is None:
                    next_event = asyncio.ensure_future(live_gen.__anext__())
                wait_s = min(SSE_IDLE_REPLAY_S, max(0.0, next_ping_at - loop.time()))
                done, _ = await asyncio.wait({next_event}, timeout=wait_s)
                if not done:
                    needs_replay = True
                    continue
                event_str = next_event.result()
                next_event = None
                evt_data = json.loads(event_str)
                seq = evt_data.get("seq")
                if not isinstance(seq, int) or seq <= curr_seq:
                    continue
                if seq != curr_seq + 1:
                    needs_replay = True
                    continue
                curr_seq = seq
                yield f"id: {seq}\nevent: {evt_data.get('type')}\ndata: {event_str}\n\n"
                if evt_data.get("type") in terminal_types:
                    return
        except StopAsyncIteration:
            return
        except Exception as e:
            logger.debug(f"SSE live streaming stream ended for {run_id}: {e}")
        finally:
            if next_event is not None and not next_event.done():
                next_event.cancel()
                with contextlib.suppress(BaseException):
                    await next_event
            await live_gen.aclose()

    return StreamingResponse(
        sse_generator(),
        media_type="text/event-stream",
        headers={
            "Cache-Control": "no-cache",
            "Connection": "keep-alive",
            "X-Accel-Buffering": "no",
        },
    )
