import json

import pytest

import grpc
import src.orchestration.shared as shared_mod
from src.agents.runtime import current_ledger_var
from src.contracts.v1 import (
    AgentExecution,
    Artifact,
    Delegation,
    WorkflowRun,
    WorkflowTask,
)
from src.grpc.auth_interceptor import GrpcAuthInterceptor
from src.grpc.control_servicer import DataControlServicer
from src.grpc.v1 import control_pb2, control_pb2_grpc
from src.orchestration.ledger import SurrealLedgerRepository


@pytest.fixture
async def grpc_control_server(monkeypatch):
    token = "test_internal_service_token_12345"
    monkeypatch.setenv("INTERNAL_SERVICE_TOKEN", token)

    server = grpc.aio.server(interceptors=[GrpcAuthInterceptor()])
    control_pb2_grpc.add_DataControlServiceServicer_to_server(DataControlServicer(), server)
    port = server.add_insecure_port("127.0.0.1:0")
    await server.start()

    yield port, token

    await server.stop(grace=0)


@pytest.mark.asyncio
async def test_all_control_commands_and_contract_responses(grpc_control_server):
    port, token = grpc_control_server
    run_id = "run_route_test_123"
    ws_id = "ws_route_test_123"

    ledger = SurrealLedgerRepository()
    shared_mod.default_ledger = ledger
    current_ledger_var.set(ledger)

    run = WorkflowRun(
        id=run_id,
        workflow_id="interview-preparation-hybrid",
        workflow_name="Hybrid",
        workspace_id=ws_id,
        user_id="usr_test",
        type="hybrid",
        status="pending",
        current_phase="queued",
        plan_revision=1,
        last_seq=0,
    )
    task = WorkflowTask(
        id="task_1",
        run_id=run_id,
        name="Analyze Job",
        assigned_agent="planner",
        status="completed",
        task_type="research",
    )
    agent = AgentExecution(
        id="exec_1",
        run_id=run_id,
        task_id="task_1",
        agent_id="company-research",
        objective="Analyze company",
        status="completed",
    )
    delegation = Delegation(
        id="del_1",
        run_id=run_id,
        parent_execution_id="exec_parent",
        child_execution_id="exec_child",
        parent_agent_id="planner",
        child_agent_id="company-research",
        delegation_task_id="task_1",
        objective="Analyze company",
        reason="Subtask delegation",
        status="completed",
    )
    artifact = Artifact(
        id="art_1",
        run_id=run_id,
        type="research_report",
        title="Test Brief",
        content='{"key":"value"}',
    )
    from src.contracts.v1 import TaskCompletedEvent, TaskCompletedPayload

    evt = TaskCompletedEvent(
        event_id="evt_1",
        run_id=run_id,
        seq=1,
        task_id="task_1",
        payload=TaskCompletedPayload(task_id="task_1", duration_ms=10.0),
    )

    async def mock_list_runs(*args, **kwargs):
        from src.contracts.v1 import RunsPageResponse

        return RunsPageResponse(items=[run], next_cursor=None)

    async def mock_list_tools(*args, **kwargs):
        from src.contracts.v1 import ToolInvocationsPageResponse

        return ToolInvocationsPageResponse(items=[], next_cursor=None)

    async def mock_tool_inv(*args, **kwargs):
        class MockTool:
            output_summary = "mock tool invocation output"

        return MockTool()

    async def mock_logs_page(*args, **kwargs):
        from src.contracts.v1 import LogsPageResponse

        return LogsPageResponse(items=[], next_cursor=None)

    async def mock_task_graph(*args, **kwargs):
        from src.contracts.v1 import TaskGraphResponse

        return TaskGraphResponse(nodes=[], edges=[])

    async def mock_del_graph(*args, **kwargs):
        from src.contracts.v1 import DelegationGraphResponse

        return DelegationGraphResponse(nodes=[], edges=[])

    async def mock_evts_history(*args, **kwargs):
        from src.contracts.v1 import EventsHistoryPageResponse

        return EventsHistoryPageResponse(events=[evt], next_after_seq=None, last_seq=1)

    ledger.get_workflow_run = lambda r: mock_return(run if r == run_id else None)
    ledger.list_workflow_runs = mock_list_runs
    ledger.get_task = lambda r, t: mock_return(task if t == "task_1" else None)
    ledger.list_agent_executions_for_run = lambda r: mock_return([agent])
    ledger.get_agent_execution = lambda r, e: mock_return(agent if e == "exec_1" else None)
    ledger.list_tools_for_execution = mock_list_tools
    ledger.get_tool_invocation = mock_tool_inv
    ledger.list_delegations_for_run = lambda r: mock_return([delegation])
    ledger.list_artifacts_for_run = lambda r: mock_return([artifact])
    ledger.get_logs_page = mock_logs_page
    ledger.get_task_graph = mock_task_graph
    ledger.get_delegation_graph = mock_del_graph
    ledger.get_events_history = mock_evts_history
    ledger.update_workflow_run = lambda *args, **kwargs: mock_return(None)
    ledger.sweep_run_to_terminal = lambda *args, **kwargs: mock_return(None)
    ledger.append_event_transactional = lambda *args, **kwargs: mock_return(None)

    async with grpc.aio.insecure_channel(f"127.0.0.1:{port}") as channel:
        stub = control_pb2_grpc.DataControlServiceStub(channel)
        metadata = (("authorization", f"Bearer {token}"),)

        # Helper to send a single command and receive envelope
        async def call_cmd(cmd: control_pb2.ControlCommand) -> control_pb2.ControlDataEnvelope:
            async def gen():
                yield cmd

            call = stub.StreamControl(gen(), metadata=metadata)
            res = []
            async for env in call:
                res.append(env)
            assert len(res) == 1
            return res[0]

        # 1. list_workflows
        e = await call_cmd(
            control_pb2.ControlCommand(
                correlation_id="1", list_workflows=control_pb2.ListWorkflowsCommand()
            )
        )
        assert e.status_code == 200
        defs = json.loads(e.payload_json)
        assert len(defs) == 3

        # 2. list_runs
        e = await call_cmd(
            control_pb2.ControlCommand(correlation_id="2", list_runs=control_pb2.ListRunsCommand())
        )
        assert e.status_code == 200
        assert run_id in e.payload_json

        # 3. get_run
        e = await call_cmd(
            control_pb2.ControlCommand(
                correlation_id="3", get_run=control_pb2.GetRunCommand(run_id=run_id)
            )
        )
        assert e.status_code == 200
        assert json.loads(e.payload_json)["id"] == run_id

        # 4. get_task
        e = await call_cmd(
            control_pb2.ControlCommand(
                correlation_id="4",
                get_task=control_pb2.GetTaskCommand(run_id=run_id, task_id="task_1"),
            )
        )
        assert e.status_code == 200
        assert json.loads(e.payload_json)["id"] == "task_1"

        # 5. list_agents
        e = await call_cmd(
            control_pb2.ControlCommand(
                correlation_id="5", list_agents=control_pb2.ListAgentsCommand(run_id=run_id)
            )
        )
        assert e.status_code == 200
        assert json.loads(e.payload_json)[0]["id"] == "exec_1"

        # 6. get_agent
        e = await call_cmd(
            control_pb2.ControlCommand(
                correlation_id="6",
                get_agent=control_pb2.GetAgentCommand(run_id=run_id, execution_id="exec_1"),
            )
        )
        assert e.status_code == 200
        assert json.loads(e.payload_json)["id"] == "exec_1"

        # 7. list_agent_tools
        e = await call_cmd(
            control_pb2.ControlCommand(
                correlation_id="7",
                list_agent_tools=control_pb2.ListAgentToolsCommand(
                    run_id=run_id, execution_id="exec_1"
                ),
            )
        )
        assert e.status_code == 200

        # 8. get_tool_output
        e = await call_cmd(
            control_pb2.ControlCommand(
                correlation_id="8",
                get_tool_output=control_pb2.GetToolOutputCommand(
                    run_id=run_id, tool_invocation_id="tool_1", offset=0, limit=100
                ),
            )
        )
        assert e.status_code == 200
        assert json.loads(e.payload_json)["content"] == "mock tool invocation output"

        # 9. list_delegations
        e = await call_cmd(
            control_pb2.ControlCommand(
                correlation_id="9",
                list_delegations=control_pb2.ListDelegationsCommand(run_id=run_id),
            )
        )
        assert e.status_code == 200
        assert json.loads(e.payload_json)[0]["id"] == "del_1"

        # 10. list_artifacts
        e = await call_cmd(
            control_pb2.ControlCommand(
                correlation_id="10", list_artifacts=control_pb2.ListArtifactsCommand(run_id=run_id)
            )
        )
        assert e.status_code == 200
        assert json.loads(e.payload_json)[0]["id"] == "art_1"

        # 11. get_logs
        e = await call_cmd(
            control_pb2.ControlCommand(
                correlation_id="11", get_logs=control_pb2.GetLogsCommand(run_id=run_id)
            )
        )
        assert e.status_code == 200

        # 12. get_task_graph
        e = await call_cmd(
            control_pb2.ControlCommand(
                correlation_id="12", get_task_graph=control_pb2.GetTaskGraphCommand(run_id=run_id)
            )
        )
        assert e.status_code == 200

        # 13. get_delegation_graph
        e = await call_cmd(
            control_pb2.ControlCommand(
                correlation_id="13",
                get_delegation_graph=control_pb2.GetDelegationGraphCommand(run_id=run_id),
            )
        )
        assert e.status_code == 200

        # 14. cancel_run
        e = await call_cmd(
            control_pb2.ControlCommand(
                correlation_id="14", cancel_run=control_pb2.CancelRunCommand(run_id=run_id)
            )
        )
        assert e.status_code == 200
        assert json.loads(e.payload_json)["status"] == "cancelled"

        # 15. get_events_history
        e = await call_cmd(
            control_pb2.ControlCommand(
                correlation_id="15",
                get_events_history=control_pb2.GetEventsHistoryCommand(
                    run_id=run_id, after_seq=0, limit=50
                ),
            )
        )
        assert e.status_code == 200
        assert len(json.loads(e.payload_json)["events"]) == 1


async def mock_return(val):
    return val
