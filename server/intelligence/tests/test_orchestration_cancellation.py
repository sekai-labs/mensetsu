import asyncio
import os
import uuid

import pytest
import pytest_asyncio

from src.contracts.v1 import WorkflowRun, WorkflowTask
from src.orchestration import service as service_module
from src.orchestration.ledger import SurrealLedgerRepository
from src.orchestration.redis_queue import RedisStreamJobQueue
from src.orchestration.runtime import worker as worker_module
from src.orchestration.runtime.worker import DurableJobWorker
from src.orchestration.service import WorkflowService


def make_run(prefix: str, status: str = "running") -> WorkflowRun:
    run_id = f"run_{prefix}_{uuid.uuid4().hex[:8]}"
    return WorkflowRun(
        id=run_id,
        workflow_id="interview-preparation-static",
        workflow_name="Guard Test",
        workspace_id=f"ws_{prefix}_{uuid.uuid4().hex[:8]}",
        user_id=f"usr_{prefix}",
        type="static",
        status=status,
    )


def make_task(run: WorkflowRun, task_id: str, status: str = "running") -> WorkflowTask:
    return WorkflowTask(
        id=task_id,
        run_id=run.id,
        task_type="research",
        name="Task",
        assigned_agent="company-research",
        status=status,
    )


def job_payload(run: WorkflowRun) -> dict:
    return {
        "run_id": run.id,
        "workflow_id": "interview-preparation-static",
        "workspace_id": run.workspace_id,
        "user_id": run.user_id,
        "company_name": "Acme",
        "role_title": "Engineer",
        "resume_text": "Engineer resume",
        "job_description": "Build systems",
    }


@pytest_asyncio.fixture
async def queue():
    suffix = uuid.uuid4().hex
    q = RedisStreamJobQueue(
        redis_url=os.environ.get("REDIS_URL", "redis://127.0.0.1:16379"),
        stream_key=f"test_cancel_stream_{suffix}",
        group_name=f"test_cancel_group_{suffix}",
    )
    yield q
    await q.close()


class BlockingGraph:
    def __init__(self):
        self.started = asyncio.Event()
        self.cancelled = False
        self.calls = 0

    async def ainvoke(self, input_value, config):
        self.calls += 1
        self.started.set()
        try:
            await asyncio.sleep(60)
        except asyncio.CancelledError:
            self.cancelled = True
            raise
        return {}


@pytest.mark.asyncio
async def test_run_and_task_terminal_statuses_are_immutable():
    ledger = SurrealLedgerRepository()
    run = make_run("immutable", status="running")
    await ledger.create_workflow_run(run)

    assert await ledger.update_workflow_run(run.id, {"status": "completed"}) is True
    assert await ledger.update_workflow_run(run.id, {"status": "failed"}) is False
    assert await ledger.update_workflow_run(run.id, {"status": "cancelled"}) is False
    assert (
        await ledger.transition_workflow_run_status(
            run.id, "failed", from_statuses=("running",)
        )
        is False
    )
    assert (await ledger.get_workflow_run(run.id)).status == "completed"

    task = make_task(run, "task_immutable")
    await ledger.save_task(task)
    task.status = "completed"
    await ledger.save_task(task)
    task.status = "failed"
    await ledger.save_task(task)
    assert (await ledger.get_task(run.id, task.id)).status == "completed"

    cancel_task = make_task(run, "task_cancel_immutable")
    await ledger.save_task(cancel_task)
    cancel_task.status = "cancelled"
    await ledger.save_task(cancel_task)
    cancel_task.status = "completed"
    await ledger.save_task(cancel_task)
    assert (await ledger.get_task(run.id, cancel_task.id)).status == "cancelled"


@pytest.mark.asyncio
async def test_cancel_run_is_authoritative_and_closes_tasks_once(queue, monkeypatch):
    ledger = SurrealLedgerRepository()
    monkeypatch.setattr(service_module, "job_queue", queue)
    monkeypatch.setattr(service_module, "get_ledger", lambda: ledger)

    run = make_run("authoritative", status="running")
    await ledger.create_workflow_run(run)
    await ledger.save_task(make_task(run, "task_cancel_once", status="running"))

    cancelled = await WorkflowService.cancel_run(run.id)
    assert cancelled.status == "cancelled"
    assert (await ledger.get_workflow_run(run.id)).status == "cancelled"
    assert (await ledger.get_task(run.id, "task_cancel_once")).status == "cancelled"
    assert await queue.is_cancel_requested(run.id)

    again = await WorkflowService.cancel_run(run.id)
    assert again.status == "cancelled"

    history = await ledger.get_events_history(run_id=run.id, limit=100)
    assert [event.type for event in history.events].count("workflow.cancelled") == 1
    assert (await ledger.get_task(run.id, "task_cancel_once")).status == "cancelled"


