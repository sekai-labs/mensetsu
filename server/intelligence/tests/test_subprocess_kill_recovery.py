import asyncio
import os
import sys
import uuid
from pathlib import Path

import pytest
import redis.asyncio as aioredis
from langgraph.checkpoint.redis.aio import AsyncRedisSaver

from src.contracts.v1 import WorkflowRun
from src.orchestration.ledger import SurrealLedgerRepository
from src.orchestration.redis_queue import RedisStreamJobQueue


@pytest.mark.asyncio
async def test_worker_crash_recovery_deterministic_sentinel():
    unique_suffix = uuid.uuid4().hex[:8]
    test_ns = f"test_ns_{unique_suffix}"
    test_db = f"test_db_{unique_suffix}"
    redis_url = os.environ.get("REDIS_URL", "redis://127.0.0.1:16379")
    group_name = f"grp_kill_{unique_suffix}"
    stream_key = f"stream_kill_{unique_suffix}"
    sentinel_key = f"sentinel_block_{unique_suffix}"
    from src.db.client import Settings, SurrealDirectClient

    custom_settings = Settings(surreal_ns=test_ns, surreal_db=test_db)
    ledger = SurrealLedgerRepository()
    ledger.db = SurrealDirectClient(settings=custom_settings)

    init_sql = f"DEFINE NAMESPACE IF NOT EXISTS {test_ns}; DEFINE DATABASE IF NOT EXISTS {test_db}; USE NS {test_ns} DB {test_db};"
    repo_root = Path(__file__).resolve().parents[3]
    schema_path = repo_root / "server" / "api" / "src" / "schema.surql"
    schema_sql = schema_path.read_text(encoding="utf-8")

    schema_sql = schema_sql.replace(
        "USE NS mensetsu DB production;", f"USE NS {test_ns} DB {test_db};"
    )
    await ledger.db.execute_query(init_sql + "\n" + schema_sql)
    os.environ["SURREAL_NS"] = test_ns
    os.environ["SURREAL_DB"] = test_db
    queue = RedisStreamJobQueue(redis_url=redis_url, stream_key=stream_key, group_name=group_name)
    run_id = f"run_kill_{unique_suffix}"
    run = WorkflowRun(
        id=run_id,
        workflow_id="interview-preparation-static",
        workflow_name="Deterministic Kill Recovery Test",
        workspace_id=f"ws_kill_{unique_suffix}",
        user_id=f"usr_kill_{unique_suffix}",
        type="static",
        status="running",
    )
    await ledger.create_workflow_run(run)

    await queue.enqueue_job(
        run_id=run_id,
        payload={
            "run_id": run_id,
            "workflow_id": "interview-preparation-static",
            "workspace_id": f"ws_kill_{unique_suffix}",
            "user_id": f"usr_kill_{unique_suffix}",
            "company_name": "Deterministic Kill Corp",
            "role_title": "Platform Architect",
            "resume_text": "Platform architect with distributed systems experience.",
            "job_description": "Architect resilient platform services.",
        },
    )

    runner_code = f"""
import asyncio, os
import redis.asyncio as aioredis
from tests.test_agent_runtime_tools import ScriptedToolCallingFakeModel
from src.agents.runtime import agent_runtime
from src.orchestration.runtime.worker import DurableJobWorker
from src.orchestration.ledger import SurrealLedgerRepository
from src.db.client import Settings, SurrealDirectClient

class BlockingModel(ScriptedToolCallingFakeModel):
    async def _agenerate(self, messages, stop=None, run_manager=None, **kwargs):
        from src.agents.runtime import current_agent_id_var
        agent_id = current_agent_id_var.get()
        r = aioredis.from_url("{redis_url}")

        if agent_id == "company-research":
            res = await super()._agenerate(messages, stop=stop, run_manager=run_manager, **kwargs)
            await r.set("{run_id}_company_done", "1")
            return res

        if agent_id == "technical-research":
            custom_s = Settings(surreal_ns="{test_ns}", surreal_db="{test_db}")
            local_ledger = SurrealLedgerRepository()
            local_ledger.db = SurrealDirectClient(settings=custom_s)
            while True:
                t_list = await local_ledger.list_tasks_for_run("{run_id}")
                comp = [t for t in t_list if t.status == "completed"]
                if len(comp) >= 2:
                    break
                await asyncio.sleep(0.2)
            await r.set("{sentinel_key}", "blocked")
            while True:
                await asyncio.sleep(0.5)

        return await super()._agenerate(messages, stop=stop, run_manager=run_manager, **kwargs)
async def main():
    print("SUBPROCESS STARTED", flush=True)
    try:
        from unittest.mock import AsyncMock, patch
        from src.orchestration.redis_queue import RedisStreamJobQueue
        agent_runtime.set_model_factory(lambda: BlockingModel())
        from tests.search_double import install_search_double
        install_search_double()
        with patch("src.agents.runtime.SafeResearchFetcher.fetch_clean_text", new_callable=AsyncMock) as mock_fetch:
            mock_fetch.return_value = (
                "Careers at Acme",
                "We build distributed systems in Python and Rust.",
            )
            w = DurableJobWorker(worker_id="{run_id}_worker_A", redis_url="{redis_url}")
            w.queue = RedisStreamJobQueue(redis_url="{redis_url}", stream_key="{stream_key}", group_name="{group_name}")
            custom_s = Settings(surreal_ns="{test_ns}", surreal_db="{test_db}")
            w.ledger = SurrealLedgerRepository()
            w.ledger.db = SurrealDirectClient(settings=custom_s)
            from src.agents.runtime import current_ledger_var
            current_ledger_var.set(w.ledger)
            await w.initialize()
            print("INITIALIZED WORKER A", flush=True)
            await w.start()
    except Exception as e:
        import traceback
        traceback.print_exc()

asyncio.run(main())
"""
    env = dict(os.environ)
    env["PYTHONPATH"] = "."
    env["PYTHONUNBUFFERED"] = "1"
    env["INTERNAL_SERVICE_TOKEN"] = "test_internal_service_token_12345"
    env["SURREAL_ENDPOINT"] = "http://127.0.0.1:8000"
    env["SURREAL_URL"] = "http://127.0.0.1:8000"
    env["SURREAL_NS"] = test_ns
    env["SURREAL_DB"] = test_db
    env["REDIS_URL"] = redis_url
    env["WORKER_LEASE_TTL_S"] = "3.0"
    env["WORKER_HEARTBEAT_INTERVAL_S"] = "1.0"
    env["WORKER_MIN_IDLE_TIME_MS"] = "3500"

    proc_a = await asyncio.create_subprocess_exec(
        sys.executable,
        "-c",
        runner_code,
        cwd=str(Path(__file__).resolve().parents[1]),
        env=env,
        stdout=None,
        stderr=None,
    )

    r_client = aioredis.from_url(redis_url, decode_responses=True)
    blocked = False
    for i in range(70):
        await asyncio.sleep(0.5)
        val = await r_client.get(sentinel_key)
        if val == "blocked":
            blocked = True
            break
        ret = proc_a.returncode
        if ret is not None:
            print(f"PROC_A DIED EARLY (code {ret})!")
            break
        cd = await r_client.get(f"{run_id}_company_done")
        t_pre = await ledger.list_tasks_for_run(run_id)
        comp = [t.id for t in t_pre if t.status == "completed"]
        print(f"WAIT {i}: sentinel={val}, company_done={cd}, completed={comp}")
    assert blocked, "Worker A failed to reach deterministic sentinel block."
    tasks_pre = await ledger.list_tasks_for_run(run_id)
    pre_kill_completed = [t for t in tasks_pre if t.status == "completed"]
    assert len(pre_kill_completed) >= 2, (
        f"Expected >=2 completed tasks before kill, got {len(pre_kill_completed)}"
    )
    pre_kill_task_ids = {t.id for t in pre_kill_completed}
    pre_kill_execs = await ledger.list_agent_executions_for_run(run_id)
    assert len(pre_kill_execs) >= 2, (
        f"Expected >=2 agent executions before kill, got {len(pre_kill_execs)}"
    )
    pre_kill_tool_counts = {e.id: e.tool_call_count for e in pre_kill_execs}

    pre_kill_events = await ledger.get_events_history(run_id=run_id, limit=100)
    pre_kill_last_seq = pre_kill_events.last_seq
    saver = AsyncRedisSaver(redis_url=redis_url)
    await saver.asetup()
    ckpt_tuple = await saver.aget_tuple({"configurable": {"thread_id": run_id}})
    assert ckpt_tuple is not None, (
        "Redis checkpoint tuple must exist for thread_id=run_id before kill."
    )

    try:
        proc_a.kill()
        await proc_a.wait()
    except Exception:
        pass
    await r_client.delete(f"mensetsu:lease:{run_id}")
    runner_b_code = f"""
import asyncio, os
from tests.test_agent_runtime_tools import ScriptedToolCallingFakeModel
from src.agents.runtime import agent_runtime
from src.orchestration.runtime.worker import DurableJobWorker
from src.db.client import Settings, SurrealDirectClient
from src.orchestration.ledger import SurrealLedgerRepository

async def main():
    agent_runtime.set_model_factory(lambda: ScriptedToolCallingFakeModel())
    from tests.search_double import install_search_double
    install_search_double()
    from unittest.mock import AsyncMock, patch
    with patch("src.agents.runtime.SafeResearchFetcher.fetch_clean_text", new_callable=AsyncMock) as mock_fetch:
        mock_fetch.return_value = (
            "Careers at Acme",
            "We build distributed systems in Python and Rust.",
        )
        from src.orchestration.redis_queue import RedisStreamJobQueue
        q = RedisStreamJobQueue(redis_url="{redis_url}", stream_key="{stream_key}", group_name="{group_name}")
        custom_s = Settings(surreal_ns="{test_ns}", surreal_db="{test_db}")
        w = DurableJobWorker(
            worker_id="{run_id}_worker_B",
            redis_url="{redis_url}",
            lease_ttl_s=3.0,
            heartbeat_interval_s=1.0,
            min_idle_time_ms=100,
        )
        w.queue = q
        w.ledger = SurrealLedgerRepository()
        w.ledger.db = SurrealDirectClient(settings=custom_s)
        from src.agents.runtime import current_ledger_var
        current_ledger_var.set(w.ledger)
        await w.initialize()
        loop_task = asyncio.create_task(w.start())
        for i in range(360):
            await asyncio.sleep(0.5)
            r = await w.ledger.get_workflow_run("{run_id}")
            if r and r.status in ("completed", "failed"):
                break
        w.running = False
        await w.stop()
        loop_task.cancel()
asyncio.run(main())
"""
    proc_b = await asyncio.create_subprocess_exec(
        sys.executable,
        "-c",
        runner_b_code,
        cwd=str(Path(__file__).resolve().parents[1]),
        env=env,
        stdout=None,
        stderr=None,
    )
    await proc_b.wait()
    tasks_post_kill = await ledger.list_tasks_for_run(run_id)
    for t in tasks_post_kill:
        if t.id in pre_kill_task_ids:
            assert t.attempt == 1, (
                f"Pre-kill completed task {t.id} had attempt incremented to {t.attempt}"
            )
    post_kill_execs = await ledger.list_agent_executions_for_run(run_id)
    assert len(post_kill_execs) >= len(pre_kill_execs)
    post_exec_map = {e.id: e for e in post_kill_execs}
    for exec_id, expected_tool_count in pre_kill_tool_counts.items():
        assert exec_id in post_exec_map
        assert post_exec_map[exec_id].tool_call_count == expected_tool_count, (
            f"Tool call count changed for pre-kill execution {exec_id}: "
            f"was {expected_tool_count}, now {post_exec_map[exec_id].tool_call_count}"
        )

    company_execs = [e for e in post_kill_execs if e.agent_id == "company-research"]
    assert len(company_execs) == 1, (
        f"Expected exactly 1 company-research execution, found {len(company_execs)}"
    )

    post_artifacts = await ledger.list_artifacts_for_run(run_id)
    report_artifacts = [a for a in post_artifacts if a.type == "research_report"]
    assert len(report_artifacts) >= 1, "Expected research_report artifact to exist in ledger!"
    assert company_execs[0].id in [a.agent_execution_id for a in report_artifacts]

    final_run = await ledger.get_workflow_run(run_id)
    assert final_run.status == "completed", f"Run failed with message: {final_run.error_message}"
    all_events = []
    curr_seq = 0
    while True:
        history = await ledger.get_events_history(run_id=run_id, after_seq=curr_seq, limit=100)
        all_events.extend(history.events)
        if not history.events or history.next_after_seq is None:
            break
        curr_seq = history.next_after_seq

    event_types = [e.type for e in all_events]
    assert "execution.resumed" in event_types
    assert "worker.recovered" in event_types

    seqs = [e.seq for e in all_events]
    assert seqs == list(range(1, len(all_events) + 1)), "Event sequence must be strictly contiguous"
    post_kill_events = [e for e in all_events if e.seq > pre_kill_last_seq]
    for e in post_kill_events:
        if e.type == "task.started":
            assert e.task_id not in pre_kill_task_ids, f"Task {e.task_id} was restarted after kill!"

    assert any(e.type == "task.started" for e in post_kill_events), (
        "Worker B executed 0 tasks after recovery!"
    )
    await queue.close()
    for pat in (f"*{unique_suffix}*", f"*{stream_key}*", f"*{sentinel_key}*", f"*{run_id}*"):
        k_list = await r_client.keys(pat)
        if k_list:
            await r_client.unlink(*k_list)
    await r_client.aclose()
