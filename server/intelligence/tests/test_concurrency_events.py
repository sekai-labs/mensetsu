import asyncio

import pytest

from src.contracts.v1 import WorkflowRun
from src.orchestration.ledger import SurrealLedgerRepository


@pytest.mark.asyncio
async def test_concurrent_event_appends_no_gaps():
    ledger = SurrealLedgerRepository()
    import uuid

    run_id = f"test_run_concurrency_{uuid.uuid4().hex[:8]}"
    run = WorkflowRun(
        id=run_id,
        workflow_id="interview-preparation-static",
        workflow_name="Static Test",
        workspace_id="ws_conc_test",
        user_id="user_conc_test",
        type="static",
        status="running",
    )
    await ledger.create_workflow_run(run)

    async def append_single(i: int):
        return await ledger.append_event_transactional(
            run_id=run_id,
            event_type="agent.activity",
            execution_id=f"exec_{i}",
            payload={
                "execution_id": f"exec_{i}",
                "activity_type": "concurrent_step",
                "description": f"Concurrent step {i}",
            },
        )

    tasks = [append_single(i) for i in range(50)]
    await asyncio.gather(*tasks)

    history = await ledger.get_events_history(run_id=run_id, after_seq=0, limit=100)
    seqs = [e.seq for e in history.events]

    assert len(seqs) == 50

    assert seqs == list(range(1, 51))
