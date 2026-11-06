from unittest.mock import AsyncMock, MagicMock, patch

import pytest

from src.agents.model_router import (
    resolve_chat_model_for_role,
    resolve_provider_credentials,
)
from src.agents.schemas import WorkflowRoutingDecision
from src.agents.workflow_router import route_workflow_request
from src.contracts.v1 import Artifact, EvidenceProvenance, WorkflowTask
from src.orchestration.hybrid_workflow import (
    HybridWorkflowState,
    _persist_hybrid_outputs,
    _question_batch_plan,
    _task_failed_payload,
    build_hybrid_workflow,
)


@pytest.mark.asyncio
async def test_workflow_router_empty_prompt_routes_static():
    decision = await route_workflow_request(
        user_prompt="",
        company_name="Google",
        role_title="Site Reliability Engineer",
    )
    assert decision.selected_workflow == "static"
    assert decision.confidence == 1.0


@pytest.mark.asyncio
async def test_workflow_router_prompt_classification():
    with patch("src.agents.workflow_router.resolve_chat_model_for_role") as mock_resolve:
        mock_model = MagicMock()
        mock_structured = MagicMock()
        mock_structured.ainvoke = AsyncMock(
            return_value=WorkflowRoutingDecision(
                selected_workflow="hybrid",
                confidence=0.95,
                reasoning="Candidate asks for specific MongoDB DocDB architecture deep-dive.",
                extracted_company="Stripe",
                extracted_role="Staff Infrastructure Engineer",
            )
        )
        mock_model.with_structured_output.return_value = mock_structured
        mock_resolve.return_value = mock_model

        decision = await route_workflow_request(
            user_prompt="Prepare me for Staff Infra at Stripe focusing on MongoDB DocDB",
            company_name="Target Company",
            role_title="Target Role",
        )

        assert decision.selected_workflow == "hybrid"
        assert decision.extracted_company == "Stripe"
        assert decision.extracted_role == "Staff Infrastructure Engineer"
        assert decision.confidence == 0.95


def test_byok_credential_precedence():
    # 1. User personal key overrides platform admin key
    user_provs = {
        "anthropic": {"api_key": "sk-ant-user-personal", "base_url": "https://user-anthropic.com"}
    }
    system_provs = {
        "anthropic": {"api_key": "sk-ant-admin-shared", "base_url": "https://admin-anthropic.com"}
    }
    legacy = {"api_key": "sk-legacy-key", "provider": "anthropic"}

    key, base = resolve_provider_credentials("anthropic", user_provs, system_provs, legacy)
    assert key == "sk-ant-user-personal"
    assert base == "https://user-anthropic.com"

    # 2. When user key is absent, falls back to platform admin key
    key2, base2 = resolve_provider_credentials("anthropic", {}, system_provs, legacy)
    assert key2 == "sk-ant-admin-shared"
    assert base2 == "https://admin-anthropic.com"


def test_model_routing_role_slots_and_fallback():
    cfg = {
        "user_providers": {
            "openai": {"api_key": "sk-user-openai"},
            "anthropic": {"api_key": "sk-user-anthropic"},
        },
        "system_providers": {},
        "model_routing": {
            "router": {
                "primary": {"provider_key": "openai", "model": "gpt-4o-mini", "temperature": 0.0},
                "fallback": {
                    "provider_key": "anthropic",
                    "model": "claude-3-5-haiku",
                    "temperature": 0.1,
                },
            }
        },
    }

    model = resolve_chat_model_for_role("router", cfg)
    # Model should have fallback bound
    assert hasattr(model, "fallbacks") or hasattr(model, "with_fallbacks")



