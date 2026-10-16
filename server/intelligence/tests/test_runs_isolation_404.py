import uuid

import pytest

import src.orchestration.shared as shared_mod
from src.contracts.v1 import WorkflowRun, WorkflowTask
from src.orchestration.ledger import SurrealLedgerRepository
from src.orchestration.service import WorkflowService


async def create_run(workspace_id: str, user_id: str, company: str) -> WorkflowRun:
    return await WorkflowService.create_run(
        workspace_id=workspace_id,
        user_id=user_id,
        workflow_id="interview-preparation-static",
        company_name=company,
        role_title="Backend Lead",
        resume_text=f"{company} resume",
        job_description=f"{company} JD",
    )


async def seed_completed_run(
    ledger: SurrealLedgerRepository, run: WorkflowRun, task_suffix: str
) -> WorkflowTask:
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
    job_queue = shared_mod.job_queue
    job_queue.client = None
    ledger = SurrealLedgerRepository()
    shared_mod.default_ledger = ledger
    suffix = uuid.uuid4().hex[:8]

    run_a = await create_run(f"ws_isolation_A_{suffix}", "usr_A", "Alpha Corp")
    run_b = await create_run(f"ws_isolation_B_{suffix}", "usr_B", "Beta Corp")
    await seed_completed_run(ledger, run_a, f"a_{suffix}")
    task_b = await seed_completed_run(ledger, run_b, f"b_{suffix}")

    run_a_refreshed = await ledger.get_workflow_run(run_a.id)
    run_b_refreshed = await ledger.get_workflow_run(run_b.id)
    assert run_a_refreshed.status == "completed"
    assert run_b_refreshed.status == "completed"

    own_task = await WorkflowService.get_task(run_b.id, task_b.id)
    assert own_task is not None
    assert own_task.id == task_b.id

    foreign_task = await WorkflowService.get_task(run_a.id, task_b.id)
    assert foreign_task is None

    await job_queue.close()
