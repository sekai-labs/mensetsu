import json
import uuid
from pathlib import Path
from unittest.mock import AsyncMock, patch

import pytest

from src.agents.runtime import agent_runtime
from src.contracts.v1 import (
    ExecutionEvent,
)
from src.orchestration.runtime.worker import DurableJobWorker
from src.orchestration.service import WorkflowService


@pytest.mark.asyncio
async def test_e2e_static_workflow_and_fixtures(monkeypatch):
    token = "test_internal_secret_token"
    monkeypatch.setenv("INTERNAL_SERVICE_TOKEN", token)

    run = await WorkflowService.create_run(
        workspace_id="ws_e2e_test",
        user_id="usr_e2e_test",
        workflow_id="interview-preparation-static",
        company_name="Acme Systems",
        role_title="Lead Backend Architect",
        company_urls=["https://acme.example.com"],
        resume_text="Experienced Python and Distributed Systems Architect.",
        job_description="Seeking expert in scalable systems, message queues, and reliability.",
    )
    run_id = run.id

    from tests.test_agent_runtime_tools import ScriptedToolCallingFakeModel

    agent_runtime.set_model_factory(lambda: ScriptedToolCallingFakeModel())
    worker_name = f"worker_e2e_{uuid.uuid4().hex[:8]}"
    worker = DurableJobWorker(worker_id=worker_name)
    await worker.initialize()
    job = {
        "run_id": run_id,
        "msg_id": "1-0",
        "payload": {
            "run_id": run_id,
            "workflow_id": "interview-preparation-static",
            "workspace_id": "ws_e2e_test",
            "user_id": "usr_e2e_test",
            "company_name": "Acme Systems",
            "role_title": "Lead Backend Architect",
            "resume_text": "Experienced Python Architect.",
            "job_description": "Distributed systems.",
        },
    }
    with patch(
        "src.agents.runtime.SafeResearchFetcher.fetch_clean_text", new_callable=AsyncMock
    ) as mock_fetch:
        mock_fetch.return_value = (
            "Careers at Acme",
            "We build distributed systems in Python and Rust.",
        )
        await worker.execute_job(job)
    await worker.queue.close()

    completed_run = await WorkflowService.get_run(run_id)
    assert completed_run is not None
    assert completed_run.status == "completed"
    assert completed_run.progress.percentage == 100

    task_graph = await WorkflowService.get_task_graph(run_id)
    del_graph = await WorkflowService.get_delegation_graph(run_id)
    agents = await WorkflowService.list_agents(run_id)
    artifacts = await WorkflowService.list_artifacts(run_id)

    events: list[ExecutionEvent] = []
    curr_after_seq = 0
    while True:
        history_page = await WorkflowService.get_events_history(
            run_id=run_id, after_seq=curr_after_seq, limit=100
        )
        events.extend(history_page.events)
        if history_page.next_after_seq is None or not history_page.events:
            break
        curr_after_seq = history_page.next_after_seq

    seqs = [e.seq for e in events]
    assert seqs == list(range(1, len(events) + 1))
    assert events[-1].seq == completed_run.last_seq
    assert events[-1].type in ("workflow.completed", "workflow.failed", "workflow.cancelled")

    # Format SSE text directly from recorded execution events
    sse_text = "".join(
        f"id: {e.seq}\nevent: {e.type}\ndata: {e.model_dump_json()}\n\n" for e in events
    )
    assert "event: workflow.completed" in sse_text

    repo_root = Path(__file__).resolve().parents[3]
    fixture_dir = repo_root / "contracts" / "rest" / "fixtures" / "workflows.v1"
    fixture_dir.mkdir(parents=True, exist_ok=True)

    (fixture_dir / "run.json").write_text(completed_run.model_dump_json(indent=2), encoding="utf-8")
    (fixture_dir / "graph.json").write_text(task_graph.model_dump_json(indent=2), encoding="utf-8")
    (fixture_dir / "delegations-graph.json").write_text(
        del_graph.model_dump_json(indent=2), encoding="utf-8"
    )
    (fixture_dir / "agents.json").write_text(
        json.dumps([a.model_dump() for a in agents], indent=2, default=str), encoding="utf-8"
    )
    (fixture_dir / "artifacts.json").write_text(
        json.dumps([a.model_dump() for a in artifacts], indent=2, default=str), encoding="utf-8"
    )
    (fixture_dir / "events.ndjson").write_text(
        "\n".join(e.model_dump_json() for e in events) + "\n", encoding="utf-8"
    )
    (fixture_dir / "sse.txt").write_text(sse_text, encoding="utf-8")