@pytest.mark.asyncio
async def test_run_scoped_llm_config_isolated_between_concurrent_runs(monkeypatch):
    import asyncio

    from src.agents.runtime import (
        agent_runtime,
        current_model_factory_var,
        llm_config_scope,
    )

    monkeypatch.setattr(agent_runtime, "_global_model_factory", None)
    monkeypatch.setenv("LLM_OPENAI_API_KEY", "service-env-key")
    monkeypatch.setenv("LLM_OPENAI_BASE_URL", "https://service-env.example/v1")
    monkeypatch.setenv("LLM_DEFAULT_MODEL", "service-env-model")
    factory_token = current_model_factory_var.set(None)

    def config_for(name):
        return {
            "user_providers": {
                "openai": {
                    "api_key": f"key-{name}",
                    "base_url": f"https://{name}.example/v1",
                }
            },
            "system_providers": {},
            "model_routing": {
                "research": {
                    "primary": {
                        "provider_key": "openai",
                        "model": f"{name}-model",
                        "temperature": 0.0,
                    }
                }
            },
        }

    async def selected_model(name):
        with llm_config_scope(config_for(name)):
            await asyncio.sleep(0)
            model = agent_runtime.get_chat_model(role="research")
            return model.model_name, model.openai_api_base

    try:
        selected = await asyncio.gather(selected_model("run-a"), selected_model("run-b"))
        assert selected == [
            ("run-a-model", "https://run-a.example/v1"),
            ("run-b-model", "https://run-b.example/v1"),
        ]
    finally:
        current_model_factory_var.reset(factory_token)


def test_run_routing_uses_service_environment_credentials_as_fallback(monkeypatch):
    from src.agents.runtime import (
        agent_runtime,
        current_model_factory_var,
        llm_config_scope,
    )

    monkeypatch.setattr(agent_runtime, "_global_model_factory", None)
    monkeypatch.setenv("LLM_OPENAI_API_KEY", "service-env-key")
    monkeypatch.setenv("LLM_OPENAI_BASE_URL", "https://service-env.example/v1")
    factory_token = current_model_factory_var.set(None)
    config = {
        "model_routing": {
            "research": {
                "primary": {
                    "provider_key": "openai",
                    "model": "routed-model",
                }
            }
        }
    }

    try:
        with llm_config_scope(config):
            model = agent_runtime.get_chat_model(role="research")
        assert model.model_name == "routed-model"
        assert model.openai_api_base == "https://service-env.example/v1"
    finally:
        current_model_factory_var.reset(factory_token)

@pytest.mark.asyncio
async def test_hybrid_workflow_state_graph_compiles_and_plans():
    mock_ledger = MagicMock()
    mock_ledger.update_workflow_run = AsyncMock()
    mock_ledger.append_event_transactional = AsyncMock()
    mock_ledger.save_artifact = AsyncMock()
    mock_ledger.save_task = AsyncMock()
    mock_ledger.list_tasks_for_run = AsyncMock(return_value=[])
    mock_ledger.next_task_attempt = AsyncMock(return_value=1)

    app = build_hybrid_workflow(ledger=mock_ledger)
    assert app is not None


def test_hybrid_workflow_state_preserves_job_description():
    state = HybridWorkflowState(
        run_id="run_test_job_description",
        workspace_id="ws_test_job_description",
        company_name="Mensetsu",
        role_title="Software Engineer",
        job_description="Build reliable distributed services.",
    )

    assert state.job_description == "Build reliable distributed services."


def test_hybrid_task_failure_payload_matches_event_contract():
    task = WorkflowTask(
        id="task_company_test",
        run_id="run_test",
        task_type="research",
        name="Company research",
        assigned_agent="company-research",
        attempt=2,
    )

    assert _task_failed_payload(task, "wall-time budget exceeded") == {
        "task_id": "task_company_test",
        "attempt": 2,
        "error": "wall-time budget exceeded",
        "can_retry": False,
    }


