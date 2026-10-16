import os
import uuid

import pytest
from langchain_core.messages import AIMessage
from langchain_core.outputs import ChatGeneration, ChatResult
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

    from src.contracts.v1 import Artifact

    for k in range(5):
        await ledger.save_artifact(
            Artifact(
                id=f"ev_{k}",
                run_id=run_id,
                type="evidence",
                title=f"Evidence {k}",
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
        "src.agents.runtime.SafeResearchFetcher.fetch_clean_text", new_callable=AsyncMock
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
