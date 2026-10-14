import pytest

from src.agents.runtime import AutonomousAgentRuntime
from src.contracts.v1 import AgentExecution
from src.orchestration.ledger import SurrealLedgerRepository


@pytest.mark.asyncio
async def test_delegation_guardrails_depth_cycle_and_children():
    ledger = SurrealLedgerRepository()
    runtime = AutonomousAgentRuntime(ledger=ledger)
    run_id = "test_run_delegation_limits"

    parent_exec = AgentExecution(
        id="exec_parent_1",
        run_id=run_id,
        task_id="task_1",
        agent_id="company-research",
        objective="Root objective",
    )
    tools = runtime.create_tools_for_execution(
        execution=parent_exec,
        run_id=run_id,
        depth=1,
        ancestors=["company-research"],
    )
    tool_map = {t.name: t for t in tools}

    depth_res = await tool_map["delegate_task"].ainvoke(
        {
            "target_agent_id": "technical-research",
            "objective": "Deep subtask",
            "reason": "Exceeding depth",
        }
    )
    assert "maximum delegation depth (1) exceeded" in depth_res
    assert len(parent_exec.child_execution_ids) == 0

    tools_level_1 = runtime.create_tools_for_execution(
        execution=parent_exec,
        run_id=run_id,
        depth=0,
        ancestors=["technical-research", "company-research"],
    )
    tool_map_1 = {t.name: t for t in tools_level_1}
    cycle_res = await tool_map_1["delegate_task"].ainvoke(
        {
            "target_agent_id": "technical-research",
            "objective": "Cycle objective",
            "reason": "Testing cycle",
        }
    )
    assert "cycle detected" in cycle_res
    assert len(parent_exec.child_execution_ids) == 0

    parent_exec.child_execution_ids = []
    unpermitted_res = await tool_map_1["delegate_task"].ainvoke(
        {
            "target_agent_id": "curriculum",
            "objective": "Unpermitted delegation",
            "reason": "Testing permission check",
        }
    )
    assert "does not have delegation permission" in unpermitted_res

    parent_exec.child_execution_ids = ["exec_c1", "exec_c2"]
    tools_level_fresh = runtime.create_tools_for_execution(
        execution=parent_exec,
        run_id=run_id,
        depth=0,
        ancestors=["company-research"],
    )
    tool_map_fresh = {t.name: t for t in tools_level_fresh}
    children_res = await tool_map_fresh["delegate_task"].ainvoke(
        {
            "target_agent_id": "technical-research",
            "objective": "Third child objective",
            "reason": "Testing children count",
        }
    )
    assert "maximum children count (2) reached" in children_res
    assert len(parent_exec.child_execution_ids) == 2
