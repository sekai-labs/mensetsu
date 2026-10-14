import asyncio
import uuid

import pytest

from src.orchestration.runtime.worker import DurableJobWorker


@pytest.mark.asyncio
async def test_worker_readiness_signal_lifecycle():

    worker_id = f"worker_ready_{uuid.uuid4().hex[:8]}"
    worker = DurableJobWorker(worker_id=worker_id)

    r_client = await worker.queue.get_client()
    alive_key = f"mensetsu:worker:{worker_id}:alive"
    assert await r_client.get(alive_key) is None

    await worker.initialize()
    val = await r_client.get(alive_key)
    assert val is not None
    ttl = await r_client.ttl(alive_key)
    assert 0 < ttl <= 15

    await asyncio.sleep(5.5)
    val2 = await r_client.get(alive_key)
    assert val2 is not None
    assert val2 >= val

    await worker.stop()
    assert await r_client.get(alive_key) is None
    await r_client.aclose()
