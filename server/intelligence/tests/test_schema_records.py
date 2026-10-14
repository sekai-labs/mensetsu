import pytest

from src.contracts.v1 import (
    AgentExecution,
    Artifact,
    Delegation,
    ToolInvocation,
    WorkflowRun,
    WorkflowTask,
)
from src.orchestration.ledger import SurrealLedgerRepository


@pytest.mark.asyncio
async def test_schema_full_minimal_and_full_records():
    ledger = SurrealLedgerRepository()
    run_id = "test_run_schema_full_v1"

    min_run = WorkflowRun(
        id=run_id,
        workflow_id="interview-preparation-static",
        workflow_name="Static",
        workspace_id="ws_schema_test",
        user_id="usr_schema_test",
        type="static",
        status="running",
    )
    await ledger.create_workflow_run(min_run)
    fetched_min_run = await ledger.get_workflow_run(run_id)
    assert fetched_min_run is not None

    task = WorkflowTask(
        id=f"task_{run_id[:8]}",
        run_id=run_id,
        task_type="research",
        name="Test Task",
        assigned_agent="company-research",
        status="completed",
    )
    await ledger.save_task(task)
    fetched_task = await ledger.get_task(run_id, task.id)
    assert fetched_task is not None

    agent_exec = AgentExecution(
        id=f"exec_{run_id[:8]}",
        run_id=run_id,
        task_id=task.id,
        agent_id="company-research",
        objective="Analyze culture",
        status="completed",
    )
    await ledger.save_agent_execution(agent_exec)
    fetched_agent = await ledger.get_agent_execution(run_id, agent_exec.id)
    assert fetched_agent is not None

    tool_inv = ToolInvocation(
        id=f"tool_{run_id[:8]}",
        execution_id=agent_exec.id,
        tool_name="web_search",
        status="completed",
    )
    await ledger.save_tool_invocation(tool_inv, run_id=run_id)
    fetched_tool = await ledger.get_tool_invocation(run_id, tool_inv.id)
    assert fetched_tool is not None

    delegation = Delegation(
        id=f"del_{run_id[:8]}",
        run_id=run_id,
        parent_execution_id=agent_exec.id,
        child_execution_id=agent_exec.id,
        parent_agent_id="company-research",
        child_agent_id="technical-research",
        delegation_task_id=task.id,
        objective="Deep blog search",
        reason="Architecture expertise",
        status="completed",
    )
    await ledger.save_delegation(delegation)
    delegations = await ledger.list_delegations_for_run(run_id)
    assert len(delegations) >= 1

    art = Artifact(
        id=f"art_{run_id[:8]}",
        run_id=run_id,
        task_id=task.id,
        type="finding",
        title="Test Finding",
    )
    await ledger.save_artifact(art)
    artifacts = await ledger.list_artifacts_for_run(run_id)
    assert len(artifacts) >= 1
