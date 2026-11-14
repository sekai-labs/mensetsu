"""Regression tests for bounded replanning in static workflow agent retries."""
from types import SimpleNamespace
from unittest.mock import AsyncMock

import pytest

from src.agents.runtime import agent_runtime
from src.evals import OutputQualityError
from src.orchestration.static_curriculum import build_curriculum_nodes
from src.orchestration.static_outputs import build_output_nodes
from src.orchestration.static_questions import build_question_nodes
from src.orchestration.static_research import build_research_nodes
from src.orchestration.static_synthesis import build_synthesis_nodes
from src.orchestration.static_support import StaticWorkflowState


class FakeLedger:
    def __init__(self):
        self.tasks: dict[str, object] = {}
        self.events: list[dict] = []
        self.db = None

    async def list_tasks_for_run(self, run_id):
        return list(self.tasks.values())

    async def get_task(self, run_id, task_id):
        return self.tasks.get(task_id)

    async def next_task_attempt(self, run_id, task_id):
        existing = self.tasks.get(task_id)
        return existing.attempt + 1 if existing is not None else 1

    async def save_task(self, task):
        self.tasks[task.id] = task

    async def append_event_transactional(self, **kwargs):
        self.events.append(kwargs)

    async def list_artifacts_for_run(self, run_id):
        return []


def build_support(ledger: FakeLedger) -> SimpleNamespace:
    return SimpleNamespace(
        ledger=ledger,
        db=None,
        check_lease=lambda: None,
        set_phase=AsyncMock(),
        completed_task_deliverable=AsyncMock(return_value=None),
    )


@pytest.mark.asyncio
async def test_company_research_second_attempt_replans_with_prior_error(monkeypatch):
    ledger = FakeLedger()
    calls: list[dict] = []

    async def fake_execute_agent_loop(**kwargs):
        calls.append(kwargs)
        if len(calls) == 1:
            raise RuntimeError("injected company research failure")
        return SimpleNamespace(id="exec_company_recovered", public_summary={}), {
            "overview": "Recovered company research"
        }

    monkeypatch.setattr(agent_runtime, "execute_agent_loop", fake_execute_agent_loop)
    company_node, _, _ = build_research_nodes(build_support(ledger))
    state = StaticWorkflowState(
        run_id="run_static_company_replan",
        workspace_id="ws_static_company_replan",
        company_name="Acme Systems",
        company_urls=["https://acme.example.com"],
        role_title="Staff Engineer",
        job_description="Build distributed services.",
    )

    result = await company_node(state)

    assert result == {
        "research_results": {"company": {"overview": "Recovered company research"}}
    }
    assert len(calls) == 2
    assert calls[0]["recovery_context"] is None
    recovery = calls[1]["recovery_context"]
    assert recovery is not None
    assert recovery.attempt == 1
    assert "injected company research failure" in recovery.previous_error
    assert "source-finding" in recovery.alternative_strategy
    assert calls[1]["agent_id"] == "company-research"
    assert calls[1]["task_id"] == calls[0]["task_id"]
    assert calls[1]["objective"] == calls[0]["objective"]
    assert calls[1]["input_data"]["retry_attempt"] == 2

    task = ledger.tasks[f"task_company_{state.run_id[:8]}"]
    assert task.status == "completed"
    assert task.attempt == 2
    assert "task.retrying" in [event["event_type"] for event in ledger.events]


