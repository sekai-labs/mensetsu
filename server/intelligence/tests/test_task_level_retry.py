import pytest
from langchain_core.language_models.chat_models import BaseChatModel
from langchain_core.messages import AIMessage
from langchain_core.outputs import ChatGeneration, ChatResult

from src.agents.runtime import MAX_MISSING_OUTPUT_TOOL_NUDGES, agent_runtime
from src.contracts.v1 import WorkflowRun
from src.orchestration.ledger import SurrealLedgerRepository
from src.orchestration.static_workflow import StaticWorkflowState, build_static_workflow


class FailOnceModel(BaseChatModel):
    attempt_count: int = 0
    plain_text_replies: int = 1

    @property
    def _llm_type(self) -> str:
        return "fail_once_model"

    def bind_tools(self, tools, **kwargs):
        return self

    def _generate(self, messages, stop=None, run_manager=None, **kwargs):
        raise NotImplementedError

    async def _agenerate(self, messages, stop=None, run_manager=None, **kwargs):
        FailOnceModel.attempt_count += 1
        if FailOnceModel.attempt_count <= FailOnceModel.plain_text_replies:
            return ChatResult(
                generations=[
                    ChatGeneration(message=AIMessage(content="Just talking, no tool call."))
                ]
            )

        output_payload = {
            "overview": "Acme Overview",
            "business_model": "SaaS",
            "engineering_culture": ["Pragmatic"],
            "tech_stack": [],
            "architecture_notes": ["Microservices"],
            "interview_process_notes": ["Round 1"],
            "claims": [
                {
                    "statement": f"Claim {i}",
                    "category": "architecture",
                    "confidence": 0.95,
                    "evidence_ids": [f"ev_{k}" for k in range(4)],
                }
                for i in range(8)
            ],
            "summary": "Recovered company summary",
        }
        msg = AIMessage(
            content="",
            tool_calls=[
                {
                    "name": "CompanyResearchOutput",
                    "args": output_payload,
                    "id": "tc_recovered",
                    "type": "tool_call",
                }
            ],
        )
        return ChatResult(generations=[ChatGeneration(message=msg)])


async def run_company_research_node(
    run_id: str, workspace_id: str, plain_text_replies: int, seed_prior_attempt: int | None = None
):
    ledger = SurrealLedgerRepository()
    await ledger.create_workflow_run(
        WorkflowRun(
            id=run_id,
            workflow_id="interview-preparation-static",
            workflow_name="Task Retry Test",
            workspace_id=workspace_id,
            user_id="usr_tr",
            type="static",
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
                content={"snippet": "Evidence text"},
            )
        )
    FailOnceModel.attempt_count = 0
    FailOnceModel.plain_text_replies = plain_text_replies
    agent_runtime.set_model_factory(lambda: FailOnceModel())

    import os

    from langgraph.checkpoint.redis.aio import AsyncRedisSaver

    saver = AsyncRedisSaver(redis_url=os.environ.get("REDIS_URL", "redis://127.0.0.1:16379"))
    await saver.asetup()

    graph = build_static_workflow(checkpointer=saver, ledger=ledger, db_client=ledger.db)

    state = StaticWorkflowState(
        run_id=run_id,
        workspace_id=workspace_id,
        company_name="Acme Retry Corp",
        role_title="Senior Engineer",
        resume_text="Senior engineer with Rust and distributed systems experience.",
        job_description="Build reliable distributed services.",
    )

    if seed_prior_attempt is not None:
        from src.contracts.v1 import WorkflowTask

        await ledger.save_task(
            WorkflowTask(
                id=f"task_company_{run_id[:8]}",
                run_id=run_id,
                task_type="research",
                name="Research Target Company",
                assigned_agent="company-research",
                status="running",
                attempt=seed_prior_attempt,
            )
        )

    await graph.nodes["company_research"].ainvoke(state)

    tasks = await ledger.list_tasks_for_run(run_id)
    comp_task = next(t for t in tasks if t.assigned_agent == "company-research")
    events = await ledger.get_events_history(run_id=run_id, limit=100)
    return comp_task, events.events


@pytest.mark.asyncio
async def test_run_level_retry_continues_task_attempt_numbering():
    comp_task, events = await run_company_research_node(
        "run_test_attempt_continue", "ws_tr_continue", plain_text_replies=0, seed_prior_attempt=2
    )
    assert comp_task.status == "completed"
    assert comp_task.attempt == 3
    started = [e for e in events if e.type == "task.started"]
    assert [e.payload.attempt for e in started] == [3]


@pytest.mark.asyncio
async def test_task_level_retry_before_run_failure():
    comp_task, events = await run_company_research_node(
        "run_test_task_retry", "ws_tr", plain_text_replies=1 + MAX_MISSING_OUTPUT_TOOL_NUDGES
    )
    assert comp_task.status == "completed"
    assert comp_task.attempt == 2

    event_types = [e.type for e in events]
    assert "task.retrying" in event_types
    retry_ev = next(e for e in events if e.type == "task.retrying")
    assert retry_ev.payload.next_attempt == 2
    assert retry_ev.payload.delay_ms == 1000
    reprompts = [
        e for e in events if e.type == "agent.activity" and e.payload.activity_type == "output_reprompt"
    ]
    assert len(reprompts) == MAX_MISSING_OUTPUT_TOOL_NUDGES


@pytest.mark.asyncio
async def test_single_plain_text_reply_is_reprompted_without_task_retry():
    comp_task, events = await run_company_research_node(
        "run_test_task_reprompt", "ws_tr_reprompt", plain_text_replies=1
    )
    assert comp_task.status == "completed"
    assert comp_task.attempt == 1
    assert FailOnceModel.attempt_count == 2

    event_types = [e.type for e in events]
    assert "task.retrying" not in event_types
    reprompts = [
        e for e in events if e.type == "agent.activity" and e.payload.activity_type == "output_reprompt"
    ]
    assert len(reprompts) == 1
