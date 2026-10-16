import pytest

import grpc
from src.grpc.auth_interceptor import GrpcAuthInterceptor
from src.grpc.control_servicer import DataControlServicer
from src.grpc.health_servicer import HealthServicer
from src.grpc.v1 import control_pb2, control_pb2_grpc, health_pb2, health_pb2_grpc


@pytest.fixture
async def ephemeral_grpc_server(monkeypatch):
    token = "test_internal_service_token_12345"
    monkeypatch.setenv("INTERNAL_SERVICE_TOKEN", token)

    server = grpc.aio.server(interceptors=[GrpcAuthInterceptor()])
    control_pb2_grpc.add_DataControlServiceServicer_to_server(DataControlServicer(), server)
    health_pb2_grpc.add_HealthServicer_to_server(HealthServicer(), server)
    port = server.add_insecure_port("127.0.0.1:0")
    await server.start()

    yield port, token

    await server.stop(grace=0)


@pytest.mark.asyncio
async def test_health_check_exempt_from_auth(ephemeral_grpc_server):
    port, _ = ephemeral_grpc_server
    async with grpc.aio.insecure_channel(f"127.0.0.1:{port}") as channel:
        stub = health_pb2_grpc.HealthStub(channel)
        # Calling without any authorization metadata should succeed (status SERVING or NOT_SERVING)
        resp = await stub.Check(health_pb2.HealthCheckRequest(service=""))
        assert resp.status in (
            health_pb2.HealthCheckResponse.ServingStatus.SERVING,
            health_pb2.HealthCheckResponse.ServingStatus.NOT_SERVING,
        )


@pytest.mark.asyncio
async def test_control_stream_requires_valid_bearer_token(ephemeral_grpc_server):
    port, token = ephemeral_grpc_server

    # 1. Calling without metadata -> UNAUTHENTICATED
    async with grpc.aio.insecure_channel(f"127.0.0.1:{port}") as channel:
        stub = control_pb2_grpc.DataControlServiceStub(channel)

        async def msg_gen():
            yield control_pb2.ControlCommand(
                correlation_id="c1",
                list_workflows=control_pb2.ListWorkflowsCommand(),
            )

        call = stub.StreamControl(msg_gen())
        with pytest.raises(grpc.aio.AioRpcError) as exc_info:
            async for _ in call:
                pass
        assert exc_info.value.code() == grpc.StatusCode.UNAUTHENTICATED

    # 2. Calling with invalid token -> UNAUTHENTICATED
    async with grpc.aio.insecure_channel(f"127.0.0.1:{port}") as channel:
        stub = control_pb2_grpc.DataControlServiceStub(channel)

        async def msg_gen():
            yield control_pb2.ControlCommand(
                correlation_id="c2",
                list_workflows=control_pb2.ListWorkflowsCommand(),
            )

        metadata = (("authorization", "Bearer invalid_token_xyz"),)
        call = stub.StreamControl(msg_gen(), metadata=metadata)
        with pytest.raises(grpc.aio.AioRpcError) as exc_info:
            async for _ in call:
                pass
        assert exc_info.value.code() == grpc.StatusCode.UNAUTHENTICATED

    # 3. Calling with valid token -> SUCCESS (returns definitions)
    async with grpc.aio.insecure_channel(f"127.0.0.1:{port}") as channel:
        stub = control_pb2_grpc.DataControlServiceStub(channel)

        async def msg_gen():
            yield control_pb2.ControlCommand(
                correlation_id="c3",
                list_workflows=control_pb2.ListWorkflowsCommand(),
            )

        metadata = (("authorization", f"Bearer {token}"),)
        call = stub.StreamControl(msg_gen(), metadata=metadata)
        envelopes = []
        async for env in call:
            envelopes.append(env)

        assert len(envelopes) == 1
        assert envelopes[0].correlation_id == "c3"
        assert envelopes[0].status_code == 200
        assert "interview-preparation-hybrid" in envelopes[0].payload_json
