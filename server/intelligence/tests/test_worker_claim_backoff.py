import logging

import pytest
from redis import exceptions as redis_exceptions

from src.orchestration.runtime import worker as worker_module
from src.orchestration.runtime.worker import (
    CLAIM_BACKOFF_MAX_S,
    CLAIM_FAILURE_ESCALATION_THRESHOLD,
    DurableJobWorker,
    claim_backoff_seconds,
)


class FlakyClaimQueue:
    def __init__(self, failures: int, worker: DurableJobWorker):
        self.failures = failures
        self.calls = 0
        self.worker = worker

    async def claim_job(self, worker_id, block_ms=2000, min_idle_time=30000):
        self.calls += 1
        if self.calls <= self.failures:
            raise redis_exceptions.TimeoutError("Timeout reading from redis")
        self.worker.running = False
        return None


def test_claim_backoff_schedule_is_capped():
    assert [claim_backoff_seconds(n) for n in range(1, 7)] == [1.0, 2.0, 4.0, 8.0, 10.0, 10.0]
    assert claim_backoff_seconds(50) == CLAIM_BACKOFF_MAX_S


async def run_claim_loop(monkeypatch, failures: int) -> list[float]:
    worker = DurableJobWorker(worker_id="worker_claim_backoff_test")
    worker.queue = FlakyClaimQueue(failures, worker)
    sleeps: list[float] = []

    async def fake_initialize():
        return None

    async def fake_sleep(seconds):
        sleeps.append(seconds)

    monkeypatch.setattr(worker, "initialize", fake_initialize)
    monkeypatch.setattr(worker_module.asyncio, "sleep", fake_sleep)
    await worker.start()
    return sleeps


@pytest.mark.asyncio
async def test_single_redis_timeout_in_claim_cycle_is_a_warning(monkeypatch, caplog):
    caplog.set_level(logging.WARNING, logger="mensetsu.worker")
    sleeps = await run_claim_loop(monkeypatch, failures=1)

    claim_records = [r for r in caplog.records if "claim cycle" in r.getMessage()]
    assert [r.levelno for r in claim_records] == [logging.WARNING]
    assert sleeps[0] == 1.0


@pytest.mark.asyncio
async def test_repeated_redis_timeouts_back_off_and_escalate(monkeypatch, caplog):
    caplog.set_level(logging.WARNING, logger="mensetsu.worker")
    failures = CLAIM_FAILURE_ESCALATION_THRESHOLD + 1
    sleeps = await run_claim_loop(monkeypatch, failures=failures)

    claim_records = [r for r in caplog.records if "claim cycle" in r.getMessage()]
    levels = [r.levelno for r in claim_records]
    assert levels[: CLAIM_FAILURE_ESCALATION_THRESHOLD - 1] == [logging.WARNING] * (
        CLAIM_FAILURE_ESCALATION_THRESHOLD - 1
    )
    assert levels[CLAIM_FAILURE_ESCALATION_THRESHOLD - 1 :] == [logging.ERROR] * 2
    assert sleeps[:failures] == [claim_backoff_seconds(n) for n in range(1, failures + 1)]
    assert max(sleeps[:failures]) <= CLAIM_BACKOFF_MAX_S
