import json
import uuid
from pathlib import Path

import pytest
from httpx import ASGITransport, AsyncClient
from pydantic import TypeAdapter

from src.agents.runtime import agent_runtime
from src.contracts.v1 import (
    AgentExecution,
    Artifact,
    DelegationGraphResponse,
    ExecutionEvent,
    TaskGraphResponse,
    WorkflowRun,
)
from src.orchestration.runtime.worker import DurableJobWorker
from src.service import app


@pytest.mark.asyncio
async def test_e2e_static_workflow_and_fixtures(monkeypatch):

    token = "test_internal_secret_token"
    monkeypatch.setenv("INTERNAL_SERVICE_TOKEN", token)

    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        req_body = {
            "workspace_id": "ws_e2e_test",
            "user_id": "usr_e2e_test",
            "workflow_id": "interview-preparation-static",
            "company_name": "Acme Systems",
            "role_title": "Lead Backend Architect",
            "company_urls": ["https://acme.example.com"],
            "resume_text": "Experienced Python and Distributed Systems Architect.",
            "job_description": "Seeking expert in scalable systems, message queues, and reliability.",
        }
        res = await client.post(
            "/internal/v1/workflows/runs",
            json=req_body,
            headers={"Authorization": f"Bearer {token}"},
        )
        assert res.status_code == 201
        run_data = res.json()
        run = WorkflowRun.model_validate(run_data)
        run_id = run.id
        from unittest.mock import AsyncMock, patch

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

        run_res = await client.get(
            f"/internal/v1/workflows/runs/{run_id}", headers={"Authorization": f"Bearer {token}"}
        )
        assert run_res.status_code == 200
        completed_run = WorkflowRun.model_validate(run_res.json())
        assert completed_run.status == "completed"
        assert completed_run.progress.percentage == 100

        graph_res = await client.get(
            f"/internal/v1/workflows/runs/{run_id}/graph",
            headers={"Authorization": f"Bearer {token}"},
        )
        task_graph = TaskGraphResponse.model_validate(graph_res.json())

        del_graph_res = await client.get(
            f"/internal/v1/workflows/runs/{run_id}/delegations/graph",
            headers={"Authorization": f"Bearer {token}"},
        )
        del_graph = DelegationGraphResponse.model_validate(del_graph_res.json())

        agents_res = await client.get(
            f"/internal/v1/workflows/runs/{run_id}/agents",
            headers={"Authorization": f"Bearer {token}"},
        )
        agents = [AgentExecution.model_validate(a) for a in agents_res.json()]

        art_res = await client.get(
            f"/internal/v1/workflows/runs/{run_id}/artifacts",
            headers={"Authorization": f"Bearer {token}"},
        )
        artifacts = [Artifact.model_validate(a) for a in art_res.json()]

        events = []
        adapter = TypeAdapter(ExecutionEvent)
        curr_after_seq = 0
        while True:
            evts_res = await client.get(
                f"/internal/v1/workflows/runs/{run_id}/events/history?after_seq={curr_after_seq}&limit=100",
                headers={"Authorization": f"Bearer {token}"},
            )
            evts_data = evts_res.json()
            page_events = [adapter.validate_python(e) for e in evts_data["events"]]
            events.extend(page_events)
            next_after = evts_data.get("next_after_seq")
            if next_after is None:
                break
            curr_after_seq = next_after

        seqs = [e.seq for e in events]
        assert seqs == list(range(1, len(events) + 1))
        assert events[-1].seq == completed_run.last_seq
        assert events[-1].type in ("workflow.completed", "workflow.failed", "workflow.cancelled")

        sse_res = await client.get(
            f"/internal/v1/workflows/runs/{run_id}/events",
            headers={"Authorization": f"Bearer {token}"},
        )
        sse_text = sse_res.text
        assert "event: workflow.completed" in sse_text

        repo_root = Path(__file__).resolve().parents[3]
        fixture_dir = repo_root / "server" / "api" / "contracts" / "fixtures" / "workflows.v1"
        fixture_dir.mkdir(parents=True, exist_ok=True)

        (fixture_dir / "run.json").write_text(
            completed_run.model_dump_json(indent=2), encoding="utf-8"
        )
        (fixture_dir / "graph.json").write_text(
            task_graph.model_dump_json(indent=2), encoding="utf-8"
        )
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
