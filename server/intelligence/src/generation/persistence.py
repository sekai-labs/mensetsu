import json
import logging
from typing import Any

from src.db.client import SurrealDirectClient

logger = logging.getLogger("mensetsu.activities.persistence")


def sanitize_surreal_value(val: Any) -> str:
    if isinstance(val, (dict, list)):
        dumped = json.dumps(val)
        return dumped.replace("\\", "\\\\").replace("'", "\\'")
    elif isinstance(val, str):
        return val.replace("\\", "\\\\").replace("'", "\\'")
    elif val is None:
        return "NONE"
    elif isinstance(val, bool):
        return "true" if val else "false"
    return str(val)


async def persist_record(
    db_client: SurrealDirectClient, table: str, record_id: str, fields: dict[str, Any]
) -> None:
    clean_rec_id = str(record_id).replace(f"{table}:", "").replace("`", "").strip()
    field_assignments = []
    vars_payload: dict[str, Any] = {"rec_id": clean_rec_id}
    for k, v in fields.items():
        if k == "workspace_id":
            clean_ws = str(v).replace("workspace:", "").replace("`", "").strip()
            field_assignments.append("workspace_id = type::record('workspace', $clean_ws)")
            vars_payload["clean_ws"] = clean_ws
        elif k == "source_id":
            clean_src = str(v).replace("source:", "").replace("`", "").strip()
            field_assignments.append("source_id = type::record('source', $clean_src)")
            vars_payload["clean_src"] = clean_src
        elif k in [
            "timestamp",
            "created_at",
            "updated_at",
            "completed_at",
            "started_at",
            "retrieved_at",
        ] and isinstance(v, str):
            field_assignments.append(f"{k} = type::datetime(${k})")
            vars_payload[k] = v
        else:
            field_assignments.append(f"{k} = ${k}")
            vars_payload[k] = v
    set_clause = ",\n        ".join(field_assignments)
    query = f"\n    UPSERT type::record('{table}', $rec_id) SET\n        {set_clause};\n    "
    try:
        await db_client.execute_query(query, vars=vars_payload)
    except Exception as e:
        logger.error(f"SurrealDB persistence error on {table}:{clean_rec_id}: {e}")
        raise
