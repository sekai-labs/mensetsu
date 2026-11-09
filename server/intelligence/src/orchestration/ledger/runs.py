"""Workflow run persistence and progress tracking."""
import asyncio
import uuid
from typing import Any

from pydantic import TypeAdapter

from src.contracts.v1 import ExecutionEvent, RunsPageResponse, WorkflowProgress, WorkflowRun
from src.orchestration.id_validator import validate_id
from src.orchestration.ledger.base import (
    DYNAMIC_EXPANDABLE_PHASES,
    PROGRESS_RETRY_ATTEMPTS,
    WORKFLOW_RUN_ALLOWED_FIELDS,
    IdempotencyConflict,
    extract_rows,
    retry_on_write_conflict,
)
from src.orchestration.ledger.events import (
    _describe_event,
    _event_severity,
    publish_committed_event,
)


class RunMixin:
    async def create_idempotent_workflow_run(
        self,
        run: WorkflowRun,
        idempotency_record_id: str,
        request_hash: str,
    ) -> tuple[WorkflowRun, bool]:
        """Atomically reserve a scoped request key with its run and creation event."""
        vars_payload = {
            "idempotency_record_id": idempotency_record_id,
            "request_hash": request_hash,
            "run_id": validate_id(run.id, "run_id"),
            "workspace_id": validate_id(run.workspace_id, "workspace_id"),
            "user_id": run.user_id,
            "workflow_id": run.workflow_id,
            "workflow_name": run.workflow_name,
            "type": run.type,
            "status": run.status,
            "current_phase": run.current_phase,
            "plan_revision": run.plan_revision,
            "progress": run.progress.model_dump(),
            "last_seq": 1,
            "event_id": f"evt_{uuid.uuid4().hex}",
            "event_payload": {
                "workflow_id": run.workflow_id,
                "workspace_id": run.workspace_id,
                "user_id": run.user_id,
                "type": run.type,
            },
        }
        transaction = (
            "BEGIN TRANSACTION;\n"
            "CREATE type::record('run_idempotency', $idempotency_record_id) CONTENT {"
            "request_hash: $request_hash, run_id: type::record('workflow_run', $run_id), "
            "user_id: $user_id, workspace_id: type::record('workspace', $workspace_id), "
            "operation: 'create_run', created_at: time::now()};\n"
            "CREATE type::record('workflow_run', $run_id) CONTENT {"
            "workspace_id: type::record('workspace', $workspace_id), user_id: $user_id, "
            "workflow_id: $workflow_id, workflow_name: $workflow_name, type: $type, "
            "status: $status, current_phase: $current_phase, plan_revision: $plan_revision, "
            "progress: $progress, last_seq: $last_seq, created_at: time::now(), "
            "updated_at: time::now()};\n"
            "CREATE type::record('execution_event', string::concat($run_id, '_1')) CONTENT {"
            "schema_version: '1.0', event_id: $event_id, run_id: type::record('workflow_run', $run_id), "
            "seq: 1, ts: time::now(), type: 'workflow.created', payload: $event_payload};\n"
            "COMMIT TRANSACTION;"
        )
        try:
            await self.db.execute_query(transaction, vars=vars_payload)
        except Exception:
            rows: list[dict[str, Any]] = []
            for attempt in range(5):
                result = await self.db.execute_query(
                    "SELECT * FROM type::record('run_idempotency', $record_id);",
                    vars={"record_id": idempotency_record_id},
                )
                rows = extract_rows(result)
                if rows:
                    break
                await asyncio.sleep(0.01 * (attempt + 1))
            if not rows:
                raise
            record = rows[0]
            if record.get("request_hash") != request_hash:
                raise IdempotencyConflict(
                    "Idempotency-Key was already used with a different request."
                )
            stored_run_id = record.get("run_id")
            if isinstance(stored_run_id, dict):
                stored_run_id = stored_run_id.get("id")
            if isinstance(stored_run_id, str):
                stored_run_id = stored_run_id.split(":", 1)[-1]
            if not isinstance(stored_run_id, str):
                raise RuntimeError("Idempotency reservation has an invalid run reference.")
            for attempt in range(5):
                existing = await self.get_workflow_run(stored_run_id)
                if existing is not None:
                    return existing, False
                await asyncio.sleep(0.01 * (attempt + 1))
            raise RuntimeError("Idempotency reservation has no persisted workflow run.")

        run.last_seq = 1
        event = TypeAdapter(ExecutionEvent).validate_python(
            {
                "event_id": vars_payload["event_id"],
                "run_id": run.id,
                "seq": 1,
                "type": "workflow.created",
                "payload": vars_payload["event_payload"],
            }
        )
        await publish_committed_event(event)
        try:
            await self.append_execution_log(
                run_id=run.id,
                message=_describe_event("workflow.created", vars_payload["event_payload"]),
                severity=_event_severity("workflow.created"),
                component="workflow",
                attributes={"event_type": "workflow.created", "seq": 1},
                seq=1,
            )
        except Exception as log_error:
            from src.orchestration.ledger.base import logger

            logger.warning(f"Failed to write execution log for workflow.created: {log_error}")
        return run, True
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
            attempt_refresh,
            f"progress refresh for {clean_run_id}",
            attempts=PROGRESS_RETRY_ATTEMPTS,
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
        query = (
            "SELECT * FROM workflow_run " + where_clause + " ORDER BY created_at DESC LIMIT $limit;"
        )
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
