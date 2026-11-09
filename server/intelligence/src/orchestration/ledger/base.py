"""Shared primitives for the SurrealDB ledger repository."""
import asyncio
import logging
import random
from typing import Any

from src.db.client import SurrealDirectClient, SurrealQueryError

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
                    logger.error(
                        f"{description}: write conflict persisted after {attempts} attempts: {e}"
                    )
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


class IdempotencyConflict(Exception):
    """An idempotency key was reused for a different create-run request."""

LOGGED_EVENT_DOMAINS = {"workflow", "task", "agent", "tool", "delegation"}


class RepositoryBase:
    def __init__(self, db_client: SurrealDirectClient | None = None):
        self.db = db_client or SurrealDirectClient()
