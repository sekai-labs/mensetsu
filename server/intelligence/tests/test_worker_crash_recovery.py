import uuid

import pytest

from src.contracts.v1 import WorkflowRun, WorkflowTask
from src.orchestration.ledger import SurrealLedgerRepository
from src.orchestration.redis_queue import RedisStreamJobQueue
from src.orchestration.runtime.worker import DurableJobWorker


@pytest.mark.asyncio
async def test_worker_crash_recovery_resumes_without_reexecution():
    ledger = SurrealLedgerRepository()
    group_name = f"test_group_{uuid.uuid4().hex[:8]}"
    queue = RedisStreamJobQueue(group_name=group_name)
    run_id = f"run_crash_{uuid.uuid4().hex[:8]}"

    run = WorkflowRun(
        id=run_id,
        workflow_id="interview-preparation-static",
        workflow_name="Crash Recovery Test",
        workspace_id="ws_crash_test",
        user_id="usr_crash_test",
        type="static",
        status="running",
    )
    await ledger.create_workflow_run(run)

    t1 = WorkflowTask(
        id=f"task_normalize_{run_id[:8]}",
        run_id=run_id,
        task_type="normalization",
        name="Normalize inputs",
        assigned_agent="job-analysis",
        status="completed",
        attempt=1,
    )
    await ledger.save_task(t1)

    await queue.enqueue_job(
        run_id=run_id,
        payload={
            "run_id": run_id,
            "workflow_id": "interview-preparation-static",
            "workspace_id": "ws_crash_test",
            "user_id": "usr_crash_test",
            "company_name": "Crash Corp",
            "role_title": "Platform Engineer",
            "resume_text": "Platform engineer with Rust and Kafka experience.",
            "job_description": "Own platform reliability and streaming infrastructure.",
        },
    )

    worker_a_id = "worker_crashed_process_A"
    claimed_a = None
    for _ in range(5):
        cand = await queue.claim_job(worker_id=worker_a_id, block_ms=1000)
        if cand and cand.get("run_id") == run_id:
            claimed_a = cand
            break
        elif cand:
            await queue.ack_job(cand["msg_id"])
    assert claimed_a is not None

    r = await queue.get_client()
    await r.delete(f"mensetsu:lease:{run_id}")

    from src.agents.runtime import agent_runtime
    from tests.test_agent_runtime_tools import ScriptedToolCallingFakeModel

    agent_runtime.set_model_factory(lambda: ScriptedToolCallingFakeModel())
    worker_b = DurableJobWorker(worker_id="worker_surviving_B")
    worker_b.queue = queue
    await worker_b.initialize()

    claimed_a["reclaimed"] = True
    from unittest.mock import AsyncMock, patch

    with patch(
        "src.agents.runtime.SafeResearchFetcher.fetch_clean_text", new_callable=AsyncMock
    ) as mock_fetch:
        mock_fetch.return_value = (
            "Careers at Acme",
            "We build distributed systems in Python and Rust.",
        )
        await worker_b.execute_job(claimed_a)

    res_t1 = await ledger.get_task(run_id, t1.id)
    assert res_t1.status == "completed"
    assert res_t1.attempt == 1

    clean_run_id = run_id.replace("workflow_run:", "")
    events = await ledger.get_events_history(run_id=clean_run_id, limit=50)
    event_types = [e.type for e in events.events]
    assert "execution.resumed" in event_types
    assert "worker.recovered" in event_types
    await worker_b.stop()
