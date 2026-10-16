import pytest

from src.agents.runtime import agent_runtime
from src.contracts.v1 import WorkflowRun
from src.orchestration.ledger import SurrealLedgerRepository
from src.orchestration.static_workflow import StaticWorkflowState, build_static_workflow
from tests.resume_double import resume_extraction_response
from tests.test_agent_runtime_tools import (
    ScriptedToolCallingFakeModel,
    requested_question_count,
    shape_curriculum_payload,
)


class MarkerCheckingModel(ScriptedToolCallingFakeModel):
    async def _agenerate(self, messages, stop=None, run_manager=None, **kwargs):
        resume_result = resume_extraction_response(messages)
        if resume_result is not None:
            return resume_result
        from langchain_core.messages import AIMessage
        from langchain_core.outputs import ChatGeneration, ChatResult

        from src.agents.runtime import current_agent_id_var
        from src.agents.schemas import (
            AGENT_OUTPUT_SCHEMAS,
            AgentTaskOutput,
            CategoryQuestionBatchOutput,
        )

        curr_agent = current_agent_id_var.get()
        if curr_agent == "interview-generation":
            target_cls = CategoryQuestionBatchOutput
        else:
            target_cls = AGENT_OUTPUT_SCHEMAS.get(curr_agent, AgentTaskOutput)
        if curr_agent == "company-research":
            payload = {
                "overview": "MARKER_COMPANY_OVERVIEW",
                "business_model": "MARKER_BUSINESS_MODEL",
                "engineering_culture": ["MARKER_CULTURE_1", "MARKER_CULTURE_2"],
                "tech_stack": [
                    {
                        "name": "MARKER_TECH_RUST",
                        "category": "Systems",
                        "evidence_ids": [],
                        "verified": True,
                    }
                ],
                "architecture_notes": ["MARKER_ARCH_NOTES"],
                "interview_process_notes": ["MARKER_INTERVIEW_PROCESS"],
                "claims": [
                    {
                        "statement": f"MARKER_CLAIM_{i}",
                        "category": "architecture",
                        "confidence": 0.95,
                        "evidence_ids": [f"ev_{k}" for k in range(4)],
                    }
                    for i in range(8)
                ],
                "summary": "MARKER_COMPANY_SUMMARY",
            }
        elif curr_agent == "job-analysis":
            payload = {
                "role_summary": "MARKER_ROLE_SUMMARY",
                "requirements": [
                    {
                        "skill": f"MARKER_REQ_SKILL_{i}",
                        "level": "expert",
                        "required": "required",
                        "quote": "Must have skill",
                    }
                    for i in range(5)
                ],
                "responsibilities": ["MARKER_RESP_1", "MARKER_RESP_2"],
                "seniority_signals": ["MARKER_SENIORITY_STAFF"],
                "evidence_ids": ["ev_0"],
                "summary": "MARKER_JOB_SUMMARY",
            }
        elif curr_agent == "technical-research":
            payload = {
                "topics": [
                    {
                        "name": f"MARKER_TOPIC_{i}",
                        "why_it_matters": "Crucial",
                        "key_concepts": ["Concept"],
                        "pitfalls": ["Pitfall"],
                        "evidence_ids": [f"ev_{k}" for k in range(4)],
                    }
                    for i in range(4)
                ],
                "architecture_topology": {"system_type": "MARKER_TOPOLOGY"},
                "summary": "MARKER_TECH_SUMMARY",
            }
        elif curr_agent == "evidence-evaluator":
            payload = {
                "overall_match_ratio": 0.93,
                "strengths": [
                    {
                        "skill": f"MARKER_STRENGTH_{i}",
                        "evidence_quote": "Strong quote",
                        "relevance": "high",
                    }
                    for i in range(3)
                ],
                "gaps": [
                    {
                        "skill": f"MARKER_GAP_{i}",
                        "requirement_quote": "Gap quote",
                        "importance": "required",
                        "recommended_focus": "Focus",
                    }
                    for i in range(3)
                ],
                "risks": ["MARKER_RISK_1"],
                "wording_improvements": [],
                "claim_verdicts": [],
                "summary": "MARKER_EVAL_SUMMARY",
            }
        elif curr_agent == "curriculum":
            payload = {
                "skills": [
                    {
                        "name": f"MARKER_SKILL_{i}",
                        "slug": f"marker-skill-{i}",
                        "category": "Systems",
                        "candidate_status": "in_progress",
                        "importance": "mandatory",
                    }
                    for i in range(12)
                ],
                "relations": [
                    {
                        "source": f"MARKER_SKILL_{i}",
                        "target": f"MARKER_SKILL_{i + 1}",
                        "relation": "prerequisite",
                    }
                    for i in range(10)
                ],
                "learning_units": [
                    {
                        "skill": f"MARKER_SKILL_{i}",
                        "title": f"MARKER_UNIT_{i}",
                        "prerequisites": [],
                        "estimated_minutes": 45,
                        "summary": "Summary",
                        "deep_dive_markdown": f"MARKER_DEEP_DIVE_{i}_" + "X" * 400,
                        "evidence_ids": ["ev_0"],
                    }
                    for i in range(6)
                ],
                "summary": "MARKER_CURRICULUM_SUMMARY",
            }
        elif curr_agent == "interview-generation":
            target_cat = "system_design"
            for m in messages:
                cnt = getattr(m, "content", "")
                for c in [
                    "system_design",
                    "target_stack",
                    "behavioral",
                    "technical_fundamentals",
                ]:
                    if c in cnt:
                        target_cat = c
            payload = {
                "questions": [
                    {
                        "question": f"MARKER_QUESTION_{i}?",
                        "category": target_cat,
                        "difficulty": "medium",
                        "rationale": "Test candidate",
                        "skills_tested": ["MARKER_SKILL"],
                        "interviewer_intent": "MARKER_INTENT",
                        "expected_signals": ["Signals 1", "Signals 2"],
                        "answer_outline": ["Outline 1", "Outline 2", "Outline 3"],
                        "pitfalls": ["Pitfalls 1", "Pitfalls 2"],
                        "follow_ups": ["Followup 1?", "Followup 2?"],
                        "evidence_ids": ["ev_0"],
                    }
                    for i in range(
                        requested_question_count(messages)
                        if target_cls.__name__ == "CategoryQuestionBatchOutput"
                        else 15
                    )
                ],
                "summary": "MARKER_QUESTIONS_SUMMARY",
            }
        elif curr_agent == "systems-deep-dive":
            from src.agents.schemas import current_execution_saved_evidence_ids_var

            exec_evs = list(current_execution_saved_evidence_ids_var.get() or []) or [
                "ev_0",
                "ev_1",
            ]
            payload = {
                "summary": "MARKER_DEEP_DIVE_SUMMARY",
                "subsystem": "storage",
                "findings": ["MARKER_FINDING_1", "MARKER_FINDING_2"],
                "evidence_ids": exec_evs[:2],
            }
        else:
            payload = {"summary": "MARKER_GENERIC_SUMMARY"}

        if curr_agent == "curriculum":
            target_cls, payload = shape_curriculum_payload(payload, messages)

        msg = AIMessage(
            content="",
            tool_calls=[
                {
                    "name": target_cls.__name__,
                    "args": payload,
                    "id": "tc_marker",
                    "type": "tool_call",
                }
            ],
        )
        return ChatResult(generations=[ChatGeneration(message=msg)])


