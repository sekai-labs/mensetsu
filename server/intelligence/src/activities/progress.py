import datetime
import uuid

from temporalio import activity

from src.activities.persistence import persist_record
from src.db.client import SurrealDirectClient

db_client = SurrealDirectClient()



async def log_pipeline_step(
    workspace_id: str,
    step: str,
    level: str,
    message: str,
    details: dict | None = None,
    execution_id: str | None = None,
) -> None:
    clean_ws = str(workspace_id).replace("workspace:", "").replace("`", "").strip()
    exec_id = execution_id or f"exec_{clean_ws}"
    log_id = f"log_{uuid.uuid4().hex[:12]}"
    now_str = datetime.datetime.now(datetime.timezone.utc).isoformat()
    await persist_record(
        db_client=db_client,
        table="pipeline_log",
        record_id=log_id,
        fields={
            "workspace_id": clean_ws,
            "execution_id": exec_id,
            "step": step,
            "level": level,
            "message": message,
            "details": details or {},
            "timestamp": now_str,
        },
    )

@activity.defn
async def update_workspace_progress(
    workspace_id: str,
    phase: str,
    percent: int,
    error_message: str | None = None,
) -> None:
    clean_ws = str(workspace_id).replace("workspace:", "").replace("`", "").strip()
    status = "ready" if (phase == "ready" or percent >= 100) else ("failed" if phase == "failed" else "processing")
    query = f"""
    UPDATE type::record('workspace', '{clean_ws}') MERGE {{
        current_phase: '{phase}',
        progress_percent: {percent},
        status: '{status}',
        updated_at: time::now()
    }};
    """
    await db_client.execute_query(query)

    exec_id = f"exec_{clean_ws}"
    now_str = datetime.datetime.now(datetime.timezone.utc).isoformat()
    step_order = [
        "preparing_input",
        "analyzing_resume",
        "reading_job_description",
        "researching_company",
        "mapping_resume_evidence",
        "constructing_knowledge_graph",
        "generating_lessons",
        "building_interview_question_book",
        "ready",
    ]
    completed_count = step_order.index(phase) if phase in step_order else (8 if phase == "ready" else 0)
    exec_fields = {
        "workspace_id": clean_ws,
        "status": status,
        "current_step": phase,
        "completed_steps": completed_count,
        "total_steps": 8,
        "error_message": error_message,
    }
    if phase == "ready" or phase == "failed":
        exec_fields["completed_at"] = now_str
    await persist_record(
        db_client=db_client,
        table="pipeline_execution",
        record_id=exec_id,
        fields=exec_fields,
    )
