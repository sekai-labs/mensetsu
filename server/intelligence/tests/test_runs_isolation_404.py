import os
import uuid

import pytest
from httpx import ASGITransport, AsyncClient

from src.contracts.v1 import WorkflowRun, WorkflowTask
from src.orchestration.ledger import SurrealLedgerRepository
from src.service import app


async def create_run(client: AsyncClient, token: str, workspace_id: str, user_id: str, company: str) -> WorkflowRun:
    res = await client.post(
        "/internal/v1/workflows/runs",
        json={
            "workspace_id": workspace_id,
            "user_id": user_id,
            "workflow_id": "interview-preparation-static",
            "company_name": company,
            "role_title": "Backend Lead",
            "resume_text": f"{company} resume",
            "job_description": f"{company} JD",
        },
        headers={"Authorization": f"Bearer {token}"},
    )
    assert res.status_code == 201
    return WorkflowRun.model_validate(res.json())


async def seed_completed_run(ledger: SurrealLedgerRepository, run: WorkflowRun, task_suffix: str) -> WorkflowTask:
    task = WorkflowTask(
        id=f"task_company_{task_suffix}",
        run_id=run.id,
        task_type="research",
        name="Research Target Company",
        assigned_agent="company-research",
        status="completed",
    )
    await ledger.save_task(task)
    await ledger.update_workflow_run(run.id, {"status": "completed", "current_phase": "completed"})
    return task


@pytest.mark.asyncio
async def test_concurrent_runs_isolation_and_foreign_lookups_404():
    token = os.environ.get("INTERNAL_SERVICE_TOKEN", "test_internal_service_token_12345")
    from src.orchestration.router import job_queue

    job_queue.client = None
    ledger = SurrealLedgerRepository()
    suffix = uuid.uuid4().hex[:8]

    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        run_a = await create_run(client, token, f"ws_isolation_A_{suffix}", "usr_A", "Alpha Corp")
        run_b = await create_run(client, token, f"ws_isolation_B_{suffix}", "usr_B", "Beta Corp")
        await seed_completed_run(ledger, run_a, f"a_{suffix}")
        task_b = await seed_completed_run(ledger, run_b, f"b_{suffix}")

        run_a_refreshed = await ledger.get_workflow_run(run_a.id)
        run_b_refreshed = await ledger.get_workflow_run(run_b.id)
        assert run_a_refreshed.status == "completed"
        assert run_b_refreshed.status == "completed"

        own_task_res = await client.get(
            f"/internal/v1/workflows/runs/{run_b.id}/tasks/{task_b.id}",
            headers={"Authorization": f"Bearer {token}"},
        )
        assert own_task_res.status_code == 200

        foreign_task_res = await client.get(
            f"/internal/v1/workflows/runs/{run_a.id}/tasks/{task_b.id}",
            headers={"Authorization": f"Bearer {token}"},
        )
        assert foreign_task_res.status_code == 404

    await job_queue.close()
