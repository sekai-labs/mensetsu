import asyncio
import logging
import random
import time
import uuid
from collections.abc import Awaitable, Callable
from datetime import datetime, timezone
from typing import Any

from pydantic import TypeAdapter

from src.contracts.v1 import (
    AgentExecution,
    Artifact,
    Delegation,
    DelegationGraphEdge,
    DelegationGraphEdgeData,
    DelegationGraphNode,
    DelegationGraphNodeData,
    DelegationGraphResponse,
    EventsHistoryPageResponse,
    ExecutionEvent,
    LogEntry,
    LogsPageResponse,
    RunsPageResponse,
    TaskGraphEdge,
    TaskGraphNode,
    TaskGraphNodeData,
    TaskGraphResponse,
    ToolInvocation,
    ToolInvocationsPageResponse,
    WorkflowProgress,
    WorkflowRun,
    WorkflowTask,
)
from src.db.client import SurrealDirectClient, SurrealQueryError
from src.orchestration.id_validator import validate_id
from src.orchestration.sanitizer import sanitize_data

logger = logging.getLogger("mensetsu.orchestration.ledger")


def extract_rows(res: Any) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for item in res if isinstance(res, list) else []:
        if isinstance(item, dict) and isinstance(item.get("result"), list):
            rows.extend(r for r in item["result"] if isinstance(r, dict))
    return rows


WRITE_CONFLICT_MARKERS = ("can be retried", "transaction conflict")
PROGRESS_RETRY_ATTEMPTS = 5
DYNAMIC_EXPANDABLE_PHASES = ("running", "planning", "executing_plan")


def is_write_conflict(err: BaseException) -> bool:
    text = str(err).lower()
    return isinstance(err, SurrealQueryError) and any(m in text for m in WRITE_CONFLICT_MARKERS)


async def retry_on_write_conflict(operation, description: str, attempts: int):
    for attempt in range(1, attempts + 1):
        try:
            return await operation()
        except Exception as e:
            if not is_write_conflict(e) or attempt == attempts:
                if is_write_conflict(e):
                    logger.error(f"{description}: write conflict persisted after {attempts} attempts: {e}")
                raise
            await asyncio.sleep(random.uniform(0.02, 0.2))
    raise RuntimeError(f"{description}: retry loop exited without result")

WORKFLOW_RUN_ALLOWED_FIELDS = {
    "workflow_id",
    "workflow_name",
    "type",
    "status",
    "current_phase",
    "plan_revision",
    "progress",
    "last_seq",
    "started_at",
    "completed_at",
    "duration_ms",
    "error_message",
    "degraded",
    "degraded_reasons",
}

LOGGED_EVENT_DOMAINS = {"workflow", "task", "agent", "tool", "delegation"}


def _event_severity(event_type: str) -> str:
    if event_type.endswith(".failed"):
        return "error"
    if event_type.endswith((".retrying", ".cancelled")):
        return "warn"
    if event_type == "agent.activity":
        return "debug"
    return "info"


def _describe_event(event_type: str, data: dict[str, Any]) -> str:
    subject = (
        data.get("tool_name")
        or data.get("agent_id")
        or data.get("assigned_agent")
        or data.get("task_id")
        or data.get("execution_id")
        or ""
    )
    detail = (
        data.get("error")
        or data.get("description")
        or data.get("output_summary")
        or data.get("summary")
        or data.get("objective")
        or data.get("reason")
        or ""
    )
    parts = [event_type]
    if subject:
        parts.append(str(subject))
    if detail:
        parts.append(str(detail)[:500])
    return " | ".join(parts)


EventPublisher = Callable[[str, str], Awaitable[None]]
_event_publisher: EventPublisher | None = None


def set_event_publisher(publisher: EventPublisher | None) -> None:
    global _event_publisher
    _event_publisher = publisher


async def publish_committed_event(evt: ExecutionEvent) -> None:
    publisher = _event_publisher
    if publisher is None:
        return
    try:
        await publisher(evt.run_id, evt.model_dump_json())
    except Exception as pe:
        logger.warning(f"Live publish failed for {evt.type} seq={evt.seq} (history replay still has it): {pe}")


