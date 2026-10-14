import uuid

import pytest

from src.contracts.v1 import WorkflowRun
from src.orchestration.ledger import SurrealLedgerRepository
from src.orchestration.redis_queue import RedisStreamJobQueue
from src.orchestration.runtime.worker import DurableJobWorker


@pytest.mark.asyncio
async def test_worker_fails_closed_job_remains_pending():

    queue = RedisStreamJobQueue()
    ledger = SurrealLedgerRepository()
    run_id = f"run_fail_closed_{uuid.uuid4().hex[:8]}"

    run = WorkflowRun(
        id=run_id,
        workflow_id="interview-preparation-static",
        workflow_name="Fail Closed Run",
        workspace_id="ws_fc",
        user_id="usr_fc",
        type="static",
        status="pending",
    )
    await ledger.create_workflow_run(run)
    await queue.enqueue_job(run_id=run.id, payload={"run_id": run.id, "workspace_id": "ws_fc"})

    failing_worker = DurableJobWorker(
        worker_id="worker_broken_deps",
        redis_url="redis://127.0.0.1:59999",
    )

    with pytest.raises(Exception):
        await failing_worker.start()

    assert failing_worker.running is False

    refreshed_run = await ledger.get_workflow_run(run.id)
    assert refreshed_run.status == "pending"

    client = await queue.get_client()
    pending_info = await client.xpending(queue.stream_key, queue.group_name)
    assert pending_info["pending"] == 0
    assert len(pending_info["consumers"]) == 0

    has_lease = await queue.verify_lease_token(run.id, "any_token")
    assert has_lease is False

    await queue.close()
