"""Agent execution, delegation, and tool invocation persistence."""
import logging
from datetime import datetime, timezone

from src.contracts.v1 import (
    AgentExecution,
    Delegation,
    ToolInvocation,
    ToolInvocationsPageResponse,
)
from src.orchestration.id_validator import validate_id
from src.orchestration.ledger.base import extract_rows
from src.orchestration.sanitizer import sanitize_data

logger = logging.getLogger("mensetsu.orchestration.ledger")


class ExecutionMixin:
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
            query = query.replace(
                "public_summary = $public_summary;",
                "public_summary = $public_summary, started_at = started_at ?? type::datetime($started_at);",
            )
            vars_payload["started_at"] = execution.started_at.isoformat()
        else:
            query = query.replace(
                "public_summary = $public_summary;",
                "public_summary = $public_summary, started_at = started_at ?? time::now();",
            )

        if execution.status in ("completed", "failed", "cancelled"):
            if execution.completed_at:
                query = query.replace(
                    "started_at = ",
                    "completed_at = type::datetime($completed_at), started_at = ",
                    1,
                )
                vars_payload["completed_at"] = execution.completed_at.isoformat()
            else:
                query = query.replace(
                    "started_at = ", "completed_at = time::now(), started_at = ", 1
                )
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
