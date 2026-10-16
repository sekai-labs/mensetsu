import re

import pytest
from langchain_core.language_models.chat_models import BaseChatModel
from langchain_core.messages import AIMessage

from src.agents.runtime import agent_runtime
from src.agents.schemas import AgentTaskOutput
from src.contracts.v1 import WorkflowRun
from src.orchestration.ledger import SurrealLedgerRepository
from tests.resume_double import resume_extraction_response


def curriculum_stage_from_messages(messages) -> str | None:
    for m in messages:
        content = getattr(m, "content", "")
        if isinstance(content, str):
            if '"curriculum_stage": "graph"' in content:
                return "graph"
            if '"curriculum_stage": "units"' in content:
                return "units"
    return None


def requested_question_count(messages, default: int = 4) -> int:
    for m in messages:
        content = getattr(m, "content", "")
        if isinstance(content, str):
            match = re.search(r'"count": (\d+)', content)
            if match and '"target_category"' in content:
                return int(match.group(1))
    return default


def shape_curriculum_payload(payload: dict, messages) -> tuple[type, dict]:
    from src.agents.schemas import CURRICULUM_STAGE_SCHEMAS, CurriculumOutput

    stage = curriculum_stage_from_messages(messages)
    if stage == "graph":
        return CURRICULUM_STAGE_SCHEMAS["graph"], {
            "skills": payload["skills"],
            "relations": payload["relations"],
            "summary": payload["summary"],
        }
    if stage == "units":
        return CURRICULUM_STAGE_SCHEMAS["units"], {
            "learning_units": payload["learning_units"][:2],
            "summary": payload["summary"],
        }
    return CurriculumOutput, payload


