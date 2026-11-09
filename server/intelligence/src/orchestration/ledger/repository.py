"""Composed SurrealDB ledger repository."""
from src.orchestration.ledger.artifacts import ArtifactMixin
from src.orchestration.ledger.base import RepositoryBase
from src.orchestration.ledger.events import EventMixin
from src.orchestration.ledger.executions import ExecutionMixin
from src.orchestration.ledger.runs import RunMixin
from src.orchestration.ledger.tasks import TaskMixin


class SurrealLedgerRepository(
    RunMixin,
    EventMixin,
    TaskMixin,
    ExecutionMixin,
    ArtifactMixin,
    RepositoryBase,
):
    """Durable workflow ledger backed by SurrealDB."""


__all__ = ["SurrealLedgerRepository"]