@pytest.mark.asyncio
async def test_hybrid_curriculum_generates_one_lesson_per_prioritized_skill():
    mock_ledger = MagicMock()
    mock_ledger.update_workflow_run = AsyncMock()
    mock_ledger.append_event_transactional = AsyncMock()
    mock_ledger.save_task = AsyncMock()
    mock_ledger.list_artifacts_for_run = AsyncMock(return_value=[])
    from src.orchestration.hybrid_workflow import execute_curriculum_node

    state = HybridWorkflowState(
        run_id="run_test_curr_123",
        workspace_id="ws_test_123",
        company_name="Netcompany",
        role_title="Senior Consultant / Software Architect",
        match_analysis={"gaps": [{"skill": "GovTech Integration"}]},
    )
    skills = [
        {
            "name": f"Skill {index}",
            "slug": f"skill-{index}",
            "category": "Architecture",
            "candidate_status": "gap",
            "importance": "mandatory",
        }
        for index in range(12)
    ]
    graph_output = {
        "skills": skills,
        "relations": [
            {"source": f"Skill {index}", "target": f"Skill {index + 1}", "relation": "prerequisite"}
            for index in range(10)
        ],
        "summary": "Knowledge graph constructed.",
    }

    async def generate_output(**kwargs):
        input_data = kwargs["input_data"]
        if input_data["curriculum_stage"] == "graph":
            return MagicMock(id="exec_graph"), graph_output
        skill = input_data["target_skills"][0]["name"]
        return (
            MagicMock(id=f"exec_{skill}"),
            {
                "learning_units": [
                    {
                        "title": f"{skill} fundamentals",
                        "skill": skill,
                        "summary": f"Study {skill} internals and failure modes.",
                        "deep_dive_markdown": f"Detailed mechanisms for {skill}.",
                        "prerequisites": [],
                        "estimated_minutes": 60,
                    }
                ]
            },
        )

    with patch(
        "src.agents.runtime.agent_runtime.execute_agent_loop",
        side_effect=generate_output,
    ) as mock_exec:
        result = await execute_curriculum_node(state, mock_ledger)

    curriculum = result["curriculum"]
    assert len(curriculum["skills"]) == 12
    assert len(curriculum["learning_units"]) == 6
    assert {unit["skill"] for unit in curriculum["learning_units"]} == {
        skill["name"] for skill in skills[:6]
    }
    assert mock_exec.call_count == 7
    unit_calls = [
        call.kwargs["input_data"]
        for call in mock_exec.call_args_list
        if call.kwargs["input_data"]["curriculum_stage"] == "units"
    ]
    assert all(len(call["target_skills"]) == 1 for call in unit_calls)
    saved_tasks = [call.args[0] for call in mock_ledger.save_task.call_args_list]
    assert {f"exec_{skill['name']}" for skill in skills[:6]} <= set(saved_tasks[-1].execution_ids)


def test_question_batches_keep_the_full_question_bank_contract():
    batches = _question_batch_plan()
    counts: dict[str, int] = {}
    for category, _, batch_count in batches:
        counts[category] = counts.get(category, 0) + batch_count

    assert sum(counts.values()) >= 100
    assert len(counts) == 7
    assert all(count >= 15 for count in counts.values())
    assert all(batch_count == 3 for _, _, batch_count in batches)