class SurrealLedgerRepository:
    def __init__(self, db_client: SurrealDirectClient | None = None):
        self.db = db_client or SurrealDirectClient()

    async def create_workflow_run(self, run: WorkflowRun) -> WorkflowRun:
        clean_id = validate_id(run.id, "run_id")
        clean_ws = validate_id(run.workspace_id, "workspace_id")

        set_items = [
            "workspace_id = type::record('workspace', $workspace_id)",
            "user_id = $user_id",
            "workflow_id = $workflow_id",
            "workflow_name = $workflow_name",
            "type = $type",
            "status = $status",
            "current_phase = $current_phase",
            "plan_revision = $plan_revision",
            "progress = $progress",
            "last_seq = $last_seq",
            "created_at = time::now()",
            "updated_at = time::now()",
        ]
        if run.error_message:
            set_items.append("error_message = $error_message")

        set_joined = ", ".join(set_items)
        query = "UPSERT type::record('workflow_run', $run_id) SET " + set_joined + ";"
        vars_payload = {
            "run_id": clean_id,
            "workspace_id": clean_ws,
            "user_id": run.user_id,
            "workflow_id": run.workflow_id,
            "workflow_name": run.workflow_name,
            "type": run.type,
            "status": run.status,
            "current_phase": run.current_phase,
            "plan_revision": run.plan_revision,
            "progress": run.progress.model_dump(),
            "last_seq": run.last_seq,
        }
        if run.error_message:
            vars_payload["error_message"] = run.error_message
        await self.db.execute_query(query, vars=vars_payload)
        return run

    async def get_workflow_run(self, run_id: str) -> WorkflowRun | None:
        clean_id = validate_id(run_id, "run_id")
        query = "SELECT * FROM type::record('workflow_run', $run_id);"
        res = await self.db.execute_query(query, vars={"run_id": clean_id})
        if isinstance(res, list) and res:
            for item in res:
                if isinstance(item, dict) and item.get("result"):
                    rows = item["result"]
                    if rows and isinstance(rows, list):
                        row = dict(rows[0])
                        if "workspace_id" in row and isinstance(row["workspace_id"], str):
                            row["workspace_id"] = row["workspace_id"].replace("workspace:", "")
                        if "id" in row and isinstance(row["id"], str):
                            row["id"] = row["id"].replace("workflow_run:", "")
                        return WorkflowRun.model_validate(row)
        return None

    async def update_workflow_run(self, run_id: str, updates: dict[str, Any]) -> None:
        clean_id = validate_id(run_id, "run_id")
        clean_updates = dict(updates)
        clean_updates.pop("id", None)

        assignments = []
        vars_payload = {"run_id": clean_id}

        for k, v in clean_updates.items():
            if k in WORKFLOW_RUN_ALLOWED_FIELDS and v is not None:
                if k == "started_at":
                    assignments.append("started_at = started_at ?? type::datetime($started_at)")
                    vars_payload[k] = v.isoformat() if hasattr(v, "isoformat") else str(v)
                elif k == "completed_at":
                    assignments.append("completed_at = type::datetime($completed_at)")
                    vars_payload[k] = v.isoformat() if hasattr(v, "isoformat") else str(v)
                else:
                    assignments.append(f"{k} = ${k}")
                    vars_payload[k] = v.model_dump() if hasattr(v, "model_dump") else v
        assignments.append("updated_at = time::now()")
        set_str = ", ".join(assignments)
        query = "UPDATE type::record('workflow_run', $run_id) SET " + set_str + ";"
        await self.db.execute_query(query, vars=vars_payload)

    async def refresh_run_progress(self, run_id: str) -> WorkflowProgress:
        clean_run_id = validate_id(run_id, "run_id")

        async def attempt_refresh() -> WorkflowProgress:
            res = await self.db.execute_query(
                "SELECT status, count() AS n FROM workflow_task WHERE run_id = type::record('workflow_run', $run_id) GROUP BY status;",
                vars={"run_id": clean_run_id},
            )
            counts: dict[str, int] = {}
            last = res[-1] if isinstance(res, list) and res else None
            for row in (last.get("result") if isinstance(last, dict) else None) or []:
                if isinstance(row, dict) and row.get("status") in WorkflowProgress.model_fields:
                    counts[row["status"]] = int(row.get("n", 0))
            total = sum(counts.values())
            run = await self.get_workflow_run(clean_run_id)
            planned = (run.progress.phase_count or 0) if run else 0
            denominator = max(total, planned)
            plan_expandable = bool(
                run and run.type == "dynamic" and run.current_phase in DYNAMIC_EXPANDABLE_PHASES
            )
            if plan_expandable:
                percentage = None
            elif denominator:
                percentage = int(counts.get("completed", 0) * 100 / denominator)
            else:
                percentage = 0
            progress = WorkflowProgress(
                total_tasks=denominator,
                percentage=percentage,
                estimate=plan_expandable or total < planned,
                phase_index=counts.get("completed", 0),
                phase_count=planned or None,
                **counts,
            )
            await self.update_workflow_run(clean_run_id, {"progress": progress})
            return progress

        return await retry_on_write_conflict(
            attempt_refresh, f"progress refresh for {clean_run_id}", attempts=PROGRESS_RETRY_ATTEMPTS
        )

    async def list_workflow_runs(
        self,
        workspace_id: str | None = None,
        status: str | None = None,
        run_type: str | None = None,
        limit: int = 50,
        cursor: str | None = None,
    ) -> RunsPageResponse:
        conditions = []
        vars_payload: dict[str, Any] = {"limit": limit}

        if workspace_id:
            clean_ws = validate_id(workspace_id, "workspace_id")
            conditions.append("workspace_id = type::record('workspace', $workspace_id)")
            vars_payload["workspace_id"] = clean_ws
        if status:
            conditions.append("status = $status")
            vars_payload["status"] = status
        if run_type:
            conditions.append("type = $run_type")
            vars_payload["run_type"] = run_type
        if cursor:
            clean_cur = validate_id(cursor, "cursor")
            conditions.append("id < type::record('workflow_run', $cursor)")
            vars_payload["cursor"] = clean_cur

        where_clause = ("WHERE " + " AND ".join(conditions)) if conditions else ""
        query = "SELECT * FROM workflow_run " + where_clause + " ORDER BY created_at DESC LIMIT $limit;"
        res = await self.db.execute_query(query, vars=vars_payload)
        items: list[WorkflowRun] = []
        if isinstance(res, list) and res:
            for item in res:
                if isinstance(item, dict) and item.get("result"):
                    for r in item["result"]:
                        row = dict(r)
                        if "workspace_id" in row and isinstance(row["workspace_id"], str):
                            row["workspace_id"] = row["workspace_id"].replace("workspace:", "")
                        if "id" in row and isinstance(row["id"], str):
                            row["id"] = row["id"].replace("workflow_run:", "")
                        items.append(WorkflowRun.model_validate(row))
        next_cursor = items[-1].id if len(items) == limit else None
        return RunsPageResponse(items=items, next_cursor=next_cursor)

    async def append_event_transactional(
        self,
        run_id: str,
        event_type: str,
        payload: dict[str, Any],
        task_id: str | None = None,
        execution_id: str | None = None,
        parent_execution_id: str | None = None,
        tool_invocation_id: str | None = None,
        correlation_id: str | None = None,
        trace_id: str | None = None,
        max_retries: int = 30,
    ) -> ExecutionEvent:
        evt = await self._append_event(
            run_id=run_id,
            event_type=event_type,
            payload=payload,
            task_id=task_id,
            execution_id=execution_id,
            parent_execution_id=parent_execution_id,
            tool_invocation_id=tool_invocation_id,
            correlation_id=correlation_id,
            trace_id=trace_id,
            max_retries=max_retries,
        )
        await publish_committed_event(evt)
        if event_type.split(".")[0] in LOGGED_EVENT_DOMAINS:
            data = payload.model_dump() if hasattr(payload, "model_dump") else dict(payload)
            try:
                await self.append_execution_log(
                    run_id=run_id,
                    message=_describe_event(event_type, data),
                    severity=_event_severity(event_type),
                    component=event_type.split(".")[0],
                    task_id=task_id,
                    execution_id=execution_id,
                    tool_invocation_id=tool_invocation_id,
                    duration_ms=data.get("duration_ms"),
                    attributes={"event_type": event_type, "seq": evt.seq},
                    seq=evt.seq,
                )
            except Exception as log_err:
                logger.warning(f"Failed to write execution log for {event_type}: {log_err}")
        return evt

    async def _append_event(
        self,
        run_id: str,
        event_type: str,
        payload: dict[str, Any],
        task_id: str | None = None,
        execution_id: str | None = None,
        parent_execution_id: str | None = None,
        tool_invocation_id: str | None = None,
        correlation_id: str | None = None,
        trace_id: str | None = None,
        max_retries: int = 30,
    ) -> ExecutionEvent:
        clean_run_id = validate_id(run_id, "run_id")
        raw_payload = (
            payload.model_dump()
            if hasattr(payload, "model_dump")
            else dict(payload)
        )
        sanitized_payload = sanitize_data(raw_payload)

        async def attempt_append() -> ExecutionEvent:
            event_content = {
                "schema_version": "1.0",
                "event_id": f"evt_{uuid.uuid4().hex}",
                "type": event_type,
                "payload": sanitized_payload,
            }
            if task_id is not None:
                event_content["task_id"] = task_id
            if execution_id is not None:
                event_content["execution_id"] = execution_id
            if parent_execution_id is not None:
                event_content["parent_execution_id"] = parent_execution_id
            if tool_invocation_id is not None:
                event_content["tool_invocation_id"] = tool_invocation_id
            if correlation_id is not None:
                event_content["correlation_id"] = correlation_id
            if trace_id is not None:
                event_content["trace_id"] = trace_id

            set_items = [f"{k} = ${k}" for k in event_content.keys()]
            set_items.append("run_id = type::record('workflow_run', $run_id)")
            set_items.append("seq = $seq")
            set_items.append("ts = time::now()")

            vars_payload = dict(event_content)
            vars_payload["run_id"] = clean_run_id

            set_items_str = ", ".join(set_items)
            txn_query = (
                "BEGIN TRANSACTION;\n"
                "LET $r = (UPDATE type::record('workflow_run', $run_id) SET last_seq += 1, updated_at = time::now() RETURN AFTER);\n"
                "LET $seq = (IF array::len($r) > 0 THEN $r[0].last_seq ELSE 1 END);\n"
                "LET $rec_id = string::concat($run_id, '_', type::string($seq));\n"
                "UPSERT type::record('execution_event', $rec_id) SET " + set_items_str + " RETURN AFTER;\n"
                "COMMIT TRANSACTION;\n"
            )
            res = await self.db.execute_query(txn_query, vars=vars_payload)
            for item in res:
                rows = item.get("result") if isinstance(item, dict) else None
                if (
                    isinstance(rows, list)
                    and rows
                    and isinstance(rows[0], dict)
                    and "seq" in rows[0]
                    and "type" in rows[0]
                ):
                    row = dict(rows[0])
                    row.pop("id", None)
                    if "run_id" in row and isinstance(row["run_id"], str):
                        row["run_id"] = row["run_id"].replace("workflow_run:", "")
                    return TypeAdapter(ExecutionEvent).validate_python(row)
            raise RuntimeError(
                f"Event append transaction for run {clean_run_id} committed without returning the event row"
            )

        return await retry_on_write_conflict(
            attempt_append, f"event append for {clean_run_id}", attempts=max_retries
        )

    async def get_events_history(
        self, run_id: str, after_seq: int = 0, limit: int = 50
    ) -> EventsHistoryPageResponse:
        clean_run_id = validate_id(run_id, "run_id")
        query = """
        SELECT * FROM execution_event 
        WHERE run_id = type::record('workflow_run', $run_id) AND seq > $after_seq 
        ORDER BY seq ASC LIMIT $limit;
        """
        res = await self.db.execute_query(
            query, vars={"run_id": clean_run_id, "after_seq": after_seq, "limit": limit}
        )
        events: list[ExecutionEvent] = []
        adapter = TypeAdapter(ExecutionEvent)
        if isinstance(res, list) and res:
            for item in res:
                if isinstance(item, dict) and item.get("result"):
                    for r in item["result"]:
                        row = dict(r)
                        row.pop("id", None)
                        if "run_id" in row and isinstance(row["run_id"], str):
                            row["run_id"] = row["run_id"].replace("workflow_run:", "")
                        events.append(adapter.validate_python(row))

        run = await self.get_workflow_run(clean_run_id)
        last_seq = run.last_seq if run else 0
        next_after_seq = events[-1].seq if len(events) == limit else None
        return EventsHistoryPageResponse(
            events=events, next_after_seq=next_after_seq, last_seq=last_seq
        )

    async def save_task(self, task: WorkflowTask) -> None:
        clean_id = task.id.replace("workflow_task:", "")
        clean_run_id = validate_id(task.run_id, "run_id")
        scoped_id = f"{clean_run_id}__{clean_id}"

        query = """
        UPSERT type::record('workflow_task', $scoped_id) SET
            run_id = type::record('workflow_run', $run_id),
            task_type = $task_type,
            name = $name,
            assigned_agent = $assigned_agent,
            dependencies = $dependencies,
            status = $status,
            attempt = $attempt,
            required = $required,
            added_in_revision = $added_in_revision,
            execution_ids = $execution_ids,
            input_data = $input_data,
            output_data = $output_data,
            created_at = created_at ?? time::now(),
            updated_at = time::now();
        """
        vars_payload = {
            "scoped_id": scoped_id,
            "run_id": clean_run_id,
            "task_type": task.task_type,
            "name": task.name,
            "assigned_agent": task.assigned_agent,
            "dependencies": task.dependencies,
            "status": task.status,
            "attempt": task.attempt,
            "required": task.required,
            "added_in_revision": task.added_in_revision,
            "execution_ids": task.execution_ids,
            "input_data": task.input_data,
            "output_data": task.output_data,
        }
        if task.error is not None:
            query = query.replace(
                "updated_at = time::now();", "error = $error, updated_at = time::now();"
            )
            vars_payload["error"] = sanitize_data(task.error)
        await self.db.execute_query(query, vars=vars_payload)
        try:
            await self.refresh_run_progress(clean_run_id)
        except Exception as pe:
            logger.warning(f"Failed to refresh run progress for {clean_run_id}: {pe}")

    async def next_task_attempt(self, run_id: str, task_id: str) -> int:
        existing = await self.get_task(run_id, task_id)
        return existing.attempt + 1 if existing is not None else 1

    async def get_task(self, run_id: str, task_id: str) -> WorkflowTask | None:
        clean_run_id = validate_id(run_id, "run_id")
        clean_task_id = task_id.replace("workflow_task:", "")
        scoped_id = f"{clean_run_id}__{clean_task_id}"
        query = "SELECT * FROM type::record('workflow_task', $scoped_id);"
        res = await self.db.execute_query(query, vars={"scoped_id": scoped_id})
        if isinstance(res, list) and res:
            for item in res:
                if isinstance(item, dict) and item.get("result"):
                    rows = item["result"]
                    if rows and isinstance(rows, list):
                        row = dict(rows[0])
                        row["id"] = clean_task_id
                        if "run_id" in row and isinstance(row["run_id"], str):
                            row["run_id"] = row["run_id"].replace("workflow_run:", "")
                        return WorkflowTask.model_validate(row)
        return None

    async def list_tasks_for_run(self, run_id: str) -> list[WorkflowTask]:
        clean_run_id = validate_id(run_id, "run_id")
        query = "SELECT * FROM workflow_task WHERE run_id = type::record('workflow_run', $run_id);"
        res = await self.db.execute_query(query, vars={"run_id": clean_run_id})
        tasks = []
        if isinstance(res, list) and res:
            for item in res:
                if isinstance(item, dict) and item.get("result"):
                    for r in item["result"]:
                        row = dict(r)
                        raw_id = row.get("id", "").replace("workflow_task:", "")
                        row["id"] = raw_id.split("__")[-1] if "__" in raw_id else raw_id
                        if "run_id" in row and isinstance(row["run_id"], str):
                            row["run_id"] = row["run_id"].replace("workflow_run:", "")
                        tasks.append(WorkflowTask.model_validate(row))
        return tasks

    async def save_agent_execution(self, execution: AgentExecution) -> None:
        clean_id = validate_id(execution.id, "execution_id")
        clean_run_id = validate_id(execution.run_id, "run_id")

        query = """
        UPSERT type::record('agent_execution', $exec_id) SET
            run_id = type::record('workflow_run', $run_id),
            task_id = $task_id,
            parent_execution_id = $parent_execution_id,
            agent_id = $agent_id,
            objective = $objective,
            status = $status,
            delegation_depth = $delegation_depth,
            retries = $retries,
            child_execution_ids = $child_execution_ids,
            model_provider = $model_provider,
            model_name = $model_name,
            budgets = $budgets,
            budgets_used = $budgets_used,
            tool_call_count = $tool_call_count,
            notes = $notes,
            artifact_ids = $artifact_ids,
            public_summary = $public_summary;
        """
        vars_payload = {
            "exec_id": clean_id,
            "run_id": clean_run_id,
            "task_id": execution.task_id,
            "parent_execution_id": execution.parent_execution_id,
            "agent_id": execution.agent_id,
            "objective": execution.objective,
            "status": execution.status,
            "delegation_depth": execution.delegation_depth,
            "retries": execution.retries,
            "child_execution_ids": execution.child_execution_ids,
            "model_provider": execution.model_provider,
            "model_name": execution.model_name,
            "budgets": execution.budgets.model_dump(),
            "budgets_used": execution.budgets_used.model_dump(),
            "tool_call_count": execution.tool_call_count,
            "notes": [n.model_dump() for n in execution.notes],
            "artifact_ids": execution.artifact_ids,
            "public_summary": execution.public_summary,
        }
        if execution.started_at:
            query = query.replace("public_summary = $public_summary;", "public_summary = $public_summary, started_at = started_at ?? type::datetime($started_at);")
            vars_payload["started_at"] = execution.started_at.isoformat()
        else:
            query = query.replace("public_summary = $public_summary;", "public_summary = $public_summary, started_at = started_at ?? time::now();")

        if execution.status in ("completed", "failed", "cancelled"):
            if execution.completed_at:
                query = query.replace("started_at = ", "completed_at = type::datetime($completed_at), started_at = ", 1)
                vars_payload["completed_at"] = execution.completed_at.isoformat()
            else:
                query = query.replace("started_at = ", "completed_at = time::now(), started_at = ", 1)
        if execution.error is not None:
            query = query.replace("started_at = ", "error = $error, started_at = ", 1)
            vars_payload["error"] = execution.error
        await self.db.execute_query(query, vars=vars_payload)

    async def get_agent_execution(self, run_id: str, execution_id: str) -> AgentExecution | None:
        clean_id = validate_id(execution_id, "execution_id")
        clean_run_id = validate_id(run_id, "run_id")
        query = "SELECT * FROM type::record('agent_execution', $exec_id) WHERE run_id = type::record('workflow_run', $run_id);"
        res = await self.db.execute_query(query, vars={"exec_id": clean_id, "run_id": clean_run_id})
        if isinstance(res, list) and res:
            for item in res:
                if isinstance(item, dict) and item.get("result"):
                    rows = item["result"]
                    if rows and isinstance(rows, list):
                        row = dict(rows[0])
                        row["id"] = row.get("id", "").replace("agent_execution:", "")
                        if "run_id" in row and isinstance(row["run_id"], str):
                            row["run_id"] = row["run_id"].replace("workflow_run:", "")
                        return AgentExecution.model_validate(row)
        return None

    async def list_agent_executions_for_run(self, run_id: str) -> list[AgentExecution]:
        clean_run_id = validate_id(run_id, "run_id")
        query = (
            "SELECT * FROM agent_execution WHERE run_id = type::record('workflow_run', $run_id);"
        )
        res = await self.db.execute_query(query, vars={"run_id": clean_run_id})
        if isinstance(res, list) and res:
            items = []
            for item in res:
                if isinstance(item, dict) and item.get("result"):
                    for r in item["result"]:
                        row = dict(r)
                        row["id"] = row.get("id", "").replace("agent_execution:", "")
                        if "run_id" in row and isinstance(row["run_id"], str):
                            row["run_id"] = row["run_id"].replace("workflow_run:", "")
                        items.append(AgentExecution.model_validate(row))
            return items
        return []

    async def save_delegation(self, delegation: Delegation) -> None:
        parent = validate_id(delegation.parent_execution_id, "parent_execution_id")
        child = validate_id(delegation.child_execution_id, "child_execution_id")
        clean_run_id = validate_id(delegation.run_id, "run_id")
        clean_del_id = validate_id(delegation.id, "delegation_id")

        check_q = "SELECT * FROM delegated_to WHERE delegation_id = $del_id AND run_id = type::record('workflow_run', $run_id);"
        existing = await self.db.execute_query(
            check_q, vars={"del_id": clean_del_id, "run_id": clean_run_id}
        )
        has_edge = bool(isinstance(existing, list) and existing and existing[-1].get("result"))

        if not has_edge:
            create_q = """
            LET $parent_rec = type::record('agent_execution', $parent_id);
            LET $child_rec = type::record('agent_execution', $child_id);
            LET $run_rec = type::record('workflow_run', $run_id);
            LET $edge_data = {
                delegation_id: $delegation_id,
                run_id: $run_rec,
                parent_execution_id: $parent_id,
                child_execution_id: $child_id,
                parent_agent_id: $parent_agent_id,
                child_agent_id: $child_agent_id,
                delegation_task_id: $delegation_task_id,
                objective: $objective,
                reason: $reason,
                depth: $depth,
                status: $status,
                input_artifact_ids: $input_artifact_ids,
                output_artifact_ids: $output_artifact_ids,
                created_at: time::now()
            };
            RELATE $parent_rec -> delegated_to -> $child_rec CONTENT $edge_data;
            """
            create_vars = {
                "parent_id": parent,
                "child_id": child,
                "run_id": clean_run_id,
                "delegation_id": clean_del_id,
                "parent_agent_id": delegation.parent_agent_id,
                "child_agent_id": delegation.child_agent_id,
                "delegation_task_id": delegation.delegation_task_id,
                "objective": delegation.objective,
                "reason": delegation.reason,
                "depth": delegation.depth,
                "status": delegation.status,
                "input_artifact_ids": delegation.input_artifact_ids,
                "output_artifact_ids": delegation.output_artifact_ids,
            }
            await self.db.execute_query(create_q, vars=create_vars)
        else:
            update_q = """
            UPDATE delegated_to SET
                status = $status,
                output_artifact_ids = $output_artifact_ids,
                error = $error,
                completed_at = (IF $status IN ['completed', 'failed', 'rejected'] THEN time::now() ELSE completed_at END)
            WHERE delegation_id = $delegation_id AND run_id = type::record('workflow_run', $run_id);
            """
            update_vars = {
                "status": delegation.status,
                "output_artifact_ids": delegation.output_artifact_ids,
                "error": sanitize_data(delegation.error) if delegation.error else None,
                "delegation_id": clean_del_id,
                "run_id": clean_run_id,
            }
            await self.db.execute_query(update_q, vars=update_vars)

    async def list_delegations_for_run(self, run_id: str) -> list[Delegation]:
        clean_run_id = validate_id(run_id, "run_id")
        query = "SELECT * FROM delegated_to WHERE run_id = type::record('workflow_run', $run_id);"
        res = await self.db.execute_query(query, vars={"run_id": clean_run_id})
        delegations = []
        if isinstance(res, list) and res:
            for item in res:
                if isinstance(item, dict) and item.get("result"):
                    for r in item["result"]:
                        if isinstance(r, dict):
                            row = dict(r)
                            row["id"] = row.get("delegation_id") or row.get("id", "").replace(
                                "delegated_to:", "del_"
                            )
                            if "run_id" in row and isinstance(row["run_id"], str):
                                row["run_id"] = row["run_id"].replace("workflow_run:", "")
                            row.pop("in", None)
                            row.pop("out", None)
                            row.pop("delegation_id", None)
                            delegations.append(Delegation.model_validate(row))
        return delegations

    async def save_tool_invocation(
        self, invocation: ToolInvocation, run_id: str | None = None
    ) -> None:
        clean_id = validate_id(invocation.id, "tool_id")
        clean_exec_id = validate_id(invocation.execution_id, "execution_id")

        query = """
        UPSERT type::record('tool_invocation', $tool_id) SET
            execution_id = $execution_id,
            tool_name = $tool_name,
            status = $status,
            duration_ms = $duration_ms,
            sanitized_input = $sanitized_input,
            output_summary = $output_summary,
            started_at = started_at ?? time::now();
        """
        vars_payload = {
            "tool_id": clean_id,
            "execution_id": clean_exec_id,
            "tool_name": invocation.tool_name,
            "status": invocation.status,
            "duration_ms": invocation.duration_ms,
            "sanitized_input": invocation.sanitized_input,
            "output_summary": invocation.output_summary,
        }
        if invocation.status in ("completed", "failed"):
            query = query.replace("started_at = ", "completed_at = time::now(), started_at = ")
        if invocation.error is not None:
            query = query.replace("started_at = ", "error = $error, started_at = ")
            vars_payload["error"] = sanitize_data(invocation.error)
        if run_id:
            clean_run_id = validate_id(run_id, "run_id")
            query = query.replace(
                "execution_id = $execution_id,",
                "run_id = type::record('workflow_run', $run_id), execution_id = $execution_id,",
            )
            vars_payload["run_id"] = clean_run_id
        await self.db.execute_query(query, vars=vars_payload)

    async def get_tool_invocation(self, run_id: str, tool_id: str) -> ToolInvocation | None:
        clean_id = validate_id(tool_id, "tool_id")
        clean_run_id = validate_id(run_id, "run_id")
        query = "SELECT * FROM type::record('tool_invocation', $tool_id) WHERE run_id = type::record('workflow_run', $run_id) OR run_id IS NONE;"
        res = await self.db.execute_query(query, vars={"tool_id": clean_id, "run_id": clean_run_id})
        if isinstance(res, list) and res:
            for item in res:
                if isinstance(item, dict) and item.get("result"):
                    rows = item["result"]
                    if rows and isinstance(rows, list):
                        row = dict(rows[0])
                        row["id"] = row.get("id", "").replace("tool_invocation:", "")
                        row.pop("run_id", None)
                        return ToolInvocation.model_validate(row)
        return None

    async def list_tools_for_execution(
        self, execution_id: str, cursor: str | None = None, limit: int = 50
    ) -> ToolInvocationsPageResponse:
        clean_id = validate_id(execution_id, "execution_id")
        query = "SELECT * FROM tool_invocation WHERE execution_id = $execution_id OR execution_id = string::concat('agent_execution:', $execution_id) LIMIT $limit;"
        res = await self.db.execute_query(query, vars={"execution_id": clean_id, "limit": limit})
        items: list[ToolInvocation] = []
        if isinstance(res, list) and res:
            for item in res:
                if isinstance(item, dict) and item.get("result"):
                    for row in item["result"]:
                        r = dict(row)
                        r["id"] = r.get("id", "").replace("tool_invocation:", "")
                        r.pop("run_id", None)
                        items.append(ToolInvocation.model_validate(r))
        return ToolInvocationsPageResponse(items=items, next_cursor=None)

    async def sweep_run_to_terminal(self, run_id: str, reason: str) -> dict[str, int]:
        clean_run_id = validate_id(run_id, "run_id")
        error = f"Closed at run end: {reason}"
        now = datetime.now(timezone.utc)
        counts = {"executions": 0, "tools": 0, "delegations": 0}

        tool_res = await self.db.execute_query(
            "SELECT * FROM tool_invocation WHERE run_id = type::record('workflow_run', $run_id) "
            "AND status IN ['pending', 'running'];",
            vars={"run_id": clean_run_id},
        )
        for row in extract_rows(tool_res):
            r = dict(row)
            r["id"] = str(r.get("id", "")).replace("tool_invocation:", "")
            r.pop("run_id", None)
            inv = ToolInvocation.model_validate(r)
            inv.status = "failed"
            inv.error = error
            await self.save_tool_invocation(inv, run_id=clean_run_id)
            await self.append_event_transactional(
                run_id=clean_run_id,
                event_type="tool.failed",
                execution_id=inv.execution_id,
                tool_invocation_id=inv.id,
                payload={"tool_invocation_id": inv.id, "tool_name": inv.tool_name, "error": error},
            )
            counts["tools"] += 1

        for delegation in await self.list_delegations_for_run(clean_run_id):
            if delegation.status in ("completed", "failed", "rejected"):
                continue
            delegation.status = "failed"
            delegation.error = error
            await self.save_delegation(delegation)
            await self.append_event_transactional(
                run_id=clean_run_id,
                event_type="delegation.failed",
                execution_id=delegation.parent_execution_id,
                payload={"delegation_id": delegation.id, "error": error},
            )
            counts["delegations"] += 1

        for execution in await self.list_agent_executions_for_run(clean_run_id):
            if execution.status not in ("pending", "running"):
                continue
            execution.status = "cancelled"
            execution.error = error
            execution.completed_at = now
            await self.save_agent_execution(execution)
            await self.append_event_transactional(
                run_id=clean_run_id,
                event_type="agent.failed",
                task_id=execution.task_id,
                execution_id=execution.id,
                payload={"execution_id": execution.id, "error": error},
            )
            counts["executions"] += 1
        if any(counts.values()):
            logger.warning(f"Run {clean_run_id} end sweep closed non-terminal records: {counts}")
        return counts

    async def save_artifact(self, artifact: Artifact) -> None:
        clean_id = validate_id(artifact.id, "artifact_id")
        clean_run_id = validate_id(artifact.run_id, "run_id")
        query = """
        UPSERT type::record('artifact', $art_id) SET
            run_id = type::record('workflow_run', $run_id),
            task_id = $task_id,
            agent_execution_id = $agent_execution_id,
            type = $type,
            title = $title,
            version = $version,
            evidence_ids = $evidence_ids,
            source_links = $source_links,
            content = $content,
            created_at = time::now();
        """
        vars_payload = {
            "art_id": clean_id,
            "run_id": clean_run_id,
            "task_id": artifact.task_id or "",
            "agent_execution_id": artifact.agent_execution_id or "",
            "type": artifact.type,
            "title": artifact.title,
            "version": artifact.version,
            "evidence_ids": artifact.evidence_ids,
            "source_links": [sl.model_dump() for sl in artifact.source_links],
            "content": artifact.content
            if isinstance(artifact.content, dict)
            else {"text": artifact.content},
        }
        if artifact.surreal_record_id:
            query = query.replace(
                "created_at = time::now();",
                "surreal_record_id = $surreal_record_id, created_at = time::now();",
            )
            vars_payload["surreal_record_id"] = artifact.surreal_record_id
        if artifact.evidence:
            query = query.replace(
                "created_at = time::now();", "evidence = $evidence, created_at = time::now();"
            )
            vars_payload["evidence"] = artifact.evidence.model_dump()
        await self.db.execute_query(query, vars=vars_payload)

    async def list_artifacts_for_run(self, run_id: str) -> list[Artifact]:
        clean_run_id = validate_id(run_id, "run_id")
        query = "SELECT * FROM artifact WHERE run_id = type::record('workflow_run', $run_id);"
        res = await self.db.execute_query(query, vars={"run_id": clean_run_id})
        items = []
        if isinstance(res, list) and res:
            for item in res:
                if isinstance(item, dict) and item.get("result"):
                    for r in item["result"]:
                        row = dict(r)
                        row["id"] = row.get("id", "").replace("artifact:", "")
                        if "run_id" in row and isinstance(row["run_id"], str):
                            row["run_id"] = row["run_id"].replace("workflow_run:", "")
                        items.append(Artifact.model_validate(row))
        return items

    async def get_task_graph(self, run_id: str) -> TaskGraphResponse:
        tasks = await self.list_tasks_for_run(run_id)
        nodes: list[TaskGraphNode] = []
        edges: list[TaskGraphEdge] = []

        for t in tasks:
            node = TaskGraphNode(
                id=t.id,
                label=t.name,
                data=TaskGraphNodeData(
                    status=t.status,
                    assigned_agent=t.assigned_agent,
                    attempt=t.attempt,
                    task_type=t.task_type,
                    required=t.required,
                    added_in_revision=t.added_in_revision,
                    execution_ids=t.execution_ids,
                    group=t.task_type,
                ),
            )
            nodes.append(node)
            for dep in t.dependencies:
                edges.append(
                    TaskGraphEdge(
                        id=f"edge_{dep}_to_{t.id}",
                        source=dep,
                        target=t.id,
                        kind="dependency",
                    )
                )

        return TaskGraphResponse(nodes=nodes, edges=edges)

    async def get_delegation_graph(self, run_id: str) -> DelegationGraphResponse:
        delegations = await self.list_delegations_for_run(run_id)
        nodes: list[DelegationGraphNode] = []
        edges: list[DelegationGraphEdge] = []
        seen_agents: set[str] = set()

        for d in delegations:
            if d.parent_execution_id not in seen_agents:
                seen_agents.add(d.parent_execution_id)
                nodes.append(
                    DelegationGraphNode(
                        id=d.parent_execution_id,
                        label=d.parent_agent_id,
                        data=DelegationGraphNodeData(
                            status="completed",
                            task_id=d.delegation_task_id,
                            agent_id=d.parent_agent_id,
                        ),
                    )
                )
            if d.child_execution_id not in seen_agents:
                seen_agents.add(d.child_execution_id)
                nodes.append(
                    DelegationGraphNode(
                        id=d.child_execution_id,
                        label=d.child_agent_id,
                        data=DelegationGraphNodeData(
                            status=d.status
                            if d.status
                            in ("pending", "running", "completed", "failed", "cancelled")
                            else "completed",
                            task_id=d.delegation_task_id,
                            agent_id=d.child_agent_id,
                        ),
                    )
                )

            edges.append(
                DelegationGraphEdge(
                    id=f"del_edge_{d.id}",
                    source=d.parent_execution_id,
                    target=d.child_execution_id,
                    kind="delegation",
                    data=DelegationGraphEdgeData(
                        delegation_id=d.id,
                        objective=d.objective,
                        status=d.status,
                        created_at=d.created_at,
                        parent_agent_id=d.parent_agent_id,
                        child_agent_id=d.child_agent_id,
                    ),
                )
            )
        return DelegationGraphResponse(nodes=nodes, edges=edges)

    async def append_execution_log(
        self,
        run_id: str,
        message: str,
        severity: str = "info",
        component: str = "orchestration",
        task_id: str | None = None,
        execution_id: str | None = None,
        tool_invocation_id: str | None = None,
        duration_ms: float | None = None,
        error_category: str | None = None,
        attributes: dict[str, Any] | None = None,
        seq: int | None = None,
    ) -> None:
        clean_run_id = validate_id(run_id, "run_id")
        clean_msg = sanitize_data(message)
        clean_attrs = sanitize_data(attributes or {})

        query = """
        CREATE type::record('execution_log', $log_id) SET
            run_id = type::record('workflow_run', $run_id),
            message = $message,
            severity = $severity,
            component = $component,
            attributes = $attributes,
            ts = time::now();
        """
        vars_payload = {
            "log_id": time.time_ns() // 1000 * 1000 + uuid.uuid4().int % 1000,
            "run_id": clean_run_id,
            "message": clean_msg,
            "severity": severity if severity in ("debug", "info", "warn", "error") else "info",
            "component": component,
            "attributes": clean_attrs,
        }
        if seq is not None:
            query = query.replace("ts = time::now();", "seq = $seq, ts = time::now();")
            vars_payload["seq"] = int(seq)
        if task_id:
            query = query.replace("ts = time::now();", "task_id = $task_id, ts = time::now();")
            vars_payload["task_id"] = task_id
        if execution_id:
            query = query.replace("ts = time::now();", "execution_id = $execution_id, ts = time::now();")
            vars_payload["execution_id"] = execution_id
        if tool_invocation_id:
            query = query.replace("ts = time::now();", "tool_invocation_id = $tool_invocation_id, ts = time::now();")
            vars_payload["tool_invocation_id"] = tool_invocation_id
        if duration_ms is not None:
            query = query.replace("ts = time::now();", "duration_ms = $duration_ms, ts = time::now();")
            vars_payload["duration_ms"] = float(duration_ms)
        if error_category:
            query = query.replace("ts = time::now();", "error_category = $error_category, ts = time::now();")
            vars_payload["error_category"] = error_category

        await self.db.execute_query(query, vars=vars_payload)

    async def get_logs_page(
        self,
        run_id: str,
        cursor: str | None = None,
        limit: int = 50,
        severity: str | None = None,
        task_id: str | None = None,
        execution_id: str | None = None,
    ) -> LogsPageResponse:
        clean_run_id = validate_id(run_id, "run_id")
        conditions = ["run_id = type::record('workflow_run', $run_id)"]
        vars_payload: dict[str, Any] = {"run_id": clean_run_id, "limit": limit + 1}

        if severity:
            conditions.append("severity = $severity")
            vars_payload["severity"] = severity
        if task_id:
            conditions.append("task_id = $task_id")
            vars_payload["task_id"] = task_id
        if execution_id:
            conditions.append("execution_id = $execution_id")
            vars_payload["execution_id"] = execution_id
        if cursor:
            conditions.append("id > type::record('execution_log', $cursor)")
            vars_payload["cursor"] = int(cursor)

        where_clause = " AND ".join(conditions)
        query = "SELECT * FROM execution_log WHERE " + where_clause + " ORDER BY id ASC LIMIT $limit;"
        res = await self.db.execute_query(query, vars=vars_payload)
        items: list[LogEntry] = []
        if isinstance(res, list) and res:
            for item in res:
                if isinstance(item, dict) and item.get("result"):
                    for idx, r in enumerate(item["result"]):
                        row = dict(r)
                        raw_id = row.get("id", idx + 1)
                        if isinstance(raw_id, str):
                            str_id = raw_id.replace("execution_log:", "")
                        else:
                            str_id = str(raw_id)
                        row["id"] = str_id
                        if "run_id" in row and isinstance(row["run_id"], str):
                            row["run_id"] = row["run_id"].replace("workflow_run:", "")
                        items.append(LogEntry.model_validate(row))

        next_cursor = None
        if len(items) > limit:
            items = items[:limit]
            next_cursor = str(items[-1].id)

        return LogsPageResponse(items=items, next_cursor=next_cursor)