@pytest.mark.asyncio
async def test_static_workflow_no_fabricated_literals():

    ledger = SurrealLedgerRepository()
    run_id = "run_test_markers"
    ws_id = "ws_test_markers"

    await ledger.create_workflow_run(
        WorkflowRun(
            id=run_id,
            workflow_id="interview-preparation-static",
            workflow_name="Marker Test",
            workspace_id=ws_id,
            user_id="usr_markers",
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
                content={"snippet": "Evidence snippet"},
            )
        )

    agent_runtime.set_model_factory(lambda: MarkerCheckingModel())

    import os

    from langgraph.checkpoint.redis.aio import AsyncRedisSaver

    saver = AsyncRedisSaver(redis_url=os.environ.get("REDIS_URL", "redis://127.0.0.1:16379"))
    await saver.asetup()

    graph = build_static_workflow(checkpointer=saver, ledger=ledger, db_client=ledger.db)
    state = StaticWorkflowState(
        run_id=run_id,
        workspace_id=ws_id,
        company_name="Marker Corp",
        role_title="Lead Architect",
        resume_text="Candidate with 10 years experience building distributed engines in Rust.",
        job_description="Seeking Lead Architect with deep knowledge of distributed protocols.",
    )

    res = await graph.ainvoke(state, config={"configurable": {"thread_id": run_id}})
    assert res is not None

    db = ledger.db

    cr_res = await db.execute_query(
        f"SELECT * FROM company_research WHERE workspace_id = type::record('workspace', '{ws_id}');"
    )
    cr_row = cr_res[0]["result"][0]
    assert cr_row["dossier"]["overview"] == "MARKER_COMPANY_OVERVIEW"

    jd_res = await db.execute_query(
        f"SELECT * FROM job_description WHERE workspace_id = type::record('workspace', '{ws_id}');"
    )
    assert "MARKER_ROLE_SUMMARY" in str(jd_res)

    ma_res = await db.execute_query(
        f"SELECT * FROM match_analysis WHERE workspace_id = type::record('workspace', '{ws_id}');"
    )
    assert "MARKER_STRENGTH_" in str(ma_res)
    assert "MARKER_GAP_" in str(ma_res)
    assert ma_res[0]["result"][0]["overall_match_ratio"] == 0.93

    skills = await db.execute_query(
        f"SELECT * FROM skill WHERE workspace_id = type::record('workspace', '{ws_id}');"
    )
    assert len(skills[0]["result"]) >= 12
    assert "MARKER_SKILL_" in str(skills)

    relations = await db.execute_query(
        f"SELECT * FROM skill_relation WHERE workspace_id = type::record('workspace', '{ws_id}');"
    )
    assert len(relations[0]["result"]) >= 10

    units = await db.execute_query(
        f"SELECT * FROM learning_unit WHERE workspace_id = type::record('workspace', '{ws_id}');"
    )
    assert len(units[0]["result"]) >= 6
    assert "MARKER_UNIT_" in str(units)
    assert len(units[0]["result"][0]["content_markdown"]) >= 400

    questions = await db.execute_query(
        f"SELECT * FROM interview_question WHERE workspace_id = type::record('workspace', '{ws_id}');"
    )
    assert len(questions[0]["result"]) >= 15
    assert "MARKER_QUESTION_" in str(questions)

    run_obj = await ledger.get_workflow_run(run_id)
    assert run_obj.status == "completed"
    assert run_obj.progress.completed >= 6
    assert run_obj.progress.total_tasks >= 6
    assert run_obj.degraded is False
    assert run_obj.degraded_reasons == []