@pytest.mark.asyncio
async def test_worker_cancellation_aborts_graph_waits_and_acks_once(queue, monkeypatch):
    ledger = SurrealLedgerRepository()
    run = make_run("worker_cancel", status="pending")
    await ledger.create_workflow_run(run)
    await ledger.save_task(make_task(run, "task_worker_cancel", status="running"))

    graph = BlockingGraph()
    monkeypatch.setattr(
        worker_module, "build_static_workflow", lambda **kwargs: graph
    )
    monkeypatch.setattr(service_module, "job_queue", queue)
    monkeypatch.setattr(service_module, "get_ledger", lambda: ledger)

    worker = DurableJobWorker(
        worker_id=f"worker_cancel_{uuid.uuid4().hex[:8]}",
        heartbeat_interval_s=0.2,
        lease_ttl_s=5.0,
        min_idle_time_ms=5200,
    )
    worker.queue = queue
    worker.ledger = ledger
    await worker.initialize()

    await queue.enqueue_job(run.id, job_payload(run))
    job = await queue.claim_job(worker.worker_id, block_ms=500, min_idle_time=100)
    assert job is not None

    execution = asyncio.create_task(worker.execute_job(job))
    await asyncio.wait_for(graph.started.wait(), timeout=5.0)

    await WorkflowService.cancel_run(run.id)
    await asyncio.wait_for(execution, timeout=5.0)

    assert graph.cancelled
    refreshed = await ledger.get_workflow_run(run.id)
    assert refreshed.status == "cancelled"
    assert (await ledger.get_task(run.id, "task_worker_cancel")).status == "cancelled"

    client = await queue.get_client()
    pending = await client.xpending_range(
        queue.stream_key, queue.group_name, job["msg_id"], job["msg_id"], 1
    )
    assert pending == []

    history = await ledger.get_events_history(run_id=run.id, limit=100)
    assert [event.type for event in history.events].count("workflow.cancelled") == 1
    await worker.stop()


@pytest.mark.asyncio
async def test_lost_lease_never_acknowledges_or_mutates_run(queue, monkeypatch):
    ledger = SurrealLedgerRepository()
    run = make_run("lease_guard", status="running")
    await ledger.create_workflow_run(run)
    await ledger.save_task(make_task(run, "task_lease_guard", status="running"))

    graph = BlockingGraph()
    monkeypatch.setattr(
        worker_module, "build_static_workflow", lambda **kwargs: graph
    )

    worker = DurableJobWorker(
        worker_id=f"worker_lease_{uuid.uuid4().hex[:8]}",
        heartbeat_interval_s=0.2,
        lease_ttl_s=2.0,
        min_idle_time_ms=2200,
    )
    worker.queue = queue
    worker.ledger = ledger
    await worker.initialize()

    await queue.enqueue_job(run.id, job_payload(run))
    job = await queue.claim_job(worker.worker_id, block_ms=500, min_idle_time=100)
    assert job is not None

    execution = asyncio.create_task(worker.execute_job(job))
    await asyncio.wait_for(graph.started.wait(), timeout=5.0)

    client = await queue.get_client()
    assert await client.delete(f"mensetsu:lease:{run.id}") == 1
    await asyncio.wait_for(execution, timeout=5.0)

    assert graph.cancelled
    refreshed = await ledger.get_workflow_run(run.id)
    assert refreshed.status == "running"
    assert (await ledger.get_task(run.id, "task_lease_guard")).status == "running"

    history = await ledger.get_events_history(run_id=run.id, limit=100)
    event_types = [event.type for event in history.events]
    assert "workflow.failed" not in event_types
    assert "workflow.completed" not in event_types
    assert "workflow.cancelled" not in event_types

    pending = await client.xpending_range(
        queue.stream_key, queue.group_name, job["msg_id"], job["msg_id"], 1
    )
    assert len(pending) == 1
    await worker.stop()


def test_worker_idle_threshold_validation_and_default():
    with pytest.raises(ValueError):
        DurableJobWorker(min_idle_time_ms=-1)
    with pytest.raises(ValueError):
        DurableJobWorker(min_idle_time_ms="not-a-number")

    defaulted = DurableJobWorker(lease_ttl_s=7.0)
    assert defaulted.min_idle_time_ms == 12000
    with pytest.raises(ValueError, match="at least"):
        DurableJobWorker(lease_ttl_s=7.0, heartbeat_interval_s=5.0, min_idle_time_ms=11999)

    queue = RedisStreamJobQueue()
    with pytest.raises(ValueError):
        asyncio.run(queue.claim_job("worker", min_idle_time=-1))


class FinalizerLedger:
    def __init__(self, run):
        self.run = run
        self.transitions = []
        self.events = []

    async def get_workflow_run(self, run_id):
        return self.run

    async def list_tasks_for_run(self, run_id):
        return []

    async def list_artifacts_for_run(self, run_id):
        return []

    async def transition_workflow_run_status(self, *args, **kwargs):
        self.transitions.append((args, kwargs))
        return True

    async def append_event_transactional(self, **kwargs):
        self.events.append(kwargs)


@pytest.mark.asyncio
async def test_hybrid_finalizer_does_not_complete_cancelled_run():
    from src.orchestration.hybrid_outputs import build_hybrid_output_nodes
    from src.orchestration.hybrid_support import HybridWorkflowState

    run = make_run("finalizer", status="cancelled")
    ledger = FinalizerLedger(run)

    async def set_run_phase(*args, **kwargs):
        return None

    nodes = build_hybrid_output_nodes(ledger, set_run_phase)
    result = await nodes["finalize_node"](
        HybridWorkflowState(
            run_id=run.id,
            workspace_id=run.workspace_id,
            company_name="Acme",
            role_title="Engineer",
        )
    )

    assert result == {}
    assert ledger.transitions == []
    assert ledger.events == []

