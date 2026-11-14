import os
import uuid
from types import SimpleNamespace

import pytest
from langchain_core.messages import AIMessage
from langchain_core.outputs import ChatGeneration, ChatResult
from langgraph.checkpoint.memory import InMemorySaver
from langgraph.checkpoint.redis.aio import AsyncRedisSaver

from src.agents.runtime import agent_runtime
from src.contracts.v1 import WorkflowRun
from src.orchestration.dynamic_workflow import (
    DynamicWorkflowState,
    build_dynamic_workflow,
    validate_plan_dag,
)
from src.orchestration.ledger import SurrealLedgerRepository
from tests.test_agent_runtime_tools import ScriptedToolCallingFakeModel


class DynamicPlanFakeModel(ScriptedToolCallingFakeModel):
    async def _agenerate(self, messages, stop=None, run_manager=None, **kwargs):
        from src.agents.runtime import current_agent_id_var

        if current_agent_id_var.get() != "planner":
            return await super()._agenerate(messages, stop=stop, run_manager=run_manager, **kwargs)
        payload = {
            "tasks": [
                {
                    "id": "dyn_task_1",
                    "name": "Research Company",
                    "assigned_agent": "company-research",
                    "dependencies": [],
                    "required": False,
                },
                {
                    "id": "dyn_task_2",
                    "name": "Research Technology",
                    "assigned_agent": "technical-research",
                    "dependencies": ["dyn_task_1"],
                    "required": True,
                },
            ],
            "summary": "Plan with 2 tasks",
        }
        msg = AIMessage(
            content="",
            tool_calls=[
                {
                    "name": "PlanOutput",
                    "args": payload,
                    "id": f"tc_{uuid.uuid4().hex[:6]}",
                    "type": "tool_call",
                }
            ],
        )
        return ChatResult(generations=[ChatGeneration(message=msg)])


def test_validate_plan_dag_rules():
    with pytest.raises(ValueError, match="maximum task count"):
        validate_plan_dag(
            [
                {"id": f"t_{i}", "assigned_agent": "job-analysis", "dependencies": []}
                for i in range(16)
            ]
        )

    with pytest.raises(ValueError, match="maximum permitted per revision is 4"):
        validate_plan_dag(
            [
                {
                    "id": f"t_{i}",
                    "assigned_agent": "company-research",
                    "dependencies": [],
                    "name": f"Task {i}",
                    "required": True,
                }
                for i in range(5)
            ]
        )
    with pytest.raises(ValueError, match="Duplicate task ID"):
        validate_plan_dag(
            [
                {"id": "t_1", "assigned_agent": "company-research", "dependencies": []},
                {"id": "t_1", "assigned_agent": "technical-research", "dependencies": []},
            ]
        )

    with pytest.raises(ValueError, match="Unknown agent"):
        validate_plan_dag(
            [{"id": "t_1", "assigned_agent": "unknown-nonexistent-agent", "dependencies": []}]
        )

    with pytest.raises(ValueError, match="nonexistent dependency"):
        validate_plan_dag(
            [
                {
                    "id": "t_1",
                    "assigned_agent": "company-research",
                    "dependencies": ["missing_dep"],
                },
            ]
        )

    with pytest.raises(ValueError, match="Cycle detected"):
        validate_plan_dag(
            [
                {"id": "t_1", "assigned_agent": "company-research", "dependencies": ["t_2"]},
                {"id": "t_2", "assigned_agent": "technical-research", "dependencies": ["t_1"]},
            ]
        )

    with pytest.raises(ValueError, match="maximum depth"):
        validate_plan_dag(
            [
                {"id": "t_1", "assigned_agent": "company-research", "dependencies": []},
                {"id": "t_2", "assigned_agent": "technical-research", "dependencies": ["t_1"]},
                {"id": "t_3", "assigned_agent": "job-analysis", "dependencies": ["t_2"]},
                {"id": "t_4", "assigned_agent": "evidence-evaluator", "dependencies": ["t_3"]},
                {"id": "t_5", "assigned_agent": "curriculum", "dependencies": ["t_4"]},
            ]
        )


