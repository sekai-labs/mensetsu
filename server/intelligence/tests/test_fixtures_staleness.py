import json
from pathlib import Path

from pydantic import TypeAdapter

from src.contracts.v1 import (
    AgentExecution,
    Artifact,
    DelegationGraphResponse,
    ExecutionEvent,
    TaskGraphResponse,
    WorkflowRun,
)


def test_contracts_fixtures_valid():
    repo_root = Path(__file__).resolve().parents[3]
    fixture_dir = repo_root / "server" / "api" / "contracts" / "fixtures" / "workflows.v1"

    if not fixture_dir.exists():
        return

    run_path = fixture_dir / "run.json"
    if run_path.exists():
        run_data = json.loads(run_path.read_text(encoding="utf-8"))
        WorkflowRun.model_validate(run_data)

    graph_path = fixture_dir / "graph.json"
    if graph_path.exists():
        graph_data = json.loads(graph_path.read_text(encoding="utf-8"))
        TaskGraphResponse.model_validate(graph_data)

    del_path = fixture_dir / "delegations-graph.json"
    if del_path.exists():
        del_data = json.loads(del_path.read_text(encoding="utf-8"))
        DelegationGraphResponse.model_validate(del_data)

    agents_path = fixture_dir / "agents.json"
    if agents_path.exists():
        agents_data = json.loads(agents_path.read_text(encoding="utf-8"))
        for a in agents_data:
            AgentExecution.model_validate(a)

    arts_path = fixture_dir / "artifacts.json"
    if arts_path.exists():
        arts_data = json.loads(arts_path.read_text(encoding="utf-8"))
        for art in arts_data:
            Artifact.model_validate(art)

    events_path = fixture_dir / "events.ndjson"
    if events_path.exists():
        adapter = TypeAdapter(ExecutionEvent)
        for line in events_path.read_text(encoding="utf-8").strip().splitlines():
            if line:
                adapter.validate_python(json.loads(line))
