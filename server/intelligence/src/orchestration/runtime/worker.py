import asyncio
import logging
import os
import signal
import socket
import time
import uuid
from datetime import datetime, timezone
from typing import Optional

from langgraph.checkpoint.redis.aio import AsyncRedisSaver
from redis import exceptions as redis_exceptions

from src.contracts.v1 import WorkflowProgress
from src.orchestration.dynamic_workflow import DynamicWorkflowState, build_dynamic_workflow
from src.orchestration.ledger import SurrealLedgerRepository, set_event_publisher
from src.orchestration.redis_queue import CLAIM_BLOCK_MS, REDIS_CLIENT_OPTIONS, RedisStreamJobQueue
from src.orchestration.static_workflow import StaticWorkflowState, build_static_workflow

STATIC_TASK_COUNT = 9
RETRYABLE_STATUS_CODES = {408, 409, 425, 429, 500, 502, 503, 504, 520, 521, 522, 523, 524, 529}
RETRYABLE_NAME_MARKERS = (
    "Timeout",
    "Connection",
    "RateLimit",
    "InternalServer",
    "APIConnectionError",
    "APITimeoutError",
    "ServiceUnavailable",
    "GatewayTimeout",
)


def is_retryable_error(err: BaseException) -> bool:
    if isinstance(err, (asyncio.TimeoutError, TimeoutError, ConnectionError)):
        return True
    status_code = getattr(err, "status_code", None)
    if status_code is None:
        status_code = getattr(getattr(err, "response", None), "status_code", None)
    if isinstance(status_code, int):
        return status_code in RETRYABLE_STATUS_CODES
    name = err.__class__.__name__
    text = str(err)
    if any(marker in name or marker in text for marker in RETRYABLE_NAME_MARKERS):
        return True
    if any(f"Error code: {code}" in text or f"status {code}" in text for code in RETRYABLE_STATUS_CODES):
        return True
    return "redis" in getattr(err.__class__, "__module__", "")


logger = logging.getLogger("mensetsu.worker")

TRANSIENT_CLAIM_ERRORS = (
    redis_exceptions.TimeoutError,
    redis_exceptions.ConnectionError,
    asyncio.TimeoutError,
    TimeoutError,
    ConnectionError,
)
CLAIM_BACKOFF_MAX_S = 10.0
CLAIM_FAILURE_ESCALATION_THRESHOLD = 5


def claim_backoff_seconds(consecutive_failures: int) -> float:
    return min(CLAIM_BACKOFF_MAX_S, float(2 ** (consecutive_failures - 1)))


