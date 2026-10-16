import asyncio
import os
import uuid

import pytest
from langchain_core.messages import AIMessage
from langchain_core.outputs import ChatGeneration, ChatResult
from openai import APITimeoutError

from src.agents.runtime import agent_runtime
from src.contracts.v1 import WorkflowRun
from src.orchestration.ledger import SurrealLedgerRepository
from src.orchestration.redis_queue import RedisStreamJobQueue
from src.orchestration.runtime.worker import DurableJobWorker
from tests.resume_double import resume_extraction_response
from tests.test_agent_runtime_tools import (
    ScriptedToolCallingFakeModel,
    requested_question_count,
    shape_curriculum_payload,
)


class FaultyModel(ScriptedToolCallingFakeModel):
    fail_count: int = 1
    current_calls: int = 0

    async def _agenerate(self, messages, stop=None, run_manager=None, **kwargs):
        self.current_calls += 1
        if self.current_calls <= self.fail_count:
            raise APITimeoutError(request=None)
        resume_result = resume_extraction_response(messages)
        if resume_result is not None:
            return resume_result
        from src.agents.runtime import current_agent_id_var
        from src.agents.schemas import AGENT_OUTPUT_SCHEMAS, AgentTaskOutput

        curr_agent = current_agent_id_var.get()
        from src.agents.schemas import CategoryQuestionBatchOutput

        if curr_agent == "interview-generation":
            target_cls = CategoryQuestionBatchOutput
        else:
            target_cls = AGENT_OUTPUT_SCHEMAS.get(curr_agent, AgentTaskOutput)
        if curr_agent == "company-research":
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
        elif curr_agent == "job-analysis":
            output_payload = {
                "role_summary": "Role Summary",
                "requirements": [
                    {
                        "skill": f"Skill {i}",
                        "level": "expert",
                        "required": "required",
                        "quote": "Must have skill",
                    }
                    for i in range(5)
                ],
                "responsibilities": ["Build software"],
                "seniority_signals": ["Senior"],
                "evidence_ids": ["ev_0"],
                "summary": "Recovered job summary",
            }
        elif curr_agent == "technical-research":
            output_payload = {
                "topics": [
                    {
                        "name": f"Topic {i}",
                        "why_it_matters": "Crucial",
                        "key_concepts": ["Concept"],
                        "pitfalls": ["Pitfall"],
                        "evidence_ids": [f"ev_{k}" for k in range(4)],
                    }
                    for i in range(4)
                ],
                "architecture_topology": {"system_type": "distributed"},
                "summary": "Recovered tech summary",
            }
        elif curr_agent == "evidence-evaluator":
            output_payload = {
                "overall_match_ratio": 0.88,
                "strengths": [
                    {
                        "skill": f"Strength {i}",
                        "evidence_quote": "Strong quote",
                        "relevance": "high",
                    }
                    for i in range(3)
                ],
                "gaps": [
                    {
                        "skill": f"Gap {i}",
                        "requirement_quote": "Gap quote",
                        "importance": "required",
                        "recommended_focus": "Focus",
                    }
                    for i in range(3)
                ],
                "risks": [],
                "wording_improvements": [],
                "claim_verdicts": [],
                "summary": "Recovered eval summary",
            }
        elif curr_agent == "curriculum":
            output_payload = {
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
                    {"source": f"Skill {i}", "target": f"Skill {i + 1}", "relation": "prerequisite"}
                    for i in range(10)
                ],
                "learning_units": [
                    {
                        "skill": f"Skill {i}",
                        "title": f"Unit {i}",
                        "prerequisites": [],
                        "estimated_minutes": 45,
                        "summary": "Summary",
                        "deep_dive_markdown": f"Deep Dive {i} " + "X" * 400,
                        "evidence_ids": ["ev_0"],
                    }
                    for i in range(6)
                ],
                "summary": "Recovered curriculum summary",
            }
        elif curr_agent == "systems-deep-dive":
            from src.agents.schemas import current_execution_saved_evidence_ids_var

            exec_evs = list(current_execution_saved_evidence_ids_var.get() or []) or [
                "ev_0",
                "ev_1",
            ]
            output_payload = {
                "summary": "Deep dive recovered summary",
                "subsystem": "storage",
                "findings": ["Finding 1", "Finding 2"],
                "evidence_ids": exec_evs[:2],
            }
        else:
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
            output_payload = {
                "questions": [
                    {
                        "question": f"Question {i}?",
                        "category": target_cat,
                        "difficulty": "medium",
                        "rationale": "Rationale",
                        "skills_tested": ["Architecture"],
                        "interviewer_intent": "Intent",
                        "expected_signals": ["Signal 1", "Signal 2"],
                        "answer_outline": ["Outline 1", "Outline 2", "Outline 3"],
                        "pitfalls": ["Pitfall 1", "Pitfall 2"],
                        "follow_ups": ["Followup 1?", "Followup 2?"],
                        "evidence_ids": ["ev_0"],
                    }
                    for i in range(requested_question_count(messages))
                ],
                "summary": "Recovered question summary",
            }
        if curr_agent == "curriculum":
            target_cls, output_payload = shape_curriculum_payload(output_payload, messages)

        msg = AIMessage(
            content="",
            tool_calls=[
                {
                    "name": target_cls.__name__,
                    "args": output_payload,
                    "id": "tc_faulty_success",
                    "type": "tool_call",
                }
            ],
        )
        return ChatResult(generations=[ChatGeneration(message=msg)])