@pytest.mark.asyncio
async def test_dynamic_workflow_plan_revision_and_parallel_overlap():
    ledger = SurrealLedgerRepository()
    run_id = f"run_dyn_test_{uuid.uuid4().hex[:8]}"
    ws_id = f"ws_dyn_test_{uuid.uuid4().hex[:8]}"

    await ledger.create_workflow_run(
        WorkflowRun(
            id=run_id,
            workflow_id="interview-preparation-dynamic",
            workflow_name="Dynamic Workflow Test",
            workspace_id=ws_id,
            user_id="usr_dyn_test",
            type="dynamic",
            status="running",
        )
    )

    from src.contracts.v1 import Artifact, EvidenceProvenance

    for k in range(5):
        await ledger.save_artifact(
            Artifact(
                id=f"ev_{k}",
                run_id=run_id,
                type="evidence",
                title=f"Evidence {k}",
                evidence=EvidenceProvenance(
                    source_url=f"https://example.com/evidence/{k}",
                    title=f"Evidence {k}",
                    snippet="Evidence snippet",
                    confidence=0.9,
                ),
                content={"snippet": "Evidence snippet"},
            )
        )

    agent_runtime.set_model_factory(lambda: DynamicPlanFakeModel())

    redis_url = os.environ.get("REDIS_URL", "redis://127.0.0.1:16379")
    saver = AsyncRedisSaver(redis_url=redis_url)
    await saver.asetup()

    graph = build_dynamic_workflow(checkpointer=saver, ledger=ledger)
    initial_state = DynamicWorkflowState(
        run_id=run_id,
        workspace_id=ws_id,
        company_name="Acme Dynamic",
        role_title="Staff Engineer",
        resume_text="Engineer with Rust and distributed systems experience.",
        job_description="Build distributed storage and networking systems in Rust.",
    )

    from unittest.mock import AsyncMock, patch

    with patch(
        "src.agents.agent_tools.SafeResearchFetcher.fetch_clean_text", new_callable=AsyncMock
    ) as mock_fetch:
        mock_fetch.return_value = (
            "Careers at Acme",
            "We build distributed systems in Python and Rust.",
        )
        res = await graph.ainvoke(initial_state, config={"configurable": {"thread_id": run_id}})
    assert res is not None

    all_events = []
    after_seq = 0
    while True:
        page = await ledger.get_events_history(run_id=run_id, after_seq=after_seq, limit=100)
        all_events.extend(page.events)
        if page.next_after_seq is None:
            break
        after_seq = page.next_after_seq
    event_types = [e.type for e in all_events]

    assert "plan.created" in event_types
    assert "plan.revised" in event_types
    assert "task.added" in event_types
    assert "workflow.completed" in event_types
    added = next(e for e in all_events if e.type == "task.added")
    assert added.payload.added_in_revision == 2

    final_run = await ledger.get_workflow_run(run_id)
    assert final_run.status == "completed"
    assert final_run.progress.completed >= 3
    assert final_run.progress.percentage == 100

    questions = await ledger.db.execute_query(
        "SELECT * FROM interview_question WHERE workspace_id = type::record('workspace', $ws);",
        vars={"ws": ws_id},
    )
    assert len(questions[-1]["result"]) >= 15
    skills = await ledger.db.execute_query(
        "SELECT * FROM skill WHERE workspace_id = type::record('workspace', $ws);",
        vars={"ws": ws_id},
    )
    assert len(skills[-1]["result"]) >= 12


