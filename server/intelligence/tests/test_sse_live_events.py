import asyncio
import contextlib
import os
from unittest.mock import patch

import pytest

import grpc
import src.orchestration.shared as shared_mod
from src.grpc.auth_interceptor import GrpcAuthInterceptor
from src.grpc.streaming_servicer import WorkflowEventStreamServicer
from src.grpc.v1 import streaming_pb2, streaming_pb2_grpc
from src.orchestration import ledger as ledger_module
from src.orchestration.ledger import SurrealLedgerRepository


class FakePubSub:
    def __init__(self, queue: asyncio.Queue):
        self.queue = queue
        self.subscribed = False

    async def subscribe(self, channel: str) -> None:
        self.subscribed = True

    async def unsubscribe(self, channel: str) -> None:
        self.subscribed = False

    async def get_message(self, ignore_subscribe_messages: bool = True, timeout: float = 1.0):
        try:
            data = await asyncio.wait_for(self.queue.get(), timeout=timeout)
            return {"type": "message", "data": data}
        except asyncio.TimeoutError:
            return None

    async def close(self) -> None:
        self.subscribed = False

    async def aclose(self) -> None:
        self.subscribed = False


class FakeRedis:
    def __init__(self, queue: asyncio.Queue):
        self.queue = queue
        self.pubsub_instance = FakePubSub(queue)

    def pubsub(self):
        return self.pubsub_instance

    async def publish(self, channel: str, message: str) -> int:
        await self.queue.put(message)
        return 1

    async def xgroup_create(self, *args, **kwargs):
        pass

    async def aclose(self):
        pass


@pytest.fixture
def fake_redis_queue():
    queue = asyncio.Queue()
    fake_client = FakeRedis(queue)
    with patch("redis.asyncio.from_url", return_value=fake_client):
        yield queue


@pytest.fixture
async def grpc_stream_server(monkeypatch):
    token = "test_internal_service_token_12345"
    monkeypatch.setenv("INTERNAL_SERVICE_TOKEN", token)

    server = grpc.aio.server(interceptors=[GrpcAuthInterceptor()])
    servicer = WorkflowEventStreamServicer()
    streaming_pb2_grpc.add_WorkflowEventStreamServiceServicer_to_server(servicer, server)
    port = server.add_insecure_port("127.0.0.1:0")
    await server.start()

    yield port, token, servicer

    await server.stop(grace=0)


async def append_task_started(ledger: SurrealLedgerRepository, run_id: str, task_id: str) -> None:
    await ledger.append_event_transactional(
        run_id=run_id,
        event_type="task.started",
        task_id=task_id,
        payload={"task_id": task_id, "attempt": 1, "assigned_agent": "planner"},
    )


async def wait_for_count(
    received: list[tuple[int, str, float]], target: int, timeout: float = 4.0
) -> None:
    deadline = asyncio.get_running_loop().time() + timeout
    while asyncio.get_running_loop().time() < deadline:
        if len(received) >= target:
            return
        await asyncio.sleep(0.05)
    raise TimeoutError(f"Expected at least {target} events, got {len(received)}")


@pytest.mark.asyncio
async def test_grpc_stream_delivers_published_events_without_duplicates(
    fake_redis_queue, grpc_stream_server
):
    port, token, servicer = grpc_stream_server
    job_queue = shared_mod.job_queue
    job_queue.client = None
    ledger_module.set_event_publisher(job_queue.publish_event)

    ledger = SurrealLedgerRepository()
    shared_mod.default_ledger = ledger
    servicer.ledger = ledger
    servicer.queue = job_queue
    from src.agents.runtime import current_ledger_var

    token_var = current_ledger_var.set(ledger)
    run_id = f"run_test_{os.urandom(4).hex()}"

    from src.contracts.v1 import WorkflowRun

    await ledger.create_workflow_run(
        WorkflowRun(
            id=run_id,
            workflow_id="interview-preparation-hybrid",
            workflow_name="Hybrid",
            workspace_id="ws",
            user_id="u",
            type="hybrid",
            status="running",
        )
    )
    await ledger.append_event_transactional(
        run_id=run_id,
        event_type="workflow.created",
        payload={"workflow_id": "test", "workspace_id": "ws", "user_id": "u", "type": "hybrid"},
    )
    history_check = await ledger.get_events_history(run_id=run_id, after_seq=0, limit=10)
    assert len(history_check.events) == 1, (
        f"Expected 1 event in history, got {len(history_check.events)}"
    )

    received: list[tuple[int, str, float]] = []

    async def client_sub():
        metadata = (("authorization", f"Bearer {token}"),)
        async with grpc.aio.insecure_channel(f"127.0.0.1:{port}") as channel:
            stub = streaming_pb2_grpc.WorkflowEventStreamServiceStub(channel)

            async def send_msg():
                yield streaming_pb2.EventStreamClientMessage(
                    subscribe=streaming_pb2.SubscribeRunMessage(run_id=run_id, after_seq=0)
                )
                while True:
                    await asyncio.sleep(1.0)

            call = stub.StreamWorkflowEvents(send_msg(), metadata=metadata)
            async for frame in call:
                received.append((frame.seq, frame.event_type, asyncio.get_running_loop().time()))
                if frame.is_terminal:
                    break

    reader = asyncio.create_task(client_sub())
    try:
        await wait_for_count(received, 1, timeout=4.0)
        assert received[0][0] == 1
        assert received[0][1] == "workflow.created"

        await append_task_started(ledger, run_id, "task_a")
        await append_task_started(ledger, run_id, "task_b")
        await wait_for_count(received, 3, timeout=4.0)

        assert [s for s, _, _ in received] == [1, 2, 3]

        await ledger.append_event_transactional(
            run_id=run_id,
            event_type="workflow.completed",
            payload={"duration_ms": 1.0, "total_tasks": 2, "artifacts_count": 0},
        )
        await wait_for_count(received, 4, timeout=4.0)
        assert [s for s, _, _ in received] == [1, 2, 3, 4]
        assert received[-1][1] == "workflow.completed"
    finally:
        reader.cancel()
        with contextlib.suppress(BaseException):
            await reader
        current_ledger_var.reset(token_var)