class ScriptedToolCallingFakeModel(BaseChatModel):
    call_index: int = 0

    @property
    def _llm_type(self) -> str:
        return "scripted_fake_chat_model"

    def bind_tools(self, tools, **kwargs):
        return self

    def _generate(self, messages, stop=None, run_manager=None, **kwargs):
        raise NotImplementedError

    async def _agenerate(self, messages, stop=None, run_manager=None, **kwargs):
        resume_result = resume_extraction_response(messages)
        if resume_result is not None:
            return resume_result
        num_msgs = len(messages)
        from src.agents.registry import registry
        from src.agents.runtime import current_agent_id_var

        curr_agent = current_agent_id_var.get()
        defn = registry.get(curr_agent) if curr_agent else None
        allowed = set(defn.allowed_tools) if defn else set()

        if num_msgs <= 2 and "web_search" in allowed:
            tool_call = {
                "name": "web_search",
                "args": {"query": "Acme engineering culture"},
                "id": "tc_1",
                "type": "tool_call",
            }
            msg = AIMessage(
                content="",
                tool_calls=[tool_call],
                usage_metadata={"input_tokens": 120, "output_tokens": 25, "total_tokens": 145},
            )
        elif num_msgs <= 4 and "read_source" in allowed:
            tool_call = {
                "name": "read_source",
                "args": {"url": "https://www.python.org/about/"},
                "id": "tc_2",
                "type": "tool_call",
            }
            msg = AIMessage(
                content="",
                tool_calls=[tool_call],
                usage_metadata={"input_tokens": 250, "output_tokens": 30, "total_tokens": 280},
            )
        elif num_msgs <= 6 and "delegate_task" in allowed:
            tool_call = {
                "name": "delegate_task",
                "args": {
                    "target_agent_id": "evidence-evaluator",
                    "objective": "Audit candidate evidence coverage",
                    "reason": "Specialist audit required",
                },
                "id": "tc_3",
                "type": "tool_call",
            }
            msg = AIMessage(
                content="",
                tool_calls=[tool_call],
                usage_metadata={"input_tokens": 350, "output_tokens": 40, "total_tokens": 390},
            )
        elif num_msgs <= 14 and "save_evidence" in allowed:
            ev_idx = (num_msgs - 6) // 2
            tool_call = {
                "name": "save_evidence",
                "args": {
                    "source_url": "https://www.python.org/about/",
                    "title": f"Careers at Acme Part {ev_idx}",
                    "snippet": "We build distributed systems in Python and Rust.",
                    "confidence": 0.95,
                },
                "id": f"tc_4_{ev_idx}",
                "type": "tool_call",
            }
            msg = AIMessage(
                content="",
                tool_calls=[tool_call],
                usage_metadata={"input_tokens": 450, "output_tokens": 50, "total_tokens": 500},
            )
        else:
            from src.agents.runtime import current_agent_id_var
            from src.agents.schemas import AGENT_OUTPUT_SCHEMAS

            target_cls = None
            from src.agents.schemas import CategoryQuestionBatchOutput

            all_known_schemas = list(AGENT_OUTPUT_SCHEMAS.values()) + [CategoryQuestionBatchOutput]
            for m in reversed(messages):
                txt = getattr(m, "content", "")
                if isinstance(txt, str):
                    for schema_cls in all_known_schemas:
                        if schema_cls.__name__ in txt:
                            target_cls = schema_cls
                            break
                if target_cls:
                    break

            if not target_cls:
                curr_agent = current_agent_id_var.get()
                if curr_agent == "interview-generation":
                    target_cls = CategoryQuestionBatchOutput
                else:
                    target_cls = AGENT_OUTPUT_SCHEMAS.get(curr_agent, AgentTaskOutput)

            if target_cls == AgentTaskOutput:
                from src.agents.schemas import current_saved_evidence_ids_var

                saved_evs = list(current_saved_evidence_ids_var.get() or []) or ["art_ev_0"]
                output_payload = {
                    "summary": "Grounded research confirms Acme uses distributed Python and Rust.",
                    "claims": [
                        {
                            "statement": "Acme uses Python and Rust.",
                            "category": "architecture",
                            "confidence": 0.95,
                            "evidence_ids": saved_evs[:1],
                        }
                    ],
                    "tools_used": ["web_search", "read_source", "delegate_task", "save_evidence"],
                    "status": "completed",
                    "confidence": 0.95,
                }
            else:
                from src.agents.schemas import current_saved_evidence_ids_var

                saved_evs = list(current_saved_evidence_ids_var.get() or []) or [
                    f"art_ev_{k}" for k in range(4)
                ]
                while len(saved_evs) < 4:
                    saved_evs.append(f"art_ev_{len(saved_evs)}")
                claims = [
                    {
                        "statement": f"Acme uses Rust {i}",
                        "category": "architecture",
                        "confidence": 0.95,
                        "evidence_ids": saved_evs[:4],
                    }
                    for i in range(8)
                ]
                topics = [
                    {
                        "name": "Consensus",
                        "why_it_matters": "Core consensus",
                        "key_concepts": ["Raft"],
                        "pitfalls": ["Split brain"],
                        "evidence_ids": saved_evs[:4],
                    }
                ]
                output_payload = {
                    "overview": "Acme is a global distributed systems engineering platform.",
                    "business_model": "Enterprise SaaS and cloud orchestration.",
                    "engineering_culture": ["Ownership", "High availability"],
                    "tech_stack": [
                        {
                            "name": "Rust",
                            "category": "Language",
                            "evidence_ids": saved_evs[:4],
                            "verified": True,
                            "notes": "Core engine",
                        }
                    ],
                    "architecture_notes": ["Event-driven distributed ledger"],
                    "interview_process_notes": ["Technical architecture round"],
                    "claims": claims,
                    "summary": "Grounded research confirms Acme uses distributed Python and Rust.",
                    "role_summary": "Platform architect role.",
                    "requirements": [
                        {
                            "skill": "Rust",
                            "level": "expert",
                            "required": "required",
                            "quote": "Must have Rust experience",
                        }
                    ],
                    "responsibilities": ["Design distributed systems"],
                    "seniority_signals": ["Staff level ownership"],
                    "evidence_ids": [saved_evs[0]],
                    "topics": topics,
                    "architecture_topology": {"system_type": "distributed_ledger"},
                    "questions": [
                        {
                            "question": f"Explain Raft consensus {i}?",
                            "category": "system_design",
                            "difficulty": "medium",
                            "rationale": "Core stack",
                            "skills_tested": ["Distributed Systems"],
                            "interviewer_intent": "Assess consensus knowledge",
                            "expected_signals": ["Raft mechanics", "Leader election"],
                            "answer_outline": [
                                "Leader election",
                                "Log replication",
                                "Safety guarantees",
                            ],
                            "pitfalls": ["Over-simplification", "Ignoring partitions"],
                            "follow_ups": [
                                "How to handle network partitions?",
                                "How does log compaction work?",
                            ],
                            "evidence_ids": ["art_ev_0"],
                        }
                        for i in range(
                            requested_question_count(messages)
                            if target_cls.__name__ == "CategoryQuestionBatchOutput"
                            else 15
                        )
                    ],
                    "overall_match_ratio": 0.92,
                    "strengths": [
                        {
                            "skill": f"Rust {i}",
                            "evidence_quote": "Production Rust architect",
                            "relevance": "high",
                        }
                        for i in range(3)
                    ],
                    "gaps": [
                        {
                            "skill": f"Kubernetes {i}",
                            "requirement_quote": "Kubernetes operations",
                            "importance": "required",
                            "recommended_focus": "Kubernetes operators",
                        }
                        for i in range(3)
                    ],
                    "risks": ["Scaling limits"],
                    "wording_improvements": [],
                    "claim_verdicts": [
                        {
                            "claim_statement": "Acme uses Rust",
                            "verified": True,
                            "verdict_explanation": "Verified in blog",
                            "evidence_ids": [],
                            "confidence": 0.95,
                        }
                    ],
                    "skills": [
                        {
                            "name": f"Skill {i}",
                            "slug": f"skill-{i}",
                            "category": "Systems",
                            "candidate_status": "in_progress",
                            "importance": "mandatory",
                        }
                        for i in range(12)
                    ],
                    "relations": [
                        {
                            "source": f"Skill {i}",
                            "target": f"Skill {i + 1}",
                            "relation": "prerequisite",
                        }
                        for i in range(10)
                    ],
                    "learning_units": [
                        {
                            "skill": f"Skill {i}",
                            "title": f"Unit {i}",
                            "prerequisites": [],
                            "estimated_minutes": 45,
                            "summary": "Summary",
                            "deep_dive_markdown": f"Deep dive on skill {i} " + "X" * 400,
                            "evidence_ids": ["art_ev_0"],
                        }
                        for i in range(6)
                    ],
                }

            if curr_agent == "curriculum":
                target_cls, output_payload = shape_curriculum_payload(output_payload, messages)
            elif curr_agent == "systems-deep-dive":
                from src.agents.schemas import current_execution_saved_evidence_ids_var

                exec_evs = (
                    list(current_execution_saved_evidence_ids_var.get() or []) or saved_evs[:2]
                )
                output_payload["subsystem"] = "storage"
                output_payload["findings"] = ["Compaction latency", "Replication throughput"]
                output_payload["evidence_ids"] = exec_evs[:2]
            format_tool_name = target_cls.__name__
            msg = AIMessage(
                content="",
                tool_calls=[
                    {
                        "name": format_tool_name,
                        "args": output_payload,
                        "id": "tc_tool_output",
                        "type": "tool_call",
                    }
                ],
                usage_metadata={"input_tokens": 520, "output_tokens": 80, "total_tokens": 600},
            )
            from langchain_core.outputs import ChatGeneration, ChatResult

            return ChatResult(generations=[ChatGeneration(message=msg)])
        from langchain_core.outputs import ChatGeneration, ChatResult

        return ChatResult(generations=[ChatGeneration(message=msg)])


