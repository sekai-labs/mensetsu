"""Execution event append, history, live publication, and log queries."""
import logging
import time
import uuid
from collections.abc import Awaitable, Callable
from typing import Any

from pydantic import TypeAdapter

from src.contracts.v1 import (
    EventsHistoryPageResponse,
    ExecutionEvent,
    LogEntry,
    LogsPageResponse,
)
from src.orchestration.id_validator import validate_id
from src.orchestration.ledger.base import (
    LOGGED_EVENT_DOMAINS,
    retry_on_write_conflict,
)
from src.orchestration.sanitizer import sanitize_data

logger = logging.getLogger("mensetsu.orchestration.ledger")


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
        logger.warning(
            f"Live publish failed for {evt.type} seq={evt.seq} (history replay still has it): {pe}"
        )


class EventMixin:
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
        raw_payload = payload.model_dump() if hasattr(payload, "model_dump") else dict(payload)
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
                "UPSERT type::record('execution_event', $rec_id) SET "
                + set_items_str
                + " RETURN AFTER;\n"
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
            query = query.replace(
                "ts = time::now();", "execution_id = $execution_id, ts = time::now();"
            )
            vars_payload["execution_id"] = execution_id
        if tool_invocation_id:
            query = query.replace(
                "ts = time::now();", "tool_invocation_id = $tool_invocation_id, ts = time::now();"
            )
            vars_payload["tool_invocation_id"] = tool_invocation_id
        if duration_ms is not None:
            query = query.replace(
                "ts = time::now();", "duration_ms = $duration_ms, ts = time::now();"
            )
            vars_payload["duration_ms"] = float(duration_ms)
        if error_category:
            query = query.replace(
                "ts = time::now();", "error_category = $error_category, ts = time::now();"
            )
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
        query = (
            "SELECT * FROM execution_log WHERE " + where_clause + " ORDER BY id ASC LIMIT $limit;"
        )
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
