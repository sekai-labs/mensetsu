import asyncio
import time
import uuid

import pytest

from src.contracts.v1 import WorkflowRun
from src.orchestration import ledger as ledger_module
from src.orchestration.ledger import SurrealLedgerRepository


class ConnectedRequest:
    async def is_disconnected(self) -> bool:
        return False


async def create_run_with_history(ledger: SurrealLedgerRepository) -> str:
    run_id = f"run_sse_{uuid.uuid4().hex[:8]}"
    await ledger.create_workflow_run(
        WorkflowRun(
            id=run_id,
            workflow_id="interview-preparation-static",
            workflow_name="SSE live",
            workspace_id=f"ws_{run_id}",
            user_id="usr_sse",
            type="static",
            status="running",
        )
    )
    await ledger.append_event_transactional(
        run_id=run_id,
        event_type="workflow.created",
        payload={
            "workflow_id": "interview-preparation-static",
            "workspace_id": f"ws_{run_id}",
            "user_id": "usr_sse",
            "type": "static",
        },
    )
    await ledger.append_event_transactional(
        run_id=run_id, event_type="workflow.started", payload={"plan_revision": 1, "initial_tasks_count": 9}
    )
    return run_id


def start_reader(run_id: str, received: list[tuple[int, str, float]]) -> asyncio.Task:
    from src.orchestration.router import stream_events

    async def read_stream():
        response = await stream_events(ConnectedRequest(), run_id=run_id, after_seq=None, last_event_id=None)
        seq = None
        async for chunk in response.body_iterator:
            for line in chunk.splitlines():
                if line.startswith("id: "):
                    seq = int(line[4:])
                elif line.startswith("event: ") and seq is not None:
                    received.append((seq, line[7:], time.monotonic()))
                    if line[7:] == "workflow.completed":
                        return

    return asyncio.create_task(read_stream())


async def wait_for_count(received: list, count: int, timeout_s: float) -> None:
    deadline = time.monotonic() + timeout_s
    while len(received) < count and time.monotonic() < deadline:
        await asyncio.sleep(0.02)


async def append_task_started(ledger: SurrealLedgerRepository, run_id: str, task_id: str) -> float:
    await ledger.append_event_transactional(
        run_id=run_id,
        event_type="task.started",
        task_id=task_id,
        payload={"task_id": task_id, "attempt": 1, "assigned_agent": "job-analysis"},
    )
    return time.monotonic()


@pytest.mark.asyncio
async def test_sse_delivers_published_events_within_two_seconds_without_duplicates():
    from src.orchestration.router import job_queue

    job_queue.client = None
    ledger_module.set_event_publisher(job_queue.publish_event)
    try:
        ledger = SurrealLedgerRepository()
        run_id = await create_run_with_history(ledger)
        received: list[tuple[int, str, float]] = []
        reader = start_reader(run_id, received)
        await wait_for_count(received, 2, 5.0)
        assert [s for s, _, _ in received] == [1, 2]
        await asyncio.sleep(0.3)

        for i in range(3):
            committed_at = await append_task_started(ledger, run_id, f"task_{i}")
            await wait_for_count(received, 3 + i, 2.0)
            assert len(received) == 3 + i, f"event seq {3 + i} not delivered within 2s"
            assert received[-1][2] - committed_at < 2.0

        await ledger.append_event_transactional(
            run_id=run_id,
            event_type="workflow.completed",
            payload={"duration_ms": 1.0, "total_tasks": 3, "artifacts_count": 0},
        )
        await asyncio.wait_for(reader, timeout=10)

        seqs = [s for s, _, _ in received]
        assert seqs == [1, 2, 3, 4, 5, 6]
        assert len(seqs) == len(set(seqs))
        assert [t for _, t, _ in received][2:] == ["task.started"] * 3 + ["workflow.completed"]
    finally:
        await job_queue.close()


@pytest.mark.asyncio
async def test_sse_backstop_replays_missed_events_when_publishing_is_disabled():
    from src.orchestration import router as router_module
    from src.orchestration.router import job_queue

    job_queue.client = None
    ledger_module.set_event_publisher(None)
    try:
        ledger = SurrealLedgerRepository()
        run_id = await create_run_with_history(ledger)
        received: list[tuple[int, str, float]] = []
        reader = start_reader(run_id, received)
        await wait_for_count(received, 2, 5.0)
        assert [s for s, _, _ in received] == [1, 2]

        await append_task_started(ledger, run_id, "task_a")
        await append_task_started(ledger, run_id, "task_b")
        await wait_for_count(received, 4, router_module.SSE_IDLE_REPLAY_S + 5.0)
        assert [s for s, _, _ in received] == [1, 2, 3, 4]

        await ledger.append_event_transactional(
            run_id=run_id,
            event_type="workflow.completed",
            payload={"duration_ms": 1.0, "total_tasks": 2, "artifacts_count": 0},
        )
        await asyncio.wait_for(reader, timeout=router_module.SSE_IDLE_REPLAY_S + 10.0)

        seqs = [s for s, _, _ in received]
        assert seqs == [1, 2, 3, 4, 5]
        assert len(seqs) == len(set(seqs))
        assert [t for _, t, _ in received][2:] == ["task.started", "task.started", "workflow.completed"]
    finally:
        ledger_module.set_event_publisher(job_queue.publish_event)
        await job_queue.close()