class DurableJobWorker:
    def __init__(
        self,
        worker_id: Optional[str] = None,
        max_concurrent_jobs: int = 3,
        redis_url: Optional[str] = None,
        max_attempts: int = 3,
        heartbeat_interval_s: Optional[float] = None,
        lease_ttl_s: Optional[float] = None,
        min_idle_time_ms: Optional[int] = None,
    ):
        self.worker_id = worker_id or f"{socket.gethostname()}-{os.getpid()}"
        self.max_concurrent_jobs = max_concurrent_jobs
        self.max_attempts = max_attempts
        self.redis_url = redis_url or os.environ.get("REDIS_URL", "redis://127.0.0.1:16379")
        self.heartbeat_interval_s = (
            heartbeat_interval_s
            if heartbeat_interval_s is not None
            else float(os.environ.get("WORKER_HEARTBEAT_INTERVAL_S", "5.0"))
        )
        self.lease_ttl_s = (
            lease_ttl_s
            if lease_ttl_s is not None
            else float(os.environ.get("WORKER_LEASE_TTL_S", "20.0"))
        )
        self.min_idle_time_ms = (
            min_idle_time_ms
            if min_idle_time_ms is not None
            else int(os.environ.get("WORKER_MIN_IDLE_TIME_MS", "15000"))
        )
        self.running = False
        self._semaphore = asyncio.Semaphore(max_concurrent_jobs)
        self.queue = RedisStreamJobQueue(redis_url=self.redis_url)
        self.ledger = SurrealLedgerRepository()
        self._saver: Optional[AsyncRedisSaver] = None
        self._in_flight_tasks: set[asyncio.Task] = set()
        self._readiness_task: Optional[asyncio.Task] = None

    async def initialize(self) -> None:

        client = await self.queue.get_client()
        await client.ping()
        await self.ledger.db.execute_query("INFO FOR DB;")
        set_event_publisher(self.queue.publish_event)
        checkpoint_ttl_min = int(os.environ.get("CHECKPOINT_TTL_MINUTES", str(7 * 24 * 60)))
        saver_ttl = {"default_ttl": checkpoint_ttl_min, "refresh_on_read": True}
        self._saver = AsyncRedisSaver(
            redis_url=self.redis_url, ttl=saver_ttl, connection_args=dict(REDIS_CLIENT_OPTIONS)
        )
        await self._saver.asetup()
        await self._saver.aset_client_info()
        self.running = True

        alive_key = f"mensetsu:worker:{self.worker_id}:alive"
        await client.set(alive_key, datetime.now(timezone.utc).isoformat(), ex=3)

        async def readiness_loop():
            while self.running:
                await asyncio.sleep(1.0)
                try:
                    c = await self.queue.get_client()
                    await c.set(alive_key, datetime.now(timezone.utc).isoformat(), ex=3)
                except Exception as re:
                    logger.debug(f"Readiness signal refresh error: {re}")

        self._readiness_task = asyncio.create_task(readiness_loop())

    async def execute_job(self, job_data: dict) -> None:
        run_id = job_data["run_id"]
        msg_id = job_data["msg_id"]
        payload = job_data["payload"]
        is_reclaimed = job_data.get("reclaimed", False)
        lease_token = str(uuid.uuid4())

        ttl_ms = int(self.lease_ttl_s * 1000)
        acquired = await self.queue.acquire_lease(run_id, lease_token, ttl_ms=ttl_ms)
        if not acquired:
            logger.warning(
                f"Worker [{self.worker_id}] failed to acquire atomic lease for run {run_id}. Live owner exists. Leaving message without executing."
            )
            return

        lost_lease_event = asyncio.Event()

        async def heartbeat_loop():
            consecutive_exceptions = 0
            last_successful_renewal = time.monotonic()
            lease_ttl_seconds = ttl_ms / 1000.0

            while not lost_lease_event.is_set() and self.running:
                await asyncio.sleep(self.heartbeat_interval_s)
                if (time.monotonic() - last_successful_renewal) >= lease_ttl_seconds:
                    logger.error(
                        f"Worker [{self.worker_id}] lease expired without renewal for run {run_id}. Halting work."
                    )
                    lost_lease_event.set()
                    break

                try:
                    renewed = await self.queue.refresh_lease(run_id, lease_token, ttl_ms=ttl_ms)
                    if not renewed:
                        renewed = await self.queue.refresh_lease(run_id, lease_token, ttl_ms=ttl_ms)
                    if not renewed:
                        logger.error(
                            f"Worker [{self.worker_id}] lost lease for run {run_id}. Halting work."
                        )
                        lost_lease_event.set()
                        break
                    last_successful_renewal = time.monotonic()
                    consecutive_exceptions = 0
                    await self.queue.reset_stream_idle(self.worker_id, msg_id)
                except Exception as he:
                    consecutive_exceptions += 1
                    logger.warning(f"Heartbeat renewal error: {he}")
                    if consecutive_exceptions >= 3:
                        logger.error(
                            f"Worker [{self.worker_id}] hit {consecutive_exceptions} heartbeat exceptions for run {run_id}. Halting work."
                        )
                        lost_lease_event.set()
                        break
                    if (time.monotonic() - last_successful_renewal) >= lease_ttl_seconds:
                        lost_lease_event.set()
                        break

        hb_task = asyncio.create_task(heartbeat_loop())

        try:
            from src.agents.runtime import current_ledger_var, current_lost_lease_var

            current_ledger_var.set(self.ledger)
            current_lost_lease_var.set(lost_lease_event)

            run = await self.ledger.get_workflow_run(run_id)
            if run and run.status == "cancelled":
                logger.info(f"Run {run_id} is cancelled. Acknowledging message.")
                await self.queue.ack_job(msg_id)
                return

            if run and run.status in ("pending", "queued") and not run.started_at:
                wf_type = payload.get("workflow_id", "interview-preparation-hybrid")
                is_static = (wf_type == "interview-preparation-static")
                is_hybrid = (wf_type == "interview-preparation-hybrid")
                planned = STATIC_TASK_COUNT if is_static else (6 if is_hybrid else 0)
                await self.ledger.update_workflow_run(
                    run_id,
                    {
                        "status": "running",
                        "started_at": datetime.now(timezone.utc),
                        "current_phase": "running",
                        "progress": WorkflowProgress(
                            total_tasks=planned,
                            pending=planned,
                            percentage=0,
                            estimate=True,
                            phase_index=0,
                            phase_count=planned or None,
                        ),
                    },
                )
                await self.ledger.append_event_transactional(
                    run_id=run_id,
                    event_type="workflow.started",
                    payload={
                        "plan_revision": run.plan_revision or 1,
                        "initial_tasks_count": planned,
                    },
                )
                await self._set_workspace_progress(payload, "preparing_input", 2)

            wf_id = payload.get("workflow_id", "interview-preparation-static")
            config = {"configurable": {"thread_id": run_id}}
            existing_checkpoint = await self._saver.aget_tuple(config) if self._saver else None

            if is_reclaimed or existing_checkpoint is not None:
                from_step = 1
                if existing_checkpoint and existing_checkpoint.metadata:
                    from_step = int(existing_checkpoint.metadata.get("step", 1))

                current_tasks = await self.ledger.list_tasks_for_run(run_id)
                non_completed = len([t for t in current_tasks if t.status != "completed"])

                await self.ledger.append_event_transactional(
                    run_id=run_id,
                    event_type="execution.resumed",
                    payload={"thread_id": run_id, "from_step": from_step},
                )
                await self.ledger.append_event_transactional(
                    run_id=run_id,
                    event_type="worker.recovered",
                    payload={
                        "worker_id": self.worker_id,
                        "run_id": run_id,
                        "resumed_tasks_count": max(1, non_completed),
                    },
                )
            if wf_id == "interview-preparation-static":
                from src.agents.runtime import current_ledger_var
                current_ledger_var.set(self.ledger)
                graph = build_static_workflow(
                    checkpointer=self._saver, ledger=self.ledger, db_client=self.ledger.db
                )
                initial_state = StaticWorkflowState(
                    run_id=run_id,
                    workspace_id=payload["workspace_id"],
                    company_name=payload["company_name"],
                    role_title=payload["role_title"],
                    company_urls=payload.get("company_urls", []),
                    resume_text=payload.get("resume_text", ""),
                    resume_pdf_base64=payload.get("resume_pdf_base64"),
                    resume_filename=payload.get("resume_filename"),
                    job_description=payload.get("job_description", ""),
                    jd_url=payload.get("jd_url"),
                    seniority=payload.get("seniority"),
                )
            elif wf_id == "interview-preparation-hybrid":
                from src.orchestration.hybrid_workflow import HybridWorkflowState, build_hybrid_workflow
                graph = build_hybrid_workflow(checkpointer=self._saver, ledger=self.ledger)
                initial_state = HybridWorkflowState(
                    run_id=run_id,
                    workspace_id=payload["workspace_id"],
                    company_name=payload["company_name"],
                    role_title=payload["role_title"],
                    company_urls=payload.get("company_urls", []),
                    resume_text=payload.get("resume_text", ""),
                    resume_pdf_base64=payload.get("resume_pdf_base64"),
                    resume_filename=payload.get("resume_filename"),
                    job_description=payload.get("job_description", ""),
                    jd_url=payload.get("jd_url"),
                    seniority=payload.get("seniority"),
                    user_prompt=payload.get("user_prompt"),
                    workflow_mode=payload.get("workflow_mode", "hybrid"),
                    llm_config=payload.get("llm_config"),
                    plan_revision=int(payload.get("plan_revision", 1)),
                )
            else:
                graph = build_dynamic_workflow(checkpointer=self._saver, ledger=self.ledger)
                initial_state = DynamicWorkflowState(
                    run_id=run_id,
                    workspace_id=payload["workspace_id"],
                    company_name=payload["company_name"],
                    role_title=payload["role_title"],
                    company_urls=payload.get("company_urls", []),
                    resume_text=payload.get("resume_text", ""),
                    resume_pdf_base64=payload.get("resume_pdf_base64"),
                    resume_filename=payload.get("resume_filename"),
                    job_description=payload.get("job_description", ""),
                    jd_url=payload.get("jd_url"),
                    seniority=payload.get("seniority"),
                )
            input_val = None if existing_checkpoint is not None else initial_state
            await graph.ainvoke(input_val, config=config)

            run_obj = await self.ledger.get_workflow_run(run_id)
            if not run_obj or run_obj.status != "completed":
                raise RuntimeError(
                    f"Workflow execution graph for {run_id} terminated without completing finalize node."
                )

            await self.queue.ack_job(msg_id)
        except Exception as e:
            err_str = str(e)
            logger.error(f"Job execution failed for {run_id}: {e}")

            if lost_lease_event.is_set():
                logger.warning(f"Lease was lost during execution of {run_id}. Not acknowledging.")
                return
            run = await self.ledger.get_workflow_run(run_id)
            if run and run.status == "cancelled":
                await self.queue.ack_job(msg_id)
                return

            exc_name = e.__class__.__name__
            orig_e = getattr(e, "__cause__", None) or e
            is_transient = is_retryable_error(e) or is_retryable_error(orig_e)

            current_attempt = int(payload.get("attempt_count", 1))

            if is_transient and current_attempt < self.max_attempts:
                delay_ms = int(1000 * (2 ** (current_attempt - 1)))
                logger.info(
                    f"Transient failure for {run_id} (exc={exc_name}). Attempt {current_attempt}/{self.max_attempts}. Re-scheduling with delay {delay_ms}ms."
                )
                await self.ledger.append_event_transactional(
                    run_id=run_id,
                    event_type="workflow.retrying",
                    payload={
                        "attempt": current_attempt,
                        "next_attempt": current_attempt + 1,
                        "delay_ms": delay_ms,
                        "error_category": exc_name,
                    },
                )

                await self.queue.ack_job(msg_id)
                new_payload = dict(payload)
                new_payload["attempt_count"] = current_attempt + 1
                await self.queue.enqueue_job(run_id, new_payload, delay_ms=delay_ms)
            else:
                if run:
                    failed_at = datetime.now(timezone.utc)
                    run_updates = {
                        "status": "failed",
                        "current_phase": "failed",
                        "error_message": err_str,
                        "completed_at": failed_at,
                    }
                    if run.started_at:
                        run_updates["duration_ms"] = (
                            failed_at - run.started_at
                        ).total_seconds() * 1000
                    await self.ledger.update_workflow_run(run_id, run_updates)
                try:
                    await self.ledger.sweep_run_to_terminal(run_id, reason="run failed")
                except Exception as se:
                    logger.error(f"Run-end sweep failed for {run_id}: {se}")
                await self.ledger.append_event_transactional(
                    run_id=run_id,
                    event_type="workflow.failed",
                    payload={"error": err_str, "failed_task_id": None},
                )
                await self._set_workspace_progress(payload, "failed", 0, error_message=err_str)
                logger.info(
                    f"Run {run_id} failed; workspace status is only downgraded if it was not already ready."
                )
                await self.queue.ack_job(msg_id)
        finally:
            lost_lease_event.set()
            hb_task.cancel()
            try:
                await self.queue.release_lease(run_id, lease_token)
            except Exception:
                pass

    async def start(self) -> None:

        await self.initialize()
        self.running = True
        logger.info(
            f"DurableJobWorker [{self.worker_id}] started with max_concurrency={self.max_concurrent_jobs}"
        )

        consecutive_claim_failures = 0
        while self.running:
            await self._semaphore.acquire()
            try:
                job_data = await self.queue.claim_job(
                    self.worker_id, block_ms=CLAIM_BLOCK_MS, min_idle_time=self.min_idle_time_ms
                )
                consecutive_claim_failures = 0
                if job_data:
                    task = asyncio.create_task(self._wrapped_execute(job_data))
                    self._in_flight_tasks.add(task)
                    task.add_done_callback(self._in_flight_tasks.discard)
                else:
                    self._semaphore.release()
                    await asyncio.sleep(1.0)
            except TRANSIENT_CLAIM_ERRORS as e:
                self._semaphore.release()
                consecutive_claim_failures += 1
                backoff_s = claim_backoff_seconds(consecutive_claim_failures)
                log = (
                    logger.error
                    if consecutive_claim_failures >= CLAIM_FAILURE_ESCALATION_THRESHOLD
                    else logger.warning
                )
                log(
                    f"Transient redis error in claim cycle ({e.__class__.__name__}: {e}); "
                    f"consecutive failures={consecutive_claim_failures}, retrying in {backoff_s:.0f}s"
                )
                await asyncio.sleep(backoff_s)
            except Exception as e:
                self._semaphore.release()
                logger.error(f"Error in claim cycle: {e}")
                await asyncio.sleep(2.0)

    async def _set_workspace_progress(
        self, payload: dict, phase: str, percent: int, error_message: str | None = None
    ) -> None:
        workspace_id = payload.get("workspace_id")
        if not workspace_id:
            return
        from src.generation.progress import update_workspace_progress

        try:
            await update_workspace_progress(
                workspace_id=workspace_id,
                phase=phase,
                percent=percent,
                error_message=error_message,
                db=self.ledger.db,
            )
        except Exception as pe:
            logger.warning(f"Could not update workspace progress for {workspace_id}: {pe}")

    async def _wrapped_execute(self, job_data: dict) -> None:
        try:
            await self.execute_job(job_data)
        finally:
            self._semaphore.release()

    async def stop(self) -> None:

        logger.info(f"Stopping worker [{self.worker_id}]...")
        self.running = False
        if self._readiness_task:
            self._readiness_task.cancel()
        try:
            c = await self.queue.get_client()
            await c.delete(f"mensetsu:worker:{self.worker_id}:alive")
        except Exception:
            pass
        if self._in_flight_tasks:
            logger.info(f"Draining {len(self._in_flight_tasks)} in-flight tasks...")
            await asyncio.wait(self._in_flight_tasks, timeout=15.0)
        await self.queue.close()
        self._saver = None
        logger.info(f"Worker [{self.worker_id}] stopped cleanly.")


def _require_llm_env() -> None:
    missing = [
        k
        for k in ("LLM_OPENAI_BASE_URL", "LLM_OPENAI_API_KEY")
        if not os.environ.get(k, "").strip()
    ]
    if missing:
        raise RuntimeError(
            f"{', '.join(missing)} unset or empty. Worker refusing to start (fail-closed)."
        )


async def run_worker_cli():
    logging.basicConfig(
        level=logging.INFO, format="%(asctime)s [%(levelname)s] %(name)s: %(message)s"
    )
    _require_llm_env()
    worker = DurableJobWorker()

    loop = asyncio.get_running_loop()
    for sig in (signal.SIGINT, signal.SIGTERM):
        try:
            loop.add_signal_handler(sig, lambda: asyncio.create_task(worker.stop()))
        except NotImplementedError:
            pass

    await worker.start()


if __name__ == "__main__":
    asyncio.run(run_worker_cli())
