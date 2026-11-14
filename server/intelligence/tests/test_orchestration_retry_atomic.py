import os
import uuid

import pytest
import pytest_asyncio

from src.contracts.v1 import WorkflowRun
from src.orchestration.ledger import SurrealLedgerRepository
from src.orchestration.redis_queue import RedisStreamJobQueue
from src.orchestration.runtime import worker as worker_module
from src.orchestration.runtime.worker import DurableJobWorker


class RetryableFailureGraph:
    def __init__(self):
        self.calls = 0

    async def ainvoke(self, input_value, config):
        self.calls += 1
        raise ConnectionError("transient transport failure")


@pytest_asyncio.fixture
async def queue():
    suffix = uuid.uuid4().hex
    q = RedisStreamJobQueue(
        redis_url=os.environ.get("REDIS_URL", "redis://127.0.0.1:16379"),
        stream_key=f"test_retry_stream_{suffix}",
        group_name=f"test_retry_group_{suffix}",
    )
    yield q
    await q.close()


def make_run(prefix: str) -> WorkflowRun:
    return WorkflowRun(
        id=f"run_{prefix}_{uuid.uuid4().hex[:8]}",
        workflow_id="interview-preparation-static",
        workflow_name="Retry Guard",
        workspace_id=f"ws_{prefix}_{uuid.uuid4().hex[:8]}",
        user_id=f"usr_{prefix}",
        type="static",
        status="running",
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


@pytest.mark.asyncio
async def test_failed_retry_scheduling_leaves_original_pending(queue, monkeypatch):
    ledger = SurrealLedgerRepository()
    run = make_run("retry_pending")
    await ledger.create_workflow_run(run)

    graph = RetryableFailureGraph()
    monkeypatch.setattr(
        worker_module, "build_static_workflow", lambda **kwargs: graph
    )

    worker = DurableJobWorker(
        worker_id=f"worker_retry_{uuid.uuid4().hex[:8]}",
        heartbeat_interval_s=0.2,
        lease_ttl_s=5.0,
        min_idle_time_ms=10000,
    )
    worker.queue = queue
    worker.ledger = ledger
    await worker.initialize()

    await queue.enqueue_job(run.id, job_payload(run))
    job = await queue.claim_job(worker.worker_id, block_ms=500, min_idle_time=100)
    assert job is not None
    original_length = await (await queue.get_client()).xlen(queue.stream_key)

    async def fail_to_schedule(**kwargs):
        raise RuntimeError("retry scheduling failed")

    monkeypatch.setattr(queue, "reschedule_job", fail_to_schedule)
    with pytest.raises(RuntimeError, match="retry scheduling failed"):
        await worker.execute_job(job)

    client = await queue.get_client()
    pending = await client.xpending_range(
        queue.stream_key, queue.group_name, job["msg_id"], job["msg_id"], 1
    )
    assert len(pending) == 1
    assert await client.xlen(queue.stream_key) == original_length
    assert (await ledger.get_workflow_run(run.id)).status == "running"
    await worker.stop()


@pytest.mark.asyncio
async def test_retry_scheduling_is_idempotent(queue):
    run = make_run("retry_idempotent")
    payload = job_payload(run)
    retry_payload = dict(payload, attempt_count=2)
    dedupe_key = f"{queue.stream_key}:retry:{run.id}:2"

    await queue.enqueue_job(run.id, payload)
    job = await queue.claim_job("worker_retry_idempotent", block_ms=500, min_idle_time=10)
    assert job is not None

    first = await queue.reschedule_job(
        run_id=run.id,
        payload=retry_payload,
        msg_id=job["msg_id"],
        delay_ms=0,
        dedupe_key=dedupe_key,
    )
    second = await queue.reschedule_job(
        run_id=run.id,
        payload=retry_payload,
        msg_id=job["msg_id"],
        delay_ms=0,
        dedupe_key=dedupe_key,
    )
    assert first == second

    client = await queue.get_client()
    entries = await client.xrange(queue.stream_key)
    assert len(entries) == 2
    assert (await client.get(dedupe_key)) == first
    pending = await client.xpending_range(
        queue.stream_key, queue.group_name, job["msg_id"], job["msg_id"], 1
    )
    assert pending == []

    claimed = await queue.claim_job("worker_retry_idempotent_2", block_ms=200, min_idle_time=10)
    assert claimed is not None
    assert claimed["run_id"] == run.id
    assert claimed["payload"]["attempt_count"] == 2
    await queue.ack_job(claimed["msg_id"])
