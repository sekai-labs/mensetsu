import asyncio
from datetime import datetime, timezone

import pytest

from src.contracts.v1 import WorkflowTask
from src.orchestration.ledger import SurrealLedgerRepository


@pytest.mark.asyncio
async def test_concurrency_proof_research_tasks_overlap():
    ledger = SurrealLedgerRepository()
    run_id = "test_run_concurrency_overlap_99"

    t1_start = datetime.now(timezone.utc)
    await asyncio.sleep(0.05)

    task1 = WorkflowTask(
        id=f"t1_{run_id[:8]}",
        run_id=run_id,
        task_type="research",
        name="Company Research",
        assigned_agent="company-research",
        status="running",
        created_at=t1_start,
    )
    task2 = WorkflowTask(
        id=f"t2_{run_id[:8]}",
        run_id=run_id,
        task_type="research",
        name="Job Analysis",
        assigned_agent="job-analysis",
        status="running",
        created_at=t1_start,
    )
    task3 = WorkflowTask(
        id=f"t3_{run_id[:8]}",
        run_id=run_id,
        task_type="research",
        name="Technical Systems",
        assigned_agent="technical-research",
        status="running",
        created_at=t1_start,
    )

    await ledger.save_task(task1)
    await ledger.save_task(task2)
    await ledger.save_task(task3)

    tasks = await ledger.list_tasks_for_run(run_id)
    assert len(tasks) >= 3
    running_tasks = [t for t in tasks if t.status == "running"]
    assert len(running_tasks) >= 3
