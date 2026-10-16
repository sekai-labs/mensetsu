import json
import logging
from typing import AsyncIterator

import grpc
from src.grpc.v1 import control_pb2, control_pb2_grpc
from src.orchestration.service import WorkflowService

logger = logging.getLogger("mensetsu.grpc.control")


class DataControlServicer(control_pb2_grpc.DataControlServiceServicer):
    async def StreamControl(
        self,
        request_iterator: AsyncIterator[control_pb2.ControlCommand],
        context: grpc.aio.ServicerContext,
    ) -> AsyncIterator[control_pb2.ControlDataEnvelope]:
        if hasattr(context, "send_initial_metadata"):
            await context.send_initial_metadata(())
        async for cmd in request_iterator:
            corr_id = cmd.correlation_id
            cmd_type = cmd.WhichOneof("command")

            try:
                if cmd_type == "ping":
                    yield control_pb2.ControlDataEnvelope(
                        correlation_id=corr_id,
                        status_code=200,
                        is_end_of_stream=True,
                        payload_json=json.dumps({"timestamp": cmd.ping.timestamp}),
                    )
                elif cmd_type == "list_workflows":
                    defs = await WorkflowService.list_definitions()
                    yield control_pb2.ControlDataEnvelope(
                        correlation_id=corr_id,
                        status_code=200,
                        is_end_of_stream=True,
                        payload_json=json.dumps([d.model_dump(mode="json") for d in defs]),
                    )
                elif cmd_type == "create_run":
                    c = cmd.create_run
                    llm_config = (
                        json.loads(c.llm_config_json)
                        if c.HasField("llm_config_json") and c.llm_config_json
                        else None
                    )
                    run = await WorkflowService.create_run(
                        workspace_id=c.workspace_id,
                        user_id=c.user_id,
                        workflow_id=c.workflow_id or "interview-preparation-hybrid",
                        company_name=c.company_name or "Target Company",
                        role_title=c.role_title or "Target Role",
                        company_urls=list(c.company_urls),
                        resume_text=c.resume_text or "",
                        resume_pdf_base64=c.resume_pdf_base64
                        if c.HasField("resume_pdf_base64")
                        else None,
                        resume_filename=c.resume_filename
                        if c.HasField("resume_filename")
                        else None,
                        job_description=c.job_description or "",
                        jd_url=c.jd_url if c.HasField("jd_url") else None,
                        seniority=c.seniority if c.HasField("seniority") else None,
                        user_prompt=c.user_prompt if c.HasField("user_prompt") else None,
                        workflow_mode=c.workflow_mode if c.HasField("workflow_mode") else None,
                        llm_config=llm_config,
                    )
                    yield control_pb2.ControlDataEnvelope(
                        correlation_id=corr_id,
                        status_code=201,
                        is_end_of_stream=True,
                        payload_json=run.model_dump_json(),
                    )
                elif cmd_type == "resume_plan":
                    res = await WorkflowService.resume_plan(
                        cmd.resume_plan.run_id, cmd.resume_plan.prompt
                    )
                    yield control_pb2.ControlDataEnvelope(
                        correlation_id=corr_id,
                        status_code=200,
                        is_end_of_stream=True,
                        payload_json=json.dumps(res),
                    )
                elif cmd_type == "list_runs":
                    c = cmd.list_runs
                    run_ids = list(c.run_ids) if c.run_ids else None
                    page = await WorkflowService.list_runs(
                        run_ids=run_ids,
                        workspace_id=c.workspace_id if c.HasField("workspace_id") else None,
                        status=c.status if c.HasField("status") else None,
                        run_type=c.run_type if c.HasField("run_type") else None,
                        limit=c.limit if c.limit > 0 else 50,
                        cursor=c.cursor if c.HasField("cursor") else None,
                    )
                    yield control_pb2.ControlDataEnvelope(
                        correlation_id=corr_id,
                        status_code=200,
                        is_end_of_stream=True,
                        payload_json=page.model_dump_json(),
                    )
                elif cmd_type == "get_run":
                    run = await WorkflowService.get_run(cmd.get_run.run_id)
                    if not run:
                        yield control_pb2.ControlDataEnvelope(
                            correlation_id=corr_id,
                            status_code=404,
                            error_message="Run not found",
                            is_end_of_stream=True,
                        )
                    else:
                        yield control_pb2.ControlDataEnvelope(
                            correlation_id=corr_id,
                            status_code=200,
                            is_end_of_stream=True,
                            payload_json=run.model_dump_json(),
                        )
                elif cmd_type == "cancel_run":
                    run = await WorkflowService.cancel_run(cmd.cancel_run.run_id)
                    yield control_pb2.ControlDataEnvelope(
                        correlation_id=corr_id,
                        status_code=200,
                        is_end_of_stream=True,
                        payload_json=run.model_dump_json(),
                    )
                elif cmd_type == "get_task":
                    task = await WorkflowService.get_task(cmd.get_task.run_id, cmd.get_task.task_id)
                    if not task:
                        yield control_pb2.ControlDataEnvelope(
                            correlation_id=corr_id,
                            status_code=404,
                            error_message="Task not found",
                            is_end_of_stream=True,
                        )
                    else:
                        yield control_pb2.ControlDataEnvelope(
                            correlation_id=corr_id,
                            status_code=200,
                            is_end_of_stream=True,
                            payload_json=task.model_dump_json(),
                        )
                elif cmd_type == "list_agents":
                    agents = await WorkflowService.list_agents(cmd.list_agents.run_id)
                    yield control_pb2.ControlDataEnvelope(
                        correlation_id=corr_id,
                        status_code=200,
                        is_end_of_stream=True,
                        payload_json=json.dumps([a.model_dump(mode="json") for a in agents]),
                    )
                elif cmd_type == "get_agent":
                    agent = await WorkflowService.get_agent(
                        cmd.get_agent.run_id, cmd.get_agent.execution_id
                    )
                    if not agent:
                        yield control_pb2.ControlDataEnvelope(
                            correlation_id=corr_id,
                            status_code=404,
                            error_message="Agent execution not found",
                            is_end_of_stream=True,
                        )
                    else:
                        yield control_pb2.ControlDataEnvelope(
                            correlation_id=corr_id,
                            status_code=200,
                            is_end_of_stream=True,
                            payload_json=agent.model_dump_json(),
                        )
                elif cmd_type == "list_agent_tools":
                    c = cmd.list_agent_tools
                    tools = await WorkflowService.list_agent_tools(
                        run_id=c.run_id,
                        execution_id=c.execution_id,
                        cursor=c.cursor if c.HasField("cursor") else None,
                        limit=c.limit if c.limit > 0 else 50,
                    )
                    yield control_pb2.ControlDataEnvelope(
                        correlation_id=corr_id,
                        status_code=200,
                        is_end_of_stream=True,
                        payload_json=tools.model_dump_json(),
                    )
                elif cmd_type == "get_tool_output":
                    c = cmd.get_tool_output
                    output = await WorkflowService.get_tool_output(
                        run_id=c.run_id,
                        tool_invocation_id=c.tool_invocation_id,
                        offset=c.offset,
                        limit=c.limit if c.limit > 0 else 1000,
                    )
                    if not output:
                        yield control_pb2.ControlDataEnvelope(
                            correlation_id=corr_id,
                            status_code=404,
                            error_message="Tool invocation not found",
                            is_end_of_stream=True,
                        )
                    else:
                        yield control_pb2.ControlDataEnvelope(
                            correlation_id=corr_id,
                            status_code=200,
                            is_end_of_stream=True,
                            payload_json=output.model_dump_json(),
                        )
                elif cmd_type == "list_delegations":
                    delegations = await WorkflowService.list_delegations(
                        cmd.list_delegations.run_id
                    )
                    yield control_pb2.ControlDataEnvelope(
                        correlation_id=corr_id,
                        status_code=200,
                        is_end_of_stream=True,
                        payload_json=json.dumps([d.model_dump(mode="json") for d in delegations]),
                    )
                elif cmd_type == "list_artifacts":
                    artifacts = await WorkflowService.list_artifacts(cmd.list_artifacts.run_id)
                    yield control_pb2.ControlDataEnvelope(
                        correlation_id=corr_id,
                        status_code=200,
                        is_end_of_stream=True,
                        payload_json=json.dumps([a.model_dump(mode="json") for a in artifacts]),
                    )
                elif cmd_type == "get_logs":
                    c = cmd.get_logs
                    if c.HasField("cursor") and c.cursor and not c.cursor.isdigit():
                        yield control_pb2.ControlDataEnvelope(
                            correlation_id=corr_id,
                            status_code=400,
                            error_message="Invalid logs cursor: must be a numeric log id.",
                            is_end_of_stream=True,
                        )
                    else:
                        logs = await WorkflowService.get_logs(
                            run_id=c.run_id,
                            cursor=c.cursor if c.HasField("cursor") else None,
                            limit=c.limit if c.limit > 0 else 100,
                            severity=c.severity if c.HasField("severity") else None,
                            task_id=c.task_id if c.HasField("task_id") else None,
                            execution_id=c.execution_id if c.HasField("execution_id") else None,
                        )
                        yield control_pb2.ControlDataEnvelope(
                            correlation_id=corr_id,
                            status_code=200,
                            is_end_of_stream=True,
                            payload_json=logs.model_dump_json(),
                        )
                elif cmd_type == "get_task_graph":
                    graph = await WorkflowService.get_task_graph(cmd.get_task_graph.run_id)
                    yield control_pb2.ControlDataEnvelope(
                        correlation_id=corr_id,
                        status_code=200,
                        is_end_of_stream=True,
                        payload_json=graph.model_dump_json(),
                    )
                elif cmd_type == "get_delegation_graph":
                    dgraph = await WorkflowService.get_delegation_graph(
                        cmd.get_delegation_graph.run_id
                    )
                    yield control_pb2.ControlDataEnvelope(
                        correlation_id=corr_id,
                        status_code=200,
                        is_end_of_stream=True,
                        payload_json=dgraph.model_dump_json(),
                    )
                elif cmd_type == "get_events_history":
                    c = cmd.get_events_history
                    history = await WorkflowService.get_events_history(
                        run_id=c.run_id,
                        after_seq=c.after_seq,
                        limit=c.limit if c.limit > 0 else 50,
                    )
                    yield control_pb2.ControlDataEnvelope(
                        correlation_id=corr_id,
                        status_code=200,
                        is_end_of_stream=True,
                        payload_json=history.model_dump_json(),
                    )
                else:
                    yield control_pb2.ControlDataEnvelope(
                        correlation_id=corr_id,
                        status_code=400,
                        error_message=f"Unknown command type: {cmd_type}",
                        is_end_of_stream=True,
                    )
            except KeyError as e:
                yield control_pb2.ControlDataEnvelope(
                    correlation_id=corr_id,
                    status_code=404,
                    error_message=str(e),
                    is_end_of_stream=True,
                )
            except ValueError as e:
                yield control_pb2.ControlDataEnvelope(
                    correlation_id=corr_id,
                    status_code=400,
                    error_message=str(e),
                    is_end_of_stream=True,
                )
            except Exception as e:
                logger.exception(f"Unhandled error processing {cmd_type}: {e}")
                yield control_pb2.ControlDataEnvelope(
                    correlation_id=corr_id,
                    status_code=500,
                    error_message=f"Internal error: {e}",
                    is_end_of_stream=True,
                )
