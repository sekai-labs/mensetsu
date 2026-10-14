import asyncio
import os
import uuid

import pytest

from src.agents.runtime import agent_runtime
from src.contracts.v1 import EventsHistoryPageResponse, WorkflowRun
from src.orchestration.ledger import SurrealLedgerRepository
from src.orchestration.redis_queue import RedisStreamJobQueue
from src.orchestration.runtime.worker import DurableJobWorker
from tests.test_agent_runtime_tools import ScriptedToolCallingFakeModel


class SlowModel(ScriptedToolCallingFakeModel):
    delay_per_call: float = 0.7

    async def _agenerate(self, messages, stop=None, run_manager=None, **kwargs):
        await asyncio.sleep(self.delay_per_call)
        return await super()._agenerate(messages, stop=stop, run_manager=run_manager, **kwargs)


@pytest.mark.asyncio
async def test_worker_race_condition_xclaim_heartbeat_prevents_theft():

    unique_suffix = uuid.uuid4().hex[:8]
    stream_key = f"test_race_stream_{unique_suffix}"
    group_name = f"test_race_grp_{unique_suffix}"
    redis_url = os.environ.get("REDIS_URL", "redis://127.0.0.1:16379")

    ledger = SurrealLedgerRepository()
    queue = RedisStreamJobQueue(redis_url=redis_url, stream_key=stream_key, group_name=group_name)

    run_id = f"run_race_{unique_suffix}"
    run = WorkflowRun(
        id=run_id,
        workflow_id="interview-preparation-static",
        workflow_name="Race Prevention Test",
        workspace_id=f"ws_race_{unique_suffix}",
        user_id=f"usr_race_{unique_suffix}",
        type="static",
        status="running",
    )
    await ledger.create_workflow_run(run)
    from src.contracts.v1 import Artifact

    for k in range(5):
        await ledger.save_artifact(
            Artifact(
                id=f"art_ev_{k}",
                run_id=run_id,
                type="evidence",
                title=f"Evidence {k}",
                content={"snippet": "Evidence text"},
            )
        )

    worker_a = DurableJobWorker(
        worker_id="worker_A_race",
        redis_url=redis_url,
        heartbeat_interval_s=0.5,
        lease_ttl_s=2.0,
        min_idle_time_ms=1200,
    )
    worker_a.queue = queue
    worker_a.ledger = ledger
    await worker_a.initialize()

    worker_b = DurableJobWorker(
        worker_id="worker_B_race",
        redis_url=redis_url,
        heartbeat_interval_s=0.5,
        lease_ttl_s=2.0,
        min_idle_time_ms=1200,
    )
    worker_b.queue = queue
    worker_b.ledger = ledger
    await worker_b.initialize()

    slow_model = SlowModel(delay_per_call=0.7)
    agent_runtime.set_model_factory(lambda: slow_model)

    await queue.enqueue_job(
        run_id=run_id,
        payload={
            "run_id": run_id,
            "workflow_id": "interview-preparation-static",
            "workspace_id": f"ws_race_{unique_suffix}",
            "user_id": f"usr_race_{unique_suffix}",
            "company_name": "Race Prevention Corp",
            "role_title": "Distributed Systems Engineer",
            "resume_text": "Distributed systems engineer with consensus protocol experience.",
            "job_description": "Design fault-tolerant distributed systems.",
        },
    )

    job = await queue.claim_job(worker_a.worker_id, block_ms=1000, min_idle_time=1200)
    assert job is not None
    assert job["run_id"] == run_id

    from unittest.mock import AsyncMock, patch

    with patch(
        "src.agents.runtime.SafeResearchFetcher.fetch_clean_text", new_callable=AsyncMock
    ) as mock_fetch:
        mock_fetch.return_value = (
            "Careers at Acme",
            "We build distributed systems in Python and Rust.",
        )

        task_a = asyncio.create_task(worker_a.execute_job(job))

        worker_b_job_claimed = False
        for _ in range(15):
            await asyncio.sleep(0.3)
            stolen_job = await queue.claim_job(worker_b.worker_id, block_ms=100, min_idle_time=1200)
            if stolen_job:
                worker_b_job_claimed = True
                await worker_b.execute_job(stolen_job)
                break

        await task_a

    assert not worker_b_job_claimed, (
        "Worker B claimed an active job whose idle time should have been reset by Worker A!"
    )

    final_run = await ledger.get_workflow_run(run_id)
    assert final_run.status == "completed"

    clean_id = run_id.replace("workflow_run:", "")
    all_events = []
    curr_seq = 0
    while True:
        history = await ledger.get_events_history(run_id=clean_id, after_seq=curr_seq, limit=100)
        all_events.extend(history.events)
        if not history.events or history.next_after_seq is None:
            break
        curr_seq = history.next_after_seq
    completed_events = [e for e in all_events if e.type == "workflow.completed"]
    assert len(completed_events) == 1, (
        f"Expected 1 workflow.completed event, got {len(completed_events)} in {len(all_events)} events"
    )
    events = EventsHistoryPageResponse(
        events=all_events, next_after_seq=None, last_seq=all_events[-1].seq if all_events else 0
    )

    recovered_from_b = [
        e
        for e in events.events
        if e.type == "worker.recovered"
        and getattr(e.payload, "worker_id", "") == worker_b.worker_id
    ]
    assert len(recovered_from_b) == 0
    await worker_a.stop()
    await worker_b.stop()
    await queue.close()
