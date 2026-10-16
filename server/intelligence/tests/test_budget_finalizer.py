import uuid

import pytest
from langchain_core.language_models.chat_models import BaseChatModel
from langchain_core.messages import AIMessage
from langchain_core.outputs import ChatGeneration, ChatResult

from src.agents.runtime import AutonomousAgentRuntime
from src.contracts.v1 import WorkflowRun
from src.orchestration.ledger import SurrealLedgerRepository


class InfiniteLoopSearchModel(BaseChatModel):
    @property
    def _llm_type(self) -> str:
        return "infinite_loop_model"

    def bind_tools(self, tools, **kwargs):
        return self

    def _generate(self, messages, stop=None, run_manager=None, **kwargs):
        raise NotImplementedError

    async def _agenerate(self, messages, stop=None, run_manager=None, **kwargs):

        messages_text = str([getattr(m, "content", "") for m in messages])
        if "Research budget is nearly exhausted" in messages_text:
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
                "summary": "Forced finalization summary",
            }
            msg = AIMessage(
                content="",
                tool_calls=[
                    {
                        "name": "CompanyResearchOutput",
                        "args": output_payload,
                        "id": "tc_forced_final",
                        "type": "tool_call",
                    }
                ],
            )
            return ChatResult(generations=[ChatGeneration(message=msg)])

        tool_call = {
            "name": "web_search",
            "args": {"query": "continue searching"},
            "id": f"tc_{len(messages)}",
            "type": "tool_call",
        }
        msg = AIMessage(content="", tool_calls=[tool_call])
        return ChatResult(generations=[ChatGeneration(message=msg)])


@pytest.mark.asyncio
async def test_budget_aware_finalization_forces_output_tool():

    ledger = SurrealLedgerRepository()
    run_id = "run_test_budget_final"
    await ledger.create_workflow_run(
        WorkflowRun(
            id=run_id,
            workflow_id="interview-preparation-static",
            workflow_name="Budget Finalizer Test",
            workspace_id="ws_bf",
            user_id="usr_bf",
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
    runtime = AutonomousAgentRuntime(ledger=ledger)
    infinite_model = InfiniteLoopSearchModel()

    exec_rec, _ = await runtime.execute_agent_loop(
        agent_id="company-research",
        objective="Research Acme architecture",
        task_id="task_bf",
        run_id=run_id,
        input_data={"company": "Acme"},
        model_override=infinite_model,
    )

    assert exec_rec.status == "completed"
    assert exec_rec.public_summary == "Forced finalization summary"

    assert exec_rec.tool_call_count <= exec_rec.budgets.tool_calls


class SchemaRepairFinalizerModel(BaseChatModel):
    attempted_invalid: bool = False

    @property
    def _llm_type(self) -> str:
        return "schema_repair_finalizer_model"

    def bind_tools(self, tools, **kwargs):
        return self

    def _generate(self, messages, stop=None, run_manager=None, **kwargs):
        raise NotImplementedError

    async def _agenerate(self, messages, stop=None, run_manager=None, **kwargs):
        messages_text = str([getattr(m, "content", "") for m in messages])
        if "Research budget is nearly exhausted" in messages_text:
            if not self.attempted_invalid:
                self.attempted_invalid = True
                # Emit an invalid payload with insufficient claims (<8)
                invalid_payload = {
                    "overview": "Acme Overview",
                    "business_model": "SaaS",
                    "engineering_culture": ["Pragmatic"],
                    "tech_stack": [],
                    "architecture_notes": ["Microservices"],
                    "interview_process_notes": ["Round 1"],
                    "claims": [
                        {
                            "statement": "Only one claim",
                            "category": "architecture",
                            "confidence": 0.95,
                            "evidence_ids": ["ev_0", "ev_1", "ev_2", "ev_3"],
                        }
                    ],
                    "summary": "Invalid initial output",
                }
                msg = AIMessage(
                    content="",
                    tool_calls=[
                        {
                            "name": "CompanyResearchOutput",
                            "args": invalid_payload,
                            "id": "tc_invalid_final",
                            "type": "tool_call",
                        }
                    ],
                )
                return ChatResult(generations=[ChatGeneration(message=msg)])

            # On second attempt (repair after validation error), emit valid output
            valid_payload = {
                "overview": "Acme Repaired Overview",
                "business_model": "SaaS",
                "engineering_culture": ["Pragmatic"],
                "tech_stack": [],
                "architecture_notes": ["Microservices"],
                "interview_process_notes": ["Round 1"],
                "claims": [
                    {
                        "statement": f"Repaired Claim {i}",
                        "category": "architecture",
                        "confidence": 0.95,
                        "evidence_ids": [f"ev_{k}" for k in range(4)],
                    }
                    for i in range(8)
                ],
                "summary": "Repaired finalization summary",
            }
            msg = AIMessage(
                content="",
                tool_calls=[
                    {
                        "name": "CompanyResearchOutput",
                        "args": valid_payload,
                        "id": "tc_repaired_final",
                        "type": "tool_call",
                    }
                ],
            )
            return ChatResult(generations=[ChatGeneration(message=msg)])

        tool_call = {
            "name": "web_search",
            "args": {"query": "continue searching"},
            "id": f"tc_{len(messages)}",
            "type": "tool_call",
        }
        msg = AIMessage(content="", tool_calls=[tool_call])
        return ChatResult(generations=[ChatGeneration(message=msg)])


@pytest.mark.asyncio
async def test_budget_finalizer_allows_schema_repair_without_exceeding_call_limits():
    ledger = SurrealLedgerRepository()
    run_id = f"run_test_schema_repair_{uuid.uuid4().hex[:6]}"
    await ledger.create_workflow_run(
        WorkflowRun(
            id=run_id,
            workflow_id="interview-preparation-static",
            workflow_name="Schema Repair Test",
            workspace_id="ws_sr",
            user_id="usr_sr",
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
    runtime = AutonomousAgentRuntime(ledger=ledger)
    repair_model = SchemaRepairFinalizerModel()

    exec_rec, _ = await runtime.execute_agent_loop(
        agent_id="company-research",
        objective="Research Acme architecture with repair",
        task_id="task_sr",
        run_id=run_id,
        input_data={"company": "Acme"},
        model_override=repair_model,
    )

    assert exec_rec.status == "completed"
    assert exec_rec.public_summary == "Repaired finalization summary"
    assert repair_model.attempted_invalid is True