@pytest.mark.asyncio
async def test_question_batch_retry_replans_with_prior_error(monkeypatch):
    ledger = FakeLedger()
    calls: list[dict] = []
    failed_once = False

    async def fake_execute_agent_loop(**kwargs):
        nonlocal failed_once
        calls.append(kwargs)
        batch = kwargs["input_data"]
        category = batch["target_category"]
        part = batch["batch_part"]
        if (category, part) == ("system_design", 2) and not failed_once:
            failed_once = True
            raise RuntimeError("injected question batch failure")
        questions = [
            {
                "question": f"{category} question {index}",
                "difficulty": "medium",
                "skills_tested": ["Python"],
                "interviewer_intent": "Assess fundamentals",
                "answer_outline": ["Explain the approach"],
                "expected_signals": ["Clear reasoning"],
                "pitfalls": ["Vague answer"],
                "follow_ups": ["Why that tradeoff?"],
            }
            for index in range(batch["count"])
        ]
        return (
            SimpleNamespace(id=f"exec_q_{category}_{part}_{len(calls)}", public_summary={}),
            {"questions": questions},
        )

    monkeypatch.setattr(agent_runtime, "execute_agent_loop", fake_execute_agent_loop)
    generate_node, _, _ = build_question_nodes(build_support(ledger))
    state = StaticWorkflowState(
        run_id="run_static_question_replan",
        workspace_id="ws_static_question_replan",
        company_name="Acme Systems",
        role_title="Staff Engineer",
        curriculum={
            "skills": [{"name": "Python"}, {"name": "Distributed Systems"}]
        },
    )

    result = await generate_node(state)

    assert len(result["questions"]) == 7 * 3 * 5
    batch_calls = [
        call
        for call in calls
        if call["input_data"]["target_category"] == "system_design"
        and call["input_data"]["batch_part"] == 2
    ]
    assert len(calls) == 7 * 3 + 1
    assert len(batch_calls) == 2
    assert batch_calls[0]["recovery_context"] is None
    recovery = batch_calls[1]["recovery_context"]
    assert recovery is not None
    assert recovery.attempt == 1
    assert "injected question batch failure" in recovery.previous_error
    assert "question angle" in recovery.alternative_strategy
    assert batch_calls[1]["agent_id"] == "interview-generation"
    assert batch_calls[1]["task_id"] == batch_calls[0]["task_id"]
    assert batch_calls[1]["objective"] == batch_calls[0]["objective"]


@pytest.mark.asyncio
async def test_curriculum_second_attempt_replans_with_prior_error(monkeypatch):
    ledger = FakeLedger()
    calls: list[dict] = []

    async def fake_execute_agent_loop(**kwargs):
        calls.append(kwargs)
        if len(calls) == 1:
            raise RuntimeError("injected curriculum graph build failure")
        stage = kwargs["input_data"].get("curriculum_stage")
        if stage == "graph":
            return SimpleNamespace(id="exec_curriculum_graph", public_summary={}), {
                "skills": [
                    {
                        "name": f"Skill {i}",
                        "slug": f"skill-{i}",
                        "category": "Architecture",
                        "candidate_status": "gap",
                        "importance": "mandatory",
                    }
                    for i in range(6)
                ],
                "relations": [
                    {"source": f"Skill {i}", "target": f"Skill {i}", "relation": "complements"}
                    for i in range(6)
                ],
                "summary": "Recovered graph",
            }
        target_skills = kwargs["input_data"].get("target_skills", [])
        return SimpleNamespace(id=f"exec_unit_{len(calls)}", public_summary={}), {
            "learning_units": [
                {
                    "title": f"Study {skill['name']}",
                    "skill": skill["name"],
                    "summary": f"Fundamentals of {skill['name']}",
                    "deep_dive_markdown": f"Content for {skill['name']}",
                    "prerequisites": [],
                    "estimated_minutes": 45,
                }
                for skill in target_skills
            ]
        }

    monkeypatch.setattr(agent_runtime, "execute_agent_loop", fake_execute_agent_loop)
    curriculum_node = build_curriculum_nodes(build_support(ledger))
    state = StaticWorkflowState(
        run_id="run_static_curriculum_replan",
        workspace_id="ws_static_curriculum_replan",
        company_name="Acme Systems",
        role_title="Staff Engineer",
        parsed_resume={"extracted_skills": [{"name": "Skill 0"}]},
        match_analysis={"gaps": ["Skill 1", "Skill 2"]},
        research_results={
            "job": {"requirements": [{"skill": "Skill 0"}]},
            "tech": {"topics": [{"name": "Skill 1"}]},
        },
    )

    result = await curriculum_node(state)

    assert len(result["curriculum"]["skills"]) == 6
    assert len(result["curriculum"]["learning_units"]) >= 6
    assert len(calls) >= 3
    assert calls[0]["recovery_context"] is None
    recovery = calls[1]["recovery_context"]
    assert recovery is not None
    assert recovery.attempt == 1
    assert "injected curriculum graph build failure" in recovery.previous_error
    assert "curriculum" in recovery.alternative_strategy.lower() or "smaller steps" in recovery.alternative_strategy.lower()
    assert calls[1]["agent_id"] == "curriculum"
    assert calls[1]["task_id"] == calls[0]["task_id"]
    assert calls[1]["objective"] == calls[0]["objective"]

    task = ledger.tasks[f"task_curriculum_{state.run_id[:8]}"]
    assert task.status == "completed"
    assert task.attempt == 2
    assert "task.retrying" in [event["event_type"] for event in ledger.events]