@pytest.mark.asyncio
async def test_worker_retry_classification_transient_vs_terminal():
    ledger = SurrealLedgerRepository()
    redis_url = os.environ.get("REDIS_URL", "redis://127.0.0.1:16379")
    group_name = f"grp_retry_{uuid.uuid4().hex[:8]}"
    queue = RedisStreamJobQueue(redis_url=redis_url, group_name=group_name)

    run_id = f"run_retry_{uuid.uuid4().hex[:8]}"
    run = WorkflowRun(
        id=run_id,
        workflow_id="interview-preparation-static",
        workflow_name="Retry Test",
        workspace_id="ws_retry",
        user_id="usr_retry",
        type="static",
        status="running",
    )
    await ledger.create_workflow_run(run)
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
    worker = DurableJobWorker(worker_id="worker_retry_test", redis_url=redis_url, max_attempts=3)
    worker.queue = queue
    await worker.initialize()

    faulty_model = FaultyModel(fail_count=10)
    agent_runtime.set_model_factory(lambda: faulty_model)

    first_job = {
        "msg_id": "1-0",
        "run_id": run_id,
        "payload": {
            "workflow_id": "interview-preparation-static",
            "workspace_id": "ws_retry",
            "company_name": "Acme Retry",
            "role_title": "Engineer",
            "resume_text": "Engineer with Python and distributed systems experience.",
            "job_description": "Build resilient backend systems.",
        },
    }
    from unittest.mock import AsyncMock, patch

    with patch(
        "src.agents.runtime.SafeResearchFetcher.fetch_clean_text", new_callable=AsyncMock
    ) as mock_fetch:
        mock_fetch.return_value = (
            "Careers at Acme",
            "We build distributed systems in Python and Rust.",
        )
        await worker.execute_job(first_job)

        events = await ledger.get_events_history(run_id=run_id, limit=50)
        event_types = [e.type for e in events.events]
        assert "workflow.retrying" in event_types
        assert "workflow.failed" not in event_types

        await asyncio.sleep(1.2)
        claimed_second = await queue.claim_job(worker.worker_id, block_ms=1000)
        assert claimed_second is not None
        faulty_model.fail_count = 0
        await worker.execute_job(claimed_second)
        final_run = await ledger.get_workflow_run(run_id)
        assert final_run.status == "completed"
    term_run_id = f"run_terminal_retry_{uuid.uuid4().hex[:8]}"
    term_run = WorkflowRun(
        id=term_run_id,
        workflow_id="interview-preparation-static",
        workflow_name="Terminal Retry Run",
        workspace_id="ws_term_retry",
        user_id="usr_term_retry",
        type="static",
        status="running",
    )
    await ledger.create_workflow_run(term_run)

    term_model = FaultyModel(fail_count=99)
    agent_runtime.set_model_factory(lambda: term_model)

    job_payload = {
        "workflow_id": "interview-preparation-static",
        "workspace_id": "ws_term_retry",
        "company_name": "Terminal Acme",
        "role_title": "Engineer",
        "resume_text": "Engineer with Python and distributed systems experience.",
        "job_description": "Build resilient backend systems.",
        "attempt_count": 1,
    }

    await worker.execute_job({"msg_id": "1-0", "run_id": term_run_id, "payload": job_payload})

    await asyncio.sleep(1.2)
    job_2 = await queue.claim_job(worker.worker_id, block_ms=1000)
    assert job_2 is not None
    await worker.execute_job(job_2)

    await asyncio.sleep(2.2)
    job_3 = await queue.claim_job(worker.worker_id, block_ms=1000)
    assert job_3 is not None
    await worker.execute_job(job_3)
    term_events = await ledger.get_events_history(run_id=term_run_id, limit=50)
    print("TERM EVENTS:", [e.type for e in term_events.events])
    retrying_evts = [e for e in term_events.events if e.type == "workflow.retrying"]
    failed_evts = [e for e in term_events.events if e.type == "workflow.failed"]
    print("RETRYING:", len(retrying_evts), "FAILED:", len(failed_evts))

    term_final_run = await ledger.get_workflow_run(term_run_id)
    assert term_final_run.status == "failed"

    delay_run_id = f"run_delay_{uuid.uuid4().hex[:8]}"
    await queue.enqueue_job(delay_run_id, {"run_id": delay_run_id}, delay_ms=1500)

    await asyncio.sleep(0.5)
    unclaimed_early = await queue.claim_job("worker_early_probe", block_ms=200)
    assert unclaimed_early is None, "Job was claimed before delay expired!"

    await asyncio.sleep(1.1)
    claimed_due = await queue.claim_job("worker_due_probe", block_ms=500)
    assert claimed_due is not None
    assert claimed_due["run_id"] == delay_run_id
    await queue.ack_job(claimed_due["msg_id"])

    await worker.stop()
