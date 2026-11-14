import asyncio
import os
import uuid

import pytest

from src.contracts.v1 import WorkflowRun
from src.orchestration.ledger import SurrealLedgerRepository
from src.orchestration.redis_queue import RedisStreamJobQueue
from src.orchestration.runtime.worker import DurableJobWorker


class SlowGraph:
    def __init__(self, ledger: SurrealLedgerRepository, run_id: str, hold_seconds: float):
        self.ledger = ledger
        self.run_id = run_id
        self.hold_seconds = hold_seconds
        self.started = asyncio.Event()
        self.calls = 0
        self.active = 0
        self.max_active = 0

    async def ainvoke(self, _input, config=None):
        self.calls += 1
        self.active += 1
        self.max_active = max(self.max_active, self.active)
        self.started.set()
        try:
            await asyncio.sleep(self.hold_seconds)
            completed = await self.ledger.transition_workflow_run_status(
                self.run_id,
                "completed",
                from_statuses=("running",),
                updates={"current_phase": "completed"},
            )
            if not completed:
                raise AssertionError("The controlled graph could not complete the run.")
        finally:
            self.active -= 1


@pytest.mark.asyncio
async def test_live_worker_heartbeat_prevents_second_worker_claim():
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

    worker_a = DurableJobWorker(
        worker_id=f"worker_A_{unique_suffix}",
        redis_url=redis_url,
        heartbeat_interval_s=0.5,
        lease_ttl_s=2.0,
        min_idle_time_ms=3000,
    )
    worker_a.queue = queue
    worker_a.ledger = ledger
    worker_b = DurableJobWorker(
        worker_id=f"worker_B_{unique_suffix}",
        redis_url=redis_url,
        heartbeat_interval_s=0.5,
        lease_ttl_s=2.0,
        min_idle_time_ms=3000,
    )
    worker_b.queue = queue
    worker_b.ledger = ledger

    task_a = None
    try:
        await worker_a.initialize()
        await worker_b.initialize()
        graph = SlowGraph(ledger, run_id, hold_seconds=8.0)
        await queue.enqueue_job(
            run_id=run_id,
            payload={
                "run_id": run_id,
                "workflow_id": "interview-preparation-static",
                "workspace_id": run.workspace_id,
                "user_id": run.user_id,
                "company_name": "Race Prevention Corp",
                "role_title": "Distributed Systems Engineer",
                "resume_text": "Distributed systems engineer.",
                "job_description": "Design fault-tolerant distributed systems.",
            },
        )
        job = await queue.claim_job(worker_a.worker_id, block_ms=1000, min_idle_time=3000)
        assert job is not None
        assert job["run_id"] == run_id

        worker_b_job_claimed = False
        with pytest.MonkeyPatch.context() as monkeypatch:
            monkeypatch.setattr(
                "src.orchestration.runtime.worker.build_static_workflow",
                lambda **_kwargs: graph,
            )
            task_a = asyncio.create_task(worker_a.execute_job(job))
            await asyncio.wait_for(graph.started.wait(), timeout=5.0)
            for _ in range(15):
                await asyncio.sleep(0.3)
                stolen_job = await queue.claim_job(
                    worker_b.worker_id, block_ms=100, min_idle_time=3000
                )
                if stolen_job:
                    worker_b_job_claimed = True
                    await worker_b.execute_job(stolen_job)
                    break
            await task_a

        assert not worker_b_job_claimed, "Worker B reclaimed a message with a live heartbeat."
        assert graph.calls == 1
        assert graph.max_active == 1
        assert (await ledger.get_workflow_run(run_id)).status == "completed"
    finally:
        if task_a is not None and not task_a.done():
            task_a.cancel()
            await asyncio.gather(task_a, return_exceptions=True)
        await worker_a.stop()
        await worker_b.stop()
        await queue.close()