@pytest.mark.asyncio
async def test_synthesis_second_attempt_replans_with_prior_error(monkeypatch):
    ledger = FakeLedger()
    calls: list[dict] = []

    async def fake_execute_agent_loop(**kwargs):
        calls.append(kwargs)
        if len(calls) == 1:
            raise RuntimeError("injected synthesis evaluation failure")
        return SimpleNamespace(id="exec_synth_recovered", public_summary={}), {
            "overall_match_ratio": 0.85,
            "strengths": [{"skill": "Python", "evidence_quote": "5 years Python"}],
            "gaps": [],
            "risks": [],
            "wording_improvements": [],
        }

    monkeypatch.setattr(agent_runtime, "execute_agent_loop", fake_execute_agent_loop)
    synthesize_node = build_synthesis_nodes(build_support(ledger))
    state = StaticWorkflowState(
        run_id="run_static_synth_replan",
        workspace_id="ws_static_synth_replan",
        company_name="Acme Systems",
        role_title="Staff Engineer",
        parsed_resume={"extracted_skills": [{"name": "Python"}]},
        research_results={
            "job": {"role_summary": "Staff Engineer", "requirements": []},
            "company": {"overview": "Acme overview", "claims": []},
            "tech": {"topics": []},
        },
    )

    result = await synthesize_node(state)

    assert result["match_analysis"]["overall_match_ratio"] == 0.85
    assert len(calls) == 2
    assert calls[0]["recovery_context"] is None
    recovery = calls[1]["recovery_context"]
    assert recovery is not None
    assert recovery.attempt == 1
    assert "injected synthesis evaluation failure" in recovery.previous_error
    assert "claim-to-evidence" in recovery.alternative_strategy
    assert calls[1]["agent_id"] == "evidence-evaluator"
    assert calls[1]["task_id"] == calls[0]["task_id"]
    assert calls[1]["objective"] == calls[0]["objective"]

    task = ledger.tasks[f"task_synth_{state.run_id[:8]}"]
    assert task.status == "completed"
    assert task.attempt == 2
    assert "task.retrying" in [event["event_type"] for event in ledger.events]


@pytest.mark.asyncio
async def test_static_outputs_enforce_output_contract_invariants():
    ledger = FakeLedger()
    support = build_support(ledger)
    persist_node = build_output_nodes(support, AsyncMock(return_value={"questions": []}))
    state = StaticWorkflowState(
        run_id="run_static_contract_check",
        workspace_id="ws_static_contract_check",
        company_name="Acme Systems",
        role_title="Staff Engineer",
        questions=[{"question": f"Valid question {i}?", "skills_tested": ["Rust"]} for i in range(100)],
        curriculum={
            "skills": [{"name": "Rust"}],
            "relations": [{"source": "Rust", "target": "Rust", "relation": "complements"}],
        },
        research_results={
            "company": {
                "claims": [
                    {
                        "statement": "Unsupported claim",
                        "evidence_ids": ["ev_nonexistent_artifact"],
                    }
                ]
            }
        },
    )

    with pytest.raises(OutputQualityError) as exc_info:
        await persist_node(state)

    assert "citation_provenance" in str(exc_info.value)
    assert "ev_nonexistent_artifact" in str(exc_info.value)
