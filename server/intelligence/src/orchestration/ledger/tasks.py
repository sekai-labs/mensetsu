"""Workflow task persistence."""
import logging

from src.contracts.v1 import WorkflowTask
from src.orchestration.id_validator import validate_id
from src.orchestration.sanitizer import sanitize_data

logger = logging.getLogger("mensetsu.orchestration.ledger")


class TaskMixin:
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