@pytest.mark.asyncio
async def test_agent_loop_real_tool_sequence_and_provenance():
    import uuid
    from unittest.mock import AsyncMock, patch

    from src.contracts.v1 import Artifact

    ledger = SurrealLedgerRepository()
    run_id = f"run_scripted_{uuid.uuid4().hex[:8]}"
    await ledger.create_workflow_run(
        WorkflowRun(
            id=run_id,
            workflow_id="interview-preparation-static",
            workflow_name="Scripted",
            workspace_id="ws_scripted",
            user_id="usr_scripted",
            type="static",
            status="running",
        )
    )
    for k in range(5):
        await ledger.save_artifact(
            Artifact(
                id=f"art_ev_{k}",
                run_id=run_id,
                type="evidence",
                title=f"Evidence {k}",
                content={"snippet": "Evidence text"},
            )
        )
    fake_model = ScriptedToolCallingFakeModel()
    with patch(
        "src.agents.runtime.SafeResearchFetcher.fetch_clean_text", new_callable=AsyncMock
    ) as mock_fetch:
        mock_fetch.return_value = (
            "Careers at Acme",
            "We build distributed systems in Python and Rust.",
        )
        exec_rec, _ = await agent_runtime.execute_agent_loop(
            agent_id="technical-research",
            objective="Analyze Acme engineering stack",
            task_id=f"task_{run_id[:8]}",
            run_id=run_id,
            input_data={"company": "Acme"},
            model_override=fake_model,
        )

    tools_page = await ledger.list_tools_for_execution(exec_rec.id)
    inv_names = [t.tool_name for t in tools_page.items]
    assert "web_search" in inv_names
    assert "read_source" in inv_names
    assert "delegate_task" in inv_names
    assert "save_evidence" in inv_names

    delegations = await ledger.list_delegations_for_run(run_id)
    assert len(delegations) >= 1
    completed_dels = [d for d in delegations if d.status == "completed"]
    assert len(completed_dels) >= 1
    assert completed_dels[0].parent_agent_id == "technical-research"
    assert completed_dels[0].child_agent_id == "evidence-evaluator"

    for t in tools_page.items:
        assert t.duration_ms >= 0.0

    artifacts = await ledger.list_artifacts_for_run(run_id)
    assert len(artifacts) >= 1
    evidence_art = next(a for a in artifacts if a.type == "evidence")
    assert evidence_art.evidence is not None
    assert evidence_art.evidence.source_url == "https://www.python.org/about/"
    assert evidence_art.evidence.confidence == 0.95
    assert "distributed systems" in evidence_art.evidence.snippet.lower()

    assert exec_rec.budgets_used.tool_calls >= 4
    assert exec_rec.budgets_used.tokens_in > 0
    assert exec_rec.budgets_used.tokens_out > 0
    assert exec_rec.budgets_used.wall_time_ms > 0.0