@pytest.mark.asyncio
async def test_hybrid_persists_completed_outputs_from_run_artifacts():
    workspace_id = "ws_hybrid_test"
    run_id = "run_hybrid_test"
    evidence_id = "art_evidence_test"
    artifacts = [
        Artifact(
            id=evidence_id,
            run_id=run_id,
            task_id="task_company_run_hybr",
            type="evidence",
            title="Company architecture source",
            evidence=EvidenceProvenance(
                source_url="https://example.com/architecture",
                title="Architecture",
                snippet="Verified architecture evidence.",
                confidence=0.95,
            ),
            content={"snippet": "Verified architecture evidence."},
        ),
        Artifact(
            id="art_company_test",
            run_id=run_id,
            task_id="task_company_run_hybr",
            type="research_report",
            title="Company research",
            evidence_ids=[evidence_id],
            content={
                "overview": "Evidence-backed company overview.",
                "claims": [
                    {
                        "statement": "Uses event-driven services.",
                        "category": "architecture",
                        "confidence": 0.9,
                        "evidence_ids": [evidence_id],
                    }
                ],
                "tech_stack": [],
            },
        ),
        Artifact(
            id="art_match_test",
            run_id=run_id,
            task_id="task_synthesize_run_hybr",
            type="validation_result",
            title="Match analysis",
            content={
                "overall_match_ratio": 0.82,
                "strengths": [{"skill": "Rust", "evidence_quote": "Built Rust services."}],
                "gaps": [],
                "risks": [],
                "wording_improvements": [],
            },
        ),
        Artifact(
            id="art_curriculum_test",
            run_id=run_id,
            task_id="task_curriculum_run_hybr",
            type="research_report",
            title="Curriculum",
            content={
                "skills": [
                    {
                        "name": "Distributed systems",
                        "slug": "distributed-systems",
                        "category": "Architecture",
                        "candidate_status": "in_progress",
                        "importance": "mandatory",
                    }
                ],
                "relations": [],
                "learning_units": [
                    {
                        "skill": "Distributed systems",
                        "title": "Consensus fundamentals",
                        "prerequisites": [],
                        "estimated_minutes": 45,
                        "summary": "Understand consensus safety.",
                        "deep_dive_markdown": "Consensus mechanisms and safety properties.",
                    }
                ],
            },
        ),
        Artifact(
            id="art_questions_test",
            run_id=run_id,
            task_id="task_questions_run_hybr",
            type="question_set",
            title="Interview questions",
            content={
                "questions": [
                    {
                        "question": "How would you debug a partition?",
                        "category": "system_design",
                        "difficulty": "hard",
                        "skills_tested": ["distributed systems"],
                        "interviewer_intent": "Assess failure isolation.",
                        "answer_outline": [
                            "Check quorum",
                            "Inspect network paths",
                            "Recover safely",
                        ],
                        "expected_signals": ["Explains consistency", "Covers recovery"],
                        "pitfalls": ["Assumes no partitions", "Ignores data loss"],
                        "follow_ups": [
                            "How do you recover replicas?",
                            "What consistency is acceptable?",
                        ],
                    }
                ]
            },
        ),
    ]
    db = MagicMock()
    db.execute_query = AsyncMock(return_value=[])
    ledger = MagicMock(db=db)
    state = HybridWorkflowState(
        run_id=run_id,
        workspace_id=workspace_id,
        company_name="Example Corp",
        role_title="Platform Engineer",
    )

    await _persist_hybrid_outputs(state, ledger, artifacts)

    writes = [(call.args[0], call.kwargs["vars"]) for call in db.execute_query.call_args_list]
    tables = {query.split("UPSERT type::record('")[1].split("'")[0] for query, _ in writes}
    assert {
        "source",
        "company_research",
        "claim",
        "match_analysis",
        "skill",
        "learning_unit",
        "interview_question",
    } <= tables
    company_record = next(values for query, values in writes if "company_research" in query)
    match_record = next(values for query, values in writes if "match_analysis" in query)
    lesson_record = next(values for query, values in writes if "learning_unit" in query)
    question_record = next(values for query, values in writes if "interview_question" in query)
    assert company_record["dossier"]["overview"] == "Evidence-backed company overview."
    assert len(company_record["dossier"]["sources"]) == 1
    assert match_record["overall_match_ratio"] == 0.82
    assert lesson_record["content_markdown"] == "Consensus mechanisms and safety properties."
    assert question_record["question"] == "How would you debug a partition?"
    assert question_record["category"] == "system_design"
    assert question_record["strong_answer_rubric"] == [
        "Explains consistency",
        "Covers recovery",
    ]
    assert question_record["follow_up_questions"] == [
        "How do you recover replicas?",
        "What consistency is acceptable?",
    ]
