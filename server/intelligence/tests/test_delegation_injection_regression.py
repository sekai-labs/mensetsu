import pytest

from src.contracts.v1 import AgentExecution, Delegation, WorkflowRun
from src.orchestration.ledger import SurrealLedgerRepository


@pytest.mark.asyncio
async def test_delegation_sql_injection_and_roundtrip():

    ledger = SurrealLedgerRepository()
    run_id = "run_delegation_injection_test"
    del_id = "del_injection_test_1"

    run = WorkflowRun(
        id=run_id,
        workflow_id="interview-preparation-static",
        workflow_name="Injection Proof Run",
        workspace_id="ws_inj_proof",
        user_id="usr_inj_proof",
        type="static",
        status="running",
    )
    await ledger.create_workflow_run(run)

    for exec_id, agent_id, objective in (
        ("exec_parent_inj", "company-research", "parent"),
        ("exec_child_inj", "technical-research", "child"),
    ):
        await ledger.save_agent_execution(
            AgentExecution(
                id=exec_id,
                run_id=run_id,
                task_id="task_inj_1",
                agent_id=agent_id,
                objective=objective,
                status="running",
            )
        )

    malicious_text = "') ; DELETE workflow_run; -- '\n\"}\n$x\t\\"

    delegation = Delegation(
        id=del_id,
        run_id=run_id,
        parent_execution_id="exec_parent_inj",
        child_execution_id="exec_child_inj",
        parent_agent_id="company-research",
        child_agent_id="technical-research",
        delegation_task_id="task_inj_1",
        objective=malicious_text,
        reason=malicious_text,
        depth=1,
        status="requested",
    )
    await ledger.save_delegation(delegation)

    for st in ["accepted", "running", "completed"]:
        delegation.status = st
        if st == "completed":
            delegation.output_artifact_ids = ["art_inj_out_1"]
        await ledger.save_delegation(delegation)

    delegations = await ledger.list_delegations_for_run(run_id)
    assert len(delegations) == 1, f"Expected exactly 1 delegation edge, found {len(delegations)}!"

    saved_del = delegations[0]
    assert saved_del.status == "completed"
    assert saved_del.objective == malicious_text
    assert saved_del.reason == malicious_text
    assert saved_del.output_artifact_ids == ["art_inj_out_1"]

    refreshed_run = await ledger.get_workflow_run(run_id)
    assert refreshed_run is not None
    assert refreshed_run.id == run_id
    assert refreshed_run.status == "running"