class EmptyFirstReplyModel(ScriptedToolCallingFakeModel):
    empty_replies: int = 1
    calls: int = 0
    nudges_seen: int = 0

    async def _agenerate(self, messages, stop=None, run_manager=None, **kwargs):
        from langchain_core.outputs import ChatGeneration, ChatResult

        self.calls += 1
        self.nudges_seen = sum(
            1 for m in messages if "contained no tool call" in str(getattr(m, "content", ""))
        )
        if self.calls <= self.empty_replies:
            return ChatResult(generations=[ChatGeneration(message=AIMessage(content=""))])
        return await super()._agenerate(messages, stop=stop, run_manager=run_manager, **kwargs)


async def create_scripted_run(prefix: str) -> str:
    import uuid

    from src.contracts.v1 import Artifact

    ledger = SurrealLedgerRepository()
    run_id = f"run_{prefix}_{uuid.uuid4().hex[:8]}"
    await ledger.create_workflow_run(
        WorkflowRun(
            id=run_id,
            workflow_id="interview-preparation-static",
            workflow_name="Scripted",
            workspace_id=f"ws_{prefix}",
            user_id=f"usr_{prefix}",
            type="static",
            status="running",
        )
    )
    for k in range(5):
        await ledger.save_artifact(
            Artifact(
                id=f"art_ev_{k}",
                run_id=run_id,
                type="evidence",
                title=f"Evidence {k}",
                content={"snippet": "Evidence text"},
            )
        )
    return run_id


@pytest.mark.asyncio
async def test_empty_model_reply_is_reprompted_for_output_tool_within_execution():
    run_id = await create_scripted_run("empty_reply")
    model = EmptyFirstReplyModel(empty_replies=1)
    exec_rec, data = await agent_runtime.execute_agent_loop(
        agent_id="evidence-evaluator",
        objective="Audit candidate evidence coverage",
        task_id=f"task_{run_id[:8]}",
        run_id=run_id,
        input_data={"claims": []},
        model_override=model,
    )

    assert exec_rec.status == "completed"
    assert model.calls == 2
    assert model.nudges_seen == 1
    assert data["strengths"]
    events = await SurrealLedgerRepository().get_events_history(run_id=run_id, limit=100)
    reprompts = [
        e
        for e in events.events
        if e.type == "agent.activity"
        and getattr(e.payload, "activity_type", None) == "output_reprompt"
    ]
    assert len(reprompts) == 1


@pytest.mark.asyncio
async def test_persistently_empty_model_reply_fails_after_bounded_reprompts():
    from src.agents.runtime import MAX_MISSING_OUTPUT_TOOL_NUDGES

    run_id = await create_scripted_run("empty_always")
    model = EmptyFirstReplyModel(empty_replies=99)
    with pytest.raises(ValueError, match="failed to produce valid structured output"):
        await agent_runtime.execute_agent_loop(
            agent_id="evidence-evaluator",
            objective="Audit candidate evidence coverage",
            task_id=f"task_{run_id[:8]}",
            run_id=run_id,
            input_data={"claims": []},
            model_override=model,
        )
    assert model.calls == 1 + MAX_MISSING_OUTPUT_TOOL_NUDGES