class CompanyAlwaysFailsModel(MarkerCheckingModel):
    async def _agenerate(self, messages, stop=None, run_manager=None, **kwargs):
        from src.agents.runtime import current_agent_id_var

        if current_agent_id_var.get() == "company-research":
            raise ValueError("company research source unavailable")
        return await super()._agenerate(messages, stop=stop, run_manager=run_manager, **kwargs)


@pytest.mark.asyncio
async def test_optional_research_failure_marks_run_degraded():
    import os

    from langgraph.checkpoint.redis.aio import AsyncRedisSaver

    from src.contracts.v1 import Artifact

    ledger = SurrealLedgerRepository()
    run_id = "run_test_degraded"
    ws_id = "ws_test_degraded"
    await ledger.create_workflow_run(
        WorkflowRun(
            id=run_id,
            workflow_id="interview-preparation-static",
            workflow_name="Degraded Test",
            workspace_id=ws_id,
            user_id="usr_degraded",
            type="static",
            status="running",
        )
    )
    for k in range(5):
        await ledger.save_artifact(
            Artifact(
                id=f"ev_{k}",
                run_id=run_id,
                type="evidence",
                title=f"Evidence {k}",
                content={"snippet": "s"},
            )
        )
    agent_runtime.set_model_factory(lambda: CompanyAlwaysFailsModel())
    saver = AsyncRedisSaver(redis_url=os.environ.get("REDIS_URL", "redis://127.0.0.1:16379"))
    await saver.asetup()
    graph = build_static_workflow(checkpointer=saver, ledger=ledger, db_client=ledger.db)
    await graph.ainvoke(
        StaticWorkflowState(
            run_id=run_id,
            workspace_id=ws_id,
            company_name="Degraded Corp",
            role_title="Lead Architect",
            resume_text="Candidate with 10 years experience building distributed engines in Rust.",
            job_description="Seeking Lead Architect with deep knowledge of distributed protocols.",
        ),
        config={"configurable": {"thread_id": run_id}},
    )

    run_obj = await ledger.get_workflow_run(run_id)
    assert run_obj.status == "completed"
    assert run_obj.degraded is True
    assert len(run_obj.degraded_reasons) == 1
    assert run_obj.degraded_reasons[0].startswith("company research unavailable")

    events, after = [], 0
    while True:
        page = await ledger.get_events_history(run_id=run_id, after_seq=after, limit=100)
        events.extend(page.events)
        if page.next_after_seq is None:
            break
        after = page.next_after_seq
    completed = next(e for e in events if e.type == "workflow.completed")
    assert completed.payload.degraded is True
    assert completed.payload.degraded_reasons == run_obj.degraded_reasons

    open_execs = [
        e
        for e in await ledger.list_agent_executions_for_run(run_id)
        if e.status in ("pending", "running")
    ]
    assert open_execs == []