@pytest.mark.asyncio
async def test_dynamic_retries_replan_with_recovery_context(monkeypatch):
    class FakeLedger:
        def __init__(self):
            self.tasks: dict[str, object] = {}
            self.events: list[dict] = []
            self.db = None

        async def get_task(self, run_id, task_id):
            return self.tasks.get(task_id)

        async def next_task_attempt(self, run_id, task_id):
            existing = self.tasks.get(task_id)
            return existing.attempt + 1 if existing is not None else 1

        async def save_task(self, task):
            self.tasks[task.id] = task

        async def append_event_transactional(self, **kwargs):
            self.events.append(kwargs)

        async def append_execution_log(self, **kwargs):
            return None

        async def update_workflow_run(self, run_id, fields):
            return None

        async def refresh_run_progress(self, run_id):
            return None

    class FakeOutputGraph:
        async def ainvoke(self, state, config=None):
            return {
                "resume_text": "Engineer with Rust experience.",
                "job_description": "Build distributed systems in Rust.",
                "parsed_resume": {},
            }

    plan_tasks = {
        "tasks": [
            {
                "id": "dyn_task_1",
                "name": "Research Company",
                "assigned_agent": "company-research",
                "dependencies": [],
                "required": False,
            },
            {
                "id": "dyn_task_2",
                "name": "Research Technology",
                "assigned_agent": "technical-research",
                "dependencies": ["dyn_task_1"],
                "required": True,
            },
        ],
        "summary": "Plan with 2 tasks",
    }
    fail_once = {
        "planner": RuntimeError("injected planner failure: model unavailable"),
        "company-research": RuntimeError("injected research failure"),
    }
    calls: dict[str, list[dict]] = {}

    async def fake_execute_agent_loop(**kwargs):
        agent_id = kwargs["agent_id"]
        agent_calls = calls.setdefault(agent_id, [])
        agent_calls.append({**kwargs, "input_data": dict(kwargs.get("input_data") or {})})
        error = fail_once.get(agent_id)
        if len(agent_calls) == 1 and error is not None:
            raise error
        data = plan_tasks if agent_id == "planner" else {
            "summary": f"{agent_id} completed",
            "claims": [],
        }
        return SimpleNamespace(id=f"exec_{agent_id}_{len(agent_calls)}", public_summary={}), data

    monkeypatch.setattr(agent_runtime, "execute_agent_loop", fake_execute_agent_loop)
    monkeypatch.setattr(
        "src.orchestration.static_workflow.build_static_workflow",
        lambda **kwargs: FakeOutputGraph(),
    )

    run_id = f"run_dyn_retry_{uuid.uuid4().hex[:8]}"
    graph = build_dynamic_workflow(checkpointer=InMemorySaver(), ledger=FakeLedger())
    result = await graph.ainvoke(
        DynamicWorkflowState(
            run_id=run_id,
            workspace_id=f"ws_dyn_retry_{uuid.uuid4().hex[:8]}",
            company_name="Acme Retry",
            role_title="Staff Engineer",
            resume_text="Engineer with Rust experience.",
            job_description="Build distributed systems in Rust.",
        ),
        config={"configurable": {"thread_id": run_id}},
    )
    assert result is not None

    planner_calls = calls["planner"]
    assert len(planner_calls) == 2
    assert planner_calls[0]["recovery_context"] is None
    assert "previous_plan_rejected" not in planner_calls[0]["input_data"]
    planner_retry = planner_calls[1]
    assert planner_retry["task_id"] == planner_calls[0]["task_id"]
    assert planner_retry["objective"] == planner_calls[0]["objective"]
    assert "injected planner failure" in planner_retry["input_data"]["previous_plan_rejected"]
    planner_recovery = planner_retry["recovery_context"]
    assert planner_recovery is not None
    assert planner_recovery.attempt == 1
    assert "injected planner failure" in planner_recovery.previous_error
    assert "plan" in planner_recovery.alternative_strategy.lower()

    company_calls = calls["company-research"]
    assert len(company_calls) == 2
    assert company_calls[0]["recovery_context"] is None
    company_retry = company_calls[1]
    assert company_retry["agent_id"] == "company-research"
    assert company_retry["task_id"] == company_calls[0]["task_id"]
    assert company_retry["run_id"] == company_calls[0]["run_id"]
    assert company_retry["objective"] == company_calls[0]["objective"]
    assert company_retry["input_data"] == company_calls[0]["input_data"]
    company_recovery = company_retry["recovery_context"]
    assert company_recovery is not None
    assert company_recovery.attempt == 1
    assert "injected research failure" in company_recovery.previous_error
    assert "source" in company_recovery.alternative_strategy.lower()


