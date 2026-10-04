import json
import logging
from typing import Any

from src.db.client import SurrealDirectClient

logger = logging.getLogger('mensetsu.activities.persistence')

def sanitize_surreal_value(val: Any) -> str:
    if isinstance(val, (dict, list)):
        dumped = json.dumps(val)
        return dumped.replace('\\', '\\\\').replace("'", "\\'")
    elif isinstance(val, str):
        return val.replace('\\', '\\\\').replace("'", "\\'")
    elif val is None:
        return 'NONE'
    elif isinstance(val, bool):
        return 'true' if val else 'false'
    return str(val)

async def persist_record(db_client: SurrealDirectClient, table: str, record_id: str, fields: dict[str, Any]) -> None:
    clean_rec_id = str(record_id).replace(f'{table}:', '').replace('`', '').strip()
    field_assignments = []
    for k, v in fields.items():
        if k == 'workspace_id':
            clean_ws = str(v).replace('workspace:', '').replace('`', '').strip()
            field_assignments.append(f"workspace_id = type::record('workspace', '{clean_ws}')")
        elif k == 'source_id':
            clean_src = str(v).replace('source:', '').replace('`', '').strip()
            field_assignments.append(f"source_id = type::record('source', '{clean_src}')")
        elif k in ['timestamp', 'created_at', 'updated_at', 'completed_at', 'started_at'] and isinstance(v, str):
            field_assignments.append(f"{k} = type::datetime('{v}')")
        elif isinstance(v, (dict, list)):
            dumped = json.dumps(v)
            field_assignments.append(f'{k} = {dumped}')
        elif isinstance(v, str):
            field_assignments.append(f"{k} = '{sanitize_surreal_value(v)}'")
        elif isinstance(v, (int, float)):
            field_assignments.append(f'{k} = {v}')
        elif isinstance(v, bool):
            field_assignments.append(f"{k} = {('true' if v else 'false')}")
        elif v is None:
            field_assignments.append(f'{k} = NONE')
    set_clause = ',\n        '.join(field_assignments)
    query = f"\n    UPSERT type::record('{table}', '{clean_rec_id}') SET\n        {set_clause};\n    "
    try:
        res = await db_client.execute_query(query)
        if isinstance(res, list) and res and res[0].get('status') == 'ERR':
            err_detail = res[0].get('result') or res[0].get('detail')
            logger.error(f'SurrealDB statement error on {table}:{clean_rec_id}: {err_detail}')
            raise ValueError(f'SurrealDB error on {table}:{clean_rec_id}: {err_detail}')
    except Exception as e:
        logger.error(f'SurrealDB persistence error on {table}:{clean_rec_id}: {e}')
        raise
