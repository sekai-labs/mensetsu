import asyncio
import json
import uuid

import pytest
import pytest_asyncio

from src.orchestration import service as service_module
from src.orchestration.ledger import IdempotencyConflict
from src.orchestration.redis_queue import RedisStreamJobQueue
from src.orchestration.service import WorkflowService


@pytest_asyncio.fixture
async def idempotent_queue(monkeypatch):
    suffix = uuid.uuid4().hex
    queue = RedisStreamJobQueue(
        stream_key=f"test_idempotency_{suffix}",
        group_name=f"test_idempotency_group_{suffix}",
    )
    monkeypatch.setattr(service_module, "job_queue", queue)
    yield queue
    await queue.close()


def request_args(user_id: str, workspace_id: str, company_name: str = "Acme"):
    return {
        "workspace_id": workspace_id,
        "user_id": user_id,
        "workflow_id": "interview-preparation-static",
        "company_name": company_name,
        "role_title": "Engineer",
        "company_urls": ["https://acme.example"],
        "resume_text": "Candidate resume",
        "job_description": "Role description",
    }


@pytest.mark.asyncio
async def test_same_key_and_request_replays_original_run_without_duplicate_job(idempotent_queue):
    args = request_args(f"usr_{uuid.uuid4().hex}", f"ws_{uuid.uuid4().hex}")
    first = await WorkflowService.create_run(**args, idempotency_key="retry-key")
    replay = await WorkflowService.create_run(**args, idempotency_key="retry-key")
    assert replay.id == first.id
    history = await service_module.get_ledger().get_events_history(
        run_id=first.id, after_seq=0, limit=10
    )
    assert [event.type for event in history.events] == ["workflow.created"]


@pytest.mark.asyncio
async def test_same_key_with_different_payload_returns_conflict(idempotent_queue):
    args = request_args(f"usr_{uuid.uuid4().hex}", f"ws_{uuid.uuid4().hex}")
    await WorkflowService.create_run(**args, idempotency_key="retry-key")
    with pytest.raises(IdempotencyConflict):
        await WorkflowService.create_run(
            **{**args, "company_name": "Different Company"}, idempotency_key="retry-key"
        )
    client = await idempotent_queue.get_client()
    assert await client.xlen(idempotent_queue.stream_key) == 1


@pytest.mark.asyncio
async def test_concurrent_identical_requests_create_one_run_and_job(idempotent_queue):
    args = request_args(f"usr_{uuid.uuid4().hex}", f"ws_{uuid.uuid4().hex}")
    runs = await asyncio.gather(
        *(WorkflowService.create_run(**args, idempotency_key="concurrent-key") for _ in range(8))
    )
    assert len({run.id for run in runs}) == 1
    client = await idempotent_queue.get_client()
    assert await client.xlen(idempotent_queue.stream_key) == 1


@pytest.mark.asyncio
async def test_retry_after_enqueue_failure_reuses_reservation_and_enqueues_once(
    idempotent_queue, monkeypatch
):
    args = request_args(f"usr_{uuid.uuid4().hex}", f"ws_{uuid.uuid4().hex}")
    enqueue = idempotent_queue.enqueue_job
    calls = 0

    async def lose_first_enqueue_response(*args, **kwargs):
        nonlocal calls
        calls += 1
        message_id = await enqueue(*args, **kwargs)
        if calls == 1:
            raise RuntimeError("simulated response loss after enqueue")
        return message_id

    monkeypatch.setattr(idempotent_queue, "enqueue_job", lose_first_enqueue_response)
    with pytest.raises(RuntimeError, match="response loss after enqueue"):
        await WorkflowService.create_run(**args, idempotency_key="partial-failure-key")
    run = await WorkflowService.create_run(**args, idempotency_key="partial-failure-key")
    replay = await WorkflowService.create_run(**args, idempotency_key="partial-failure-key")
    assert replay.id == run.id
    client = await idempotent_queue.get_client()
    assert await client.xlen(idempotent_queue.stream_key) == 1


@pytest.mark.asyncio
async def test_same_key_is_scoped_to_user_and_workspace(idempotent_queue):
    user_a = f"usr_{uuid.uuid4().hex}"
    workspace_a = f"ws_{uuid.uuid4().hex}"
    first = await WorkflowService.create_run(
        **request_args(user_a, workspace_a), idempotency_key="scoped-key"
    )
    other_user = await WorkflowService.create_run(
        **request_args(f"usr_{uuid.uuid4().hex}", workspace_a), idempotency_key="scoped-key"
    )
    other_workspace = await WorkflowService.create_run(
        **request_args(user_a, f"ws_{uuid.uuid4().hex}"), idempotency_key="scoped-key"
    )
    assert len({first.id, other_user.id, other_workspace.id}) == 3
    client = await idempotent_queue.get_client()
    assert await client.xlen(idempotent_queue.stream_key) == 3


@pytest.mark.asyncio
async def test_grpc_create_run_replays_and_returns_conflict(idempotent_queue):
    from src.grpc.control_servicer import DataControlServicer
    from src.grpc.v1 import control_pb2

    user_id = f"usr_{uuid.uuid4().hex}"
    workspace_id = f"ws_{uuid.uuid4().hex}"

    class Context:
        async def send_initial_metadata(self, metadata):
            return None

    async def dispatch(company_name: str):
        command = control_pb2.ControlCommand(
            correlation_id=uuid.uuid4().hex,
            create_run=control_pb2.CreateRunCommand(
                workspace_id=workspace_id,
                user_id=user_id,
                workflow_id="interview-preparation-static",
                company_name=company_name,
                idempotency_key="grpc-key",
            ),
        )

        async def commands():
            yield command

        return [
            envelope
            async for envelope in DataControlServicer().StreamControl(commands(), Context())
        ][0]

    first = await dispatch("Acme")
    replay = await dispatch("Acme")
    mismatch = await dispatch("Different")
    assert first.status_code == 201
    assert json.loads(replay.payload_json)["id"] == json.loads(first.payload_json)["id"]
    assert mismatch.status_code == 409
    assert "different request" in mismatch.error_message
    client = await idempotent_queue.get_client()
    assert await client.xlen(idempotent_queue.stream_key) == 1
