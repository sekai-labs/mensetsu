import pytest
from httpx import ASGITransport, AsyncClient

from src.contracts.v1 import (
    AgentExecution,
    Artifact,
    DelegationGraphResponse,
    EventsHistoryPageResponse,
    LogsPageResponse,
    TaskGraphResponse,
    ToolInvocationsPageResponse,
    ToolOutputResponse,
    WorkflowRun,
    WorkflowTask,
)
from src.orchestration.ledger import SurrealLedgerRepository
from src.service import app


@pytest.mark.asyncio
async def test_all_internal_workflow_routes_signatures_and_responses():
    token = "test_internal_service_token_12345"
    run_id = "run_route_test_123"
    ws_id = "ws_route_test_123"

    ledger = SurrealLedgerRepository()
    import src.orchestration.router as router_mod
    from src.agents.runtime import current_ledger_var
    router_mod.default_ledger = ledger
    current_ledger_var.set(ledger)

    run = WorkflowRun(
        id=run_id,
        workflow_id="interview-preparation-static",
        workflow_name="Route Test",
        workspace_id=ws_id,
        user_id="usr_route",
        type="static",
        status="running",
    )
    await ledger.create_workflow_run(run)

    task = WorkflowTask(
        id="task_route_1",
        run_id=run_id,
        task_type="research",
        name="Route Task",
        assigned_agent="company-research",
        status="completed",
    )
    await ledger.save_task(task)

    agent_exec = AgentExecution(
        id="exec_route_1",
        run_id=run_id,
        task_id=task.id,
        agent_id="company-research",
        objective="Analyze routes",
        status="completed",
        public_summary="Route analysis complete",
    )
    await ledger.save_agent_execution(agent_exec)

    from src.contracts.v1 import ToolInvocation
    tool_inv = ToolInvocation(
        id="tool_route_1",
        execution_id=agent_exec.id,
        tool_name="web_search",
        status="completed",
        output_summary="Tool output test text",
    )
    await ledger.save_tool_invocation(tool_inv, run_id=run_id)

    art = Artifact(
        id="art_route_1",
        run_id=run_id,
        task_id=task.id,
        agent_execution_id=agent_exec.id,
        type="research_report",
        title="Route Deliverable",
        content={"data": "test"},
    )
    await ledger.save_artifact(art)

    await ledger.append_execution_log(
        run_id=run_id,
        message="Route execution log entry",
        severity="info",
        task_id=task.id,
        execution_id=agent_exec.id,
    )

    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test", timeout=10.0) as client:
        headers = {"Authorization": f"Bearer {token}"}

        r_runs = await client.get("/internal/v1/workflows/runs", headers=headers)
        assert r_runs.status_code == 200
        r_run = await client.get(f"/internal/v1/workflows/runs/{run_id}", headers=headers)
        assert r_run.status_code == 200
        WorkflowRun.model_validate(r_run.json())

        r_task = await client.get(f"/internal/v1/workflows/runs/{run_id}/tasks/{task.id}", headers=headers)
        assert r_task.status_code == 200
        WorkflowTask.model_validate(r_task.json())

        r_agents = await client.get(f"/internal/v1/workflows/runs/{run_id}/agents", headers=headers)
        assert r_agents.status_code == 200

        r_agent = await client.get(f"/internal/v1/workflows/runs/{run_id}/agents/{agent_exec.id}", headers=headers)
        assert r_agent.status_code == 200
        AgentExecution.model_validate(r_agent.json())

        r_tools = await client.get(f"/internal/v1/workflows/runs/{run_id}/agents/{agent_exec.id}/tools", headers=headers)
        assert r_tools.status_code == 200
        ToolInvocationsPageResponse.model_validate(r_tools.json())

        r_out = await client.get(f"/internal/v1/workflows/runs/{run_id}/tools/{tool_inv.id}/output", headers=headers)
        assert r_out.status_code == 200
        t_out = ToolOutputResponse.model_validate(r_out.json())
        assert t_out.content == "Tool output test text"

        r_arts = await client.get(f"/internal/v1/workflows/runs/{run_id}/artifacts", headers=headers)
        assert r_arts.status_code == 200

        r_logs = await client.get(f"/internal/v1/workflows/runs/{run_id}/logs", headers=headers)
        assert r_logs.status_code == 200
        logs_page = LogsPageResponse.model_validate(r_logs.json())
        assert len(logs_page.items) >= 1
        assert logs_page.items[0].message == "Route execution log entry"

        r_graph = await client.get(f"/internal/v1/workflows/runs/{run_id}/graph", headers=headers)
        assert r_graph.status_code == 200
        TaskGraphResponse.model_validate(r_graph.json())

        r_del_graph = await client.get(f"/internal/v1/workflows/runs/{run_id}/delegations/graph", headers=headers)
        assert r_del_graph.status_code == 200
        DelegationGraphResponse.model_validate(r_del_graph.json())

        r_ev_hist = await client.get(f"/internal/v1/workflows/runs/{run_id}/events/history", headers=headers)
        assert r_ev_hist.status_code == 200
        EventsHistoryPageResponse.model_validate(r_ev_hist.json())
