import pytest

from src.contracts.v1 import WorkflowRun
from src.orchestration.ledger import SurrealLedgerRepository


@pytest.mark.asyncio
async def test_ledger_injection_safety_and_special_characters():
    ledger = SurrealLedgerRepository()
    run_id = "test_run_injection_safe_1"

    special_objective = "Analyze 'special' chars: \"; -- $eval() } \t"

    run = WorkflowRun(
        id=run_id,
        workflow_id="interview-preparation-static",
        workflow_name="Special Chars Run",
        workspace_id="ws_injection_safe",
        user_id="usr_safe",
        type="static",
        status="running",
        error_message=special_objective,
    )

    await ledger.create_workflow_run(run)
    fetched = await ledger.get_workflow_run(run_id)

    assert fetched is not None

    assert fetched.error_message == special_objective
    assert fetched.workflow_name == "Special Chars Run"