@pytest.mark.asyncio
async def test_dynamic_workflow_enforces_output_contract_invariants(monkeypatch):
    from src.evals import OutputQualityError, validate_run_outputs

    class InvariantEnforcingOutputGraph:
        def __init__(self, mode: str):
            self.mode = mode

        async def ainvoke(self, state, config=None):
            if self.mode == "normalize_only":
                return {
                    "resume_text": state.resume_text,
                    "job_description": state.job_description,
                    "parsed_resume": state.parsed_resume,
                }
            validate_run_outputs(
                company=state.research_results.get("company"),
                evidence_artifacts=[],
                questions=state.questions,
                skills=[],
                relations=[],
                missing_research=state.missing_research,
            ).raise_if_invalid(context=f"dynamic run {state.run_id}")
            return {}

    monkeypatch.setattr(
        "src.orchestration.static_workflow.build_static_workflow",
        lambda **kwargs: InvariantEnforcingOutputGraph(kwargs["mode"]),
    )

    class FakeLedger:
        def __init__(self):
            self.tasks: dict[str, object] = {}
            self.events: list[dict] = []
            self.db = None

        async def get_task(self, run_id, task_id):
            return self.tasks.get(task_id)

        async def next_task_attempt(self, run_id, task_id):
            existing = self.tasks.get(task_id)
            return existing.attempt + 1 if existing is not None else 1

        async def save_task(self, task):
            self.tasks[task.id] = task

        async def append_event_transactional(self, **kwargs):
            self.events.append(kwargs)

        async def append_execution_log(self, **kwargs):
            return None

        async def update_workflow_run(self, run_id, fields):
            return None

        async def refresh_run_progress(self, run_id):
            return None

    plan_tasks = {
        "tasks": [
            {
                "id": "dyn_task_job",
                "name": "Analyze Job",
                "assigned_agent": "job-analysis",
                "dependencies": [],
                "required": True,
            },
            {
                "id": "dyn_task_comp",
                "name": "Research Company",
                "assigned_agent": "company-research",
                "dependencies": [],
                "required": True,
            },
        ],
        "summary": "Plan with job and company tasks",
    }

    async def fake_execute_agent_loop(**kwargs):
        agent_id = kwargs["agent_id"]
        if agent_id == "planner":
            return SimpleNamespace(id="exec_planner", public_summary={}), plan_tasks
        if agent_id == "job-analysis":
            return SimpleNamespace(id="exec_job", public_summary={}), {
                "role_summary": "Staff",
                "requirements": [],
            }
        if agent_id == "company-research":
            return SimpleNamespace(id="exec_comp", public_summary={}), {
                "claims": [
                    {
                        "statement": "Claim missing evidence",
                        "evidence_ids": ["ev_nonexistent_artifact"],
                    }
                ]
            }
        return SimpleNamespace(id=f"exec_{agent_id}", public_summary={}), {}

    monkeypatch.setattr(agent_runtime, "execute_agent_loop", fake_execute_agent_loop)

    run_id = f"run_dyn_contract_{uuid.uuid4().hex[:8]}"
    graph = build_dynamic_workflow(checkpointer=InMemorySaver(), ledger=FakeLedger())

    with pytest.raises(OutputQualityError) as exc_info:
        await graph.ainvoke(
            DynamicWorkflowState(
                run_id=run_id,
                workspace_id=f"ws_dyn_contract_{uuid.uuid4().hex[:8]}",
                company_name="Acme Invariant",
                role_title="Staff Engineer",
                resume_text="Engineer with Rust experience.",
                job_description="Build distributed systems in Rust.",
            ),
            config={"configurable": {"thread_id": run_id}},
        )

    assert "citation_provenance" in str(exc_info.value)
    assert "ev_nonexistent_artifact" in str(exc_info.value)
