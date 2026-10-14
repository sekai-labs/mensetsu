import asyncio
import os
import uuid

import pytest

from src.agents.runtime import agent_runtime
from src.contracts.v1 import WorkflowRun
from src.orchestration.ledger import SurrealLedgerRepository
from src.orchestration.redis_queue import RedisStreamJobQueue
from src.orchestration.runtime.worker import DurableJobWorker
from tests.test_agent_runtime_tools import ScriptedToolCallingFakeModel


class SlowPerJobModel(ScriptedToolCallingFakeModel):
    delay_per_call: float = 0.8

    async def _agenerate(self, messages, stop=None, run_manager=None, **kwargs):
        await asyncio.sleep(self.delay_per_call)
        return await super()._agenerate(messages, stop=stop, run_manager=run_manager, **kwargs)


@pytest.mark.asyncio
async def test_worker_concurrent_jobs_isolated_lease_loss():

    unique_suffix = uuid.uuid4().hex[:8]
    stream_key = f"test_conc_lease_stream_{unique_suffix}"
    group_name = f"test_conc_lease_grp_{unique_suffix}"
    redis_url = os.environ.get("REDIS_URL", "redis://127.0.0.1:16379")

    ledger = SurrealLedgerRepository()
    queue = RedisStreamJobQueue(redis_url=redis_url, stream_key=stream_key, group_name=group_name)

    run_id_a = f"run_lease_loss_A_{unique_suffix}"
    run_id_b = f"run_lease_loss_B_{unique_suffix}"

    run_a = WorkflowRun(
        id=run_id_a,
        workflow_id="interview-preparation-static",
        workflow_name="Concurrent Run A",
        workspace_id=f"ws_conc_A_{unique_suffix}",
        user_id=f"usr_conc_A_{unique_suffix}",
        type="static",
        status="running",
    )
    run_b = WorkflowRun(
        id=run_id_b,
        workflow_id="interview-preparation-static",
        workflow_name="Concurrent Run B",
        workspace_id=f"ws_conc_B_{unique_suffix}",
        user_id=f"usr_conc_B_{unique_suffix}",
        type="static",
        status="running",
    )
    await ledger.create_workflow_run(run_a)
    await ledger.create_workflow_run(run_b)

    worker = DurableJobWorker(
        worker_id=f"worker_concurrent_{unique_suffix}",
        max_concurrent_jobs=2,
        redis_url=redis_url,
        heartbeat_interval_s=0.4,
        lease_ttl_s=3.0,
    )
    worker.queue = queue
    worker.ledger = ledger
    await worker.initialize()

    slow_model = SlowPerJobModel()
    agent_runtime.set_model_factory(lambda: slow_model)

    await queue.enqueue_job(
        run_id=run_id_a,
        payload={
            "run_id": run_id_a,
            "workflow_id": "interview-preparation-static",
            "workspace_id": f"ws_conc_A_{unique_suffix}",
            "user_id": f"usr_conc_A_{unique_suffix}",
            "company_name": "Job A Corp",
            "role_title": "Senior Engineer A",
            "resume_text": "Engineer A with Python and Kubernetes experience.",
            "job_description": "Operate scalable backend services.",
        },
    )
    await queue.enqueue_job(
        run_id=run_id_b,
        payload={
            "run_id": run_id_b,
            "workflow_id": "interview-preparation-static",
            "workspace_id": f"ws_conc_B_{unique_suffix}",
            "user_id": f"usr_conc_B_{unique_suffix}",
            "company_name": "Job B Corp",
            "role_title": "Senior Engineer B",
            "resume_text": "Engineer B with Go and PostgreSQL experience.",
            "job_description": "Build data-intensive platform services.",
        },
    )

    job_a = await queue.claim_job(worker.worker_id, block_ms=1000)
    job_b = await queue.claim_job(worker.worker_id, block_ms=1000)
    assert job_a is not None and job_b is not None

    from unittest.mock import AsyncMock, patch

    with patch(
        "src.agents.runtime.SafeResearchFetcher.fetch_clean_text", new_callable=AsyncMock
    ) as mock_fetch:
        mock_fetch.return_value = (
            "Careers at Acme",
            "We build distributed systems in Python and Rust.",
        )
        task_a = asyncio.create_task(worker.execute_job(job_a))
        task_b = asyncio.create_task(worker.execute_job(job_b))

        await asyncio.sleep(1.0)

        r_client = await queue.get_client()
        deleted_a = await r_client.delete(f"mensetsu:lease:{run_id_a}")
        assert deleted_a > 0, "Failed to delete lease key for Job A"

        await asyncio.gather(task_a, task_b, return_exceptions=True)

    final_a = await ledger.get_workflow_run(run_id_a)
    assert final_a.status != "completed", (
        f"Job A should have aborted, but completed! status={final_a.status}"
    )

    pending_a = await r_client.xpending_range(
        queue.stream_key, queue.group_name, job_a["msg_id"], job_a["msg_id"], 1
    )
    assert len(pending_a) == 1, "Job A was acknowledged despite losing lease!"

    final_b = await ledger.get_workflow_run(run_id_b)
    assert final_b.status == "completed", f"Job B failed: {final_b.error_message}"

    pending_b = await r_client.xpending_range(
        queue.stream_key, queue.group_name, job_b["msg_id"], job_b["msg_id"], 1
    )
    assert len(pending_b) == 0, "Job B was not acknowledged!"

    await worker.stop()
    await queue.close()
