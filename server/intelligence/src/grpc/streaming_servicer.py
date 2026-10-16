import asyncio
import contextlib
import json
import logging
from typing import AsyncIterator

import grpc
from src.grpc.v1 import streaming_pb2, streaming_pb2_grpc
from src.orchestration.shared import get_ledger, job_queue

logger = logging.getLogger("mensetsu.grpc.streaming")
SSE_PING_INTERVAL_S = 15.0
SSE_IDLE_REPLAY_S = 5.0
TERMINAL_EVENT_TYPES = ("workflow.completed", "workflow.failed", "workflow.cancelled")


class WorkflowEventStreamServicer(streaming_pb2_grpc.WorkflowEventStreamServiceServicer):
    def __init__(self, ledger=None, queue=None):
        self.ledger = ledger
        self.queue = queue

    async def StreamWorkflowEvents(
        self,
        request_iterator: AsyncIterator[streaming_pb2.EventStreamClientMessage],
        context: grpc.aio.ServicerContext,
    ) -> AsyncIterator[streaming_pb2.ExecutionEventFrame]:
        if hasattr(context, "send_initial_metadata"):
            await context.send_initial_metadata(())
        out_queue: asyncio.Queue[streaming_pb2.ExecutionEventFrame | None] = asyncio.Queue(
            maxsize=1000
        )
        active_sub_task: asyncio.Task | None = None
        current_run_id: str | None = None
        last_ack_seq: int = 0
        active_ledger = self.ledger if self.ledger is not None else get_ledger()
        active_queue = self.queue if self.queue is not None else job_queue

        async def run_subscriber(run_id: str, start_seq: int):
            curr_seq = start_seq

            async def missed_events() -> AsyncIterator[streaming_pb2.ExecutionEventFrame]:
                nonlocal curr_seq
                while True:
                    history = await active_ledger.get_events_history(
                        run_id=run_id, after_seq=curr_seq, limit=100
                    )
                    for evt in history.events:
                        if evt.seq > curr_seq:
                            curr_seq = evt.seq
                            is_term = evt.type in TERMINAL_EVENT_TYPES
                            yield streaming_pb2.ExecutionEventFrame(
                                seq=evt.seq,
                                run_id=run_id,
                                event_type=evt.type,
                                task_id=evt.task_id or "",
                                execution_id=evt.execution_id or "",
                                payload_json=evt.model_dump_json(),
                                timestamp=evt.ts.isoformat()
                                if hasattr(evt.ts, "isoformat")
                                else str(evt.ts),
                                is_terminal=is_term,
                            )
                    if history.next_after_seq is None or not history.events:
                        return

            loop = asyncio.get_running_loop()
            live_gen = active_queue.subscribe_events(run_id)
            next_event: asyncio.Future | None = asyncio.ensure_future(live_gen.__anext__())
            next_ping_at = loop.time() + SSE_PING_INTERVAL_S
            needs_replay = True

            try:
                while True:
                    if needs_replay:
                        needs_replay = False
                        async for frame in missed_events():
                            await out_queue.put(frame)
                            if frame.is_terminal:
                                return

                    if loop.time() >= next_ping_at:
                        next_ping_at = loop.time() + SSE_PING_INTERVAL_S
                        needs_replay = True
                        continue

                    if next_event is None:
                        next_event = asyncio.ensure_future(live_gen.__anext__())

                    wait_s = min(SSE_IDLE_REPLAY_S, max(0.0, next_ping_at - loop.time()))
                    done, _ = await asyncio.wait({next_event}, timeout=wait_s)

                    if not done or next_event not in done:
                        needs_replay = True
                        continue

                    event_str = next_event.result()
                    next_event = None
                    evt_data = json.loads(event_str)
                    seq = evt_data.get("seq")
                    if not isinstance(seq, int) or seq <= curr_seq:
                        continue
                    if seq != curr_seq + 1:
                        needs_replay = True
                        continue

                    curr_seq = seq
                    evt_type = evt_data.get("type", "")
                    is_term = evt_type in TERMINAL_EVENT_TYPES
                    frame = streaming_pb2.ExecutionEventFrame(
                        seq=seq,
                        run_id=run_id,
                        event_type=evt_type,
                        task_id=evt_data.get("task_id", "") or "",
                        execution_id=evt_data.get("execution_id", "") or "",
                        payload_json=event_str,
                        timestamp=evt_data.get("timestamp", "")
                        or evt_data.get("created_at", "")
                        or "",
                        is_terminal=is_term,
                    )
                    await out_queue.put(frame)
                    if is_term:
                        return
            except asyncio.CancelledError:
                raise
            except StopAsyncIteration:
                return
            except Exception as e:
                logger.exception(f"Subscription loop for {run_id} ended: {e}")
                if next_event is not None and not next_event.done():
                    next_event.cancel()
                    with contextlib.suppress(BaseException):
                        await next_event
                await live_gen.aclose()

        async def reader_loop():
            nonlocal active_sub_task, current_run_id, last_ack_seq
            try:
                async for client_msg in request_iterator:
                    kind = client_msg.WhichOneof("message")
                    if kind == "subscribe":
                        sub = client_msg.subscribe
                        if active_sub_task and not active_sub_task.done():
                            active_sub_task.cancel()
                            with contextlib.suppress(BaseException):
                                await active_sub_task
                        current_run_id = sub.run_id
                        active_sub_task = asyncio.create_task(
                            run_subscriber(sub.run_id, sub.after_seq)
                        )
                    elif kind == "unsubscribe":
                        if active_sub_task and not active_sub_task.done():
                            active_sub_task.cancel()
                            with contextlib.suppress(BaseException):
                                await active_sub_task
                        current_run_id = None
                    elif kind == "ack":
                        last_ack_seq = client_msg.ack.ack_seq
                    elif kind == "ping":
                        ping_frame = streaming_pb2.ExecutionEventFrame(
                            seq=0,
                            run_id=current_run_id or "",
                            event_type="ping",
                            task_id="",
                            execution_id="",
                            payload_json=json.dumps({"ping_timestamp": client_msg.ping.timestamp}),
                            timestamp="",
                            is_terminal=False,
                        )
                        await out_queue.put(ping_frame)
            except Exception as e:
                logger.exception(f"Reader loop encountered error: {e}")
                if active_sub_task and not active_sub_task.done():
                    active_sub_task.cancel()
                    with contextlib.suppress(BaseException):
                        await active_sub_task
                await out_queue.put(None)

        reader_task = asyncio.create_task(reader_loop())

        try:
            while True:
                item = await out_queue.get()
                if item is None:
                    break
                yield item
        finally:
            if not reader_task.done():
                reader_task.cancel()
                with contextlib.suppress(BaseException):
                    await reader_task
            if active_sub_task and not active_sub_task.done():
                active_sub_task.cancel()
                with contextlib.suppress(BaseException):
                    await active_sub_task
