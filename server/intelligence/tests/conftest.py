import os
import uuid
from pathlib import Path

import httpx
import pytest
import pytest_asyncio
import redis.asyncio as aioredis

SESSION_ID = uuid.uuid4().hex[:8]
TEST_NS = f"test_ns_sess_{SESSION_ID}"
TEST_DB = f"test_db_sess_{SESSION_ID}"

os.environ["INTERNAL_SERVICE_TOKEN"] = "test_internal_service_token_12345"
os.environ["REDIS_URL"] = os.environ.get("TEST_REDIS_URL", "redis://127.0.0.1:16379")
os.environ["SURREAL_ENDPOINT"] = os.environ.get("TEST_SURREAL_URL", "http://127.0.0.1:8000")
os.environ["SURREAL_NS"] = TEST_NS
os.environ["SURREAL_DB"] = TEST_DB


@pytest.fixture(autouse=True)
def offline_search_engine(monkeypatch):
    from src.agents import runtime as runtime_module
    from tests.search_double import InMemorySearchEngine, install_search_double

    monkeypatch.setattr(runtime_module, "AgentSearchEngine", InMemorySearchEngine)
    engine = install_search_double()
    yield engine
    runtime_module.agent_runtime.set_search_engine(None)


@pytest.fixture
def resume_model():
    from src.agents.runtime import agent_runtime
    from tests.resume_double import ResumeDoubleModel

    agent_runtime.set_model_factory(lambda: ResumeDoubleModel())
    yield
    agent_runtime.set_model_factory(None)



@pytest_asyncio.fixture(scope="session", autouse=True)
async def setup_isolated_surrealdb_session():

    base_url = os.environ["SURREAL_ENDPOINT"].rstrip("/")
    repo_root = Path(__file__).resolve().parents[3]
    schema_path = repo_root / "server" / "api" / "src" / "schema.surql"
    schema_sql = schema_path.read_text(encoding="utf-8")
    schema_sql = schema_sql.replace(
        "USE NS mensetsu DB production;", f"USE NS {TEST_NS} DB {TEST_DB};"
    )

    init_sql = f"DEFINE NAMESPACE IF NOT EXISTS {TEST_NS}; DEFINE DATABASE IF NOT EXISTS {TEST_DB}; USE NS {TEST_NS} DB {TEST_DB};\n{schema_sql}"
    async with httpx.AsyncClient(timeout=30.0) as client:
        resp = await client.post(
            f"{base_url}/sql",
            auth=("root", "root"),
            headers={"surreal-ns": TEST_NS, "surreal-db": TEST_DB, "Accept": "application/json"},
            content=init_sql.encode("utf-8"),
        )
        resp.raise_for_status()

    yield

    drop_sql = f"REMOVE DATABASE {TEST_DB};"
    async with httpx.AsyncClient(timeout=10.0) as client:
        try:
            await client.post(
                f"{base_url}/sql",
                auth=("root", "root"),
                headers={"surreal-ns": TEST_NS, "Accept": "application/json"},
                content=drop_sql.encode("utf-8"),
            )
        except Exception:
            pass


@pytest_asyncio.fixture(scope="session", autouse=True)
async def redis_session_hygiene_check():
    redis_url = os.environ.get("REDIS_URL", "redis://127.0.0.1:16379")
    r = aioredis.from_url(redis_url)
    try:
        await r.config_set("maxmemory", "1gb")
        await r.config_set("maxmemory-policy", "noeviction")
        await r.flushall()
        initial_keys = await r.dbsize()
    except Exception:
        initial_keys = 0
    await r.aclose()

    yield

    r = aioredis.from_url(redis_url)
    try:
        alive_keys = await r.keys("mensetsu:worker:*:alive")
        if alive_keys:
            await r.unlink(*alive_keys)
        final_keys = await r.dbsize()
    except Exception:
        final_keys = 0
    await r.aclose()

    assert final_keys == initial_keys, (
        f"Redis test hygiene failure: initial keys={initial_keys}, final keys={final_keys}. "
        f"Tests leaked {final_keys - initial_keys} keys!"
    )


@pytest_asyncio.fixture(autouse=True)
async def isolate_test_redis_resources(request):
    test_id = uuid.uuid4().hex[:8]
    test_stream = f"test_jobs_stream_{test_id}"
    test_group = f"test_group_{test_id}"

    from src.orchestration import redis_queue

    orig_stream = redis_queue.DEFAULT_STREAM_KEY
    orig_group = redis_queue.DEFAULT_GROUP_NAME
    redis_queue.DEFAULT_STREAM_KEY = test_stream
    redis_queue.DEFAULT_GROUP_NAME = test_group

    redis_url = os.environ.get("REDIS_URL", "redis://127.0.0.1:16379")
    r_client = aioredis.from_url(redis_url)
    keys_before = set(await r_client.keys("*"))
    await r_client.aclose()

    yield

    redis_queue.DEFAULT_STREAM_KEY = orig_stream
    redis_queue.DEFAULT_GROUP_NAME = orig_group

    try:
        r = aioredis.from_url(redis_url)
        keys_after = set(await r.keys("*"))
        new_keys = list(keys_after - keys_before)
        if new_keys:
            chunk_size = 500
            for i in range(0, len(new_keys), chunk_size):
                await r.unlink(*new_keys[i : i + chunk_size])
        await r.aclose()
    except Exception:
        pass
