from unittest.mock import AsyncMock, MagicMock, patch

import pytest

from src.agents.model_router import (
    resolve_chat_model_for_role,
    resolve_provider_credentials,
)
from src.agents.schemas import WorkflowRoutingDecision
from src.agents.workflow_router import route_workflow_request
from src.contracts.v1 import Artifact, EvidenceProvenance, WorkflowTask
from src.orchestration.hybrid_support import (
    _persist_hybrid_outputs,
    _question_batch_plan,
    _task_failed_payload,
)
from src.orchestration.hybrid_workflow import (
    HybridWorkflowState,
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
    mock_ledger.next_task_attempt = AsyncMock(return_value=1)
    from src.orchestration.hybrid_curriculum import execute_curriculum_node

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


def test_curriculum_graph_rejects_relations_to_unknown_skill_names():
    from pydantic import ValidationError

    from src.agents.schemas import CurriculumGraphOutput

    skills = [
        {
            "name": f"Skill {index}",
            "slug": f"skill-{index}",
            "category": "Systems",
        }
        for index in range(12)
    ]
    relations = [
        {"source": f"Skill {index}", "target": f"Skill {index + 1}"}
        for index in range(10)
    ]
    relations[-1]["target"] = "Model Provider Abstration and Routing"

    with pytest.raises(ValidationError, match="unknown references"):
        CurriculumGraphOutput(
            skills=skills,
            relations=relations,
            summary="Test graph",
        )


def test_question_skill_references_must_match_curriculum_names():
    from pydantic import ValidationError

    from src.agents.schemas import (
        InterviewQuestionItem,
        current_knowledge_graph_skill_names_var,
    )

    skill_name = "Model Provider Abstraction and Routing"
    token = current_knowledge_graph_skill_names_var.set(frozenset({skill_name}))
    question = {
        "question": "How would you route model requests?",
        "category": "system_design",
        "difficulty": "medium",
        "rationale": "Test model routing.",
        "skills_tested": ["Model Provider Abstration and Routing"],
        "interviewer_intent": "Assess routing design.",
        "expected_signals": ["Explains routing", "Handles provider failures"],
        "answer_outline": ["Define provider policy", "Route requests", "Measure failures"],
        "pitfalls": ["Hard-code providers", "Ignore retries"],
        "follow_ups": ["How would you fail over?", "How would you add a provider?"],
        "evidence_ids": ["evidence-1"],
    }

    try:
        with pytest.raises(ValidationError, match="must exactly match"):
            InterviewQuestionItem(**question)

        question["skills_tested"] = [skill_name]
        parsed = InterviewQuestionItem(**question)
        assert parsed.skills_tested == [skill_name]
    finally:
        current_knowledge_graph_skill_names_var.reset(token)


def _replan_ledger(next_attempt: int = 1) -> MagicMock:
    ledger = MagicMock()
    ledger.next_task_attempt = AsyncMock(return_value=next_attempt)
    ledger.save_task = AsyncMock()
    ledger.append_event_transactional = AsyncMock()
    ledger.update_workflow_run = AsyncMock()
    ledger.list_tasks_for_run = AsyncMock(return_value=[])
    ledger.list_artifacts_for_run = AsyncMock(return_value=[])
    ledger.get_workflow_run = AsyncMock(return_value=None)
    return ledger


def _execution(execution_id: str, summary: str = "Recovered output") -> MagicMock:
    execution = MagicMock()
    execution.id = execution_id
    execution.public_summary = summary
    return execution


def _dynamic_task_inputs() -> dict:
    return {
        "task_info": {
            "id": "task_company_run_hybr",
            "assigned_agent": "company-research",
            "objective": "Research Acme engineering culture",
            "dependencies": [],
        },
        "run_id": "run_hybr",
        "workspace_id": "ws_hybr",
        "company_name": "Acme",
        "role_title": "Staff Engineer",
        "company_urls": [],
        "parsed_resume": {},
        "job_description": "Build reliable distributed services.",
        "user_prompt": "",
    }


def _graph_output() -> dict:
    return {
        "skills": [
            {
                "name": f"Skill {index}",
                "slug": f"skill-{index}",
                "category": "Architecture",
                "candidate_status": "gap",
                "importance": "mandatory",
            }
            for index in range(6)
        ],
        "relations": [],
        "summary": "Graph rebuilt with a different decomposition.",
    }


@pytest.mark.asyncio
async def test_dynamic_task_replans_with_prior_error_on_second_attempt():
    from src.orchestration.hybrid_nodes import build_hybrid_nodes

    ledger = _replan_ledger(next_attempt=2)
    node = build_hybrid_nodes(checkpointer=None, ledger=ledger)["execute_dynamic_task_node"]
    calls: list[dict] = []

    async def execute_agent_loop(**kwargs):
        calls.append(kwargs)
        if len(calls) == 1:
            raise RuntimeError("agent loop exhausted its tool budget")
        return _execution("exec_recovered", "Recovered research"), {
            "summary": "Recovered research"
        }

    with patch(
        "src.agents.runtime.agent_runtime.execute_agent_loop",
        side_effect=execute_agent_loop,
    ), patch("asyncio.sleep", new_callable=AsyncMock):
        result = await node(_dynamic_task_inputs())

    assert result == {
        "completed_task_ids": ["task_company_run_hybr"],
        "research_results": {"company": {"summary": "Recovered research"}},
    }
    assert len(calls) == 2
    assert calls[0]["recovery_context"] is None
    assert calls[0]["objective"] == calls[1]["objective"]
    assert calls[1]["input_data"] == calls[0]["input_data"]
    recovery = calls[1]["recovery_context"]
    assert recovery is not None
    assert recovery.attempt == 1
    assert "tool budget" in recovery.previous_error
    saved_tasks = [call.args[0] for call in ledger.save_task.call_args_list]
    assert saved_tasks[-1].attempt == 3
    assert saved_tasks[-1].status == "completed"
    started_events = [
        call.kwargs["payload"]
        for call in ledger.append_event_transactional.call_args_list
        if call.kwargs.get("event_type") == "task.started"
    ]
    assert started_events == [
        {
            "task_id": "task_company_run_hybr",
            "attempt": 2,
            "assigned_agent": "company-research",
        }
    ]
    retry_events = [
        call.kwargs
        for call in ledger.append_event_transactional.call_args_list
        if call.kwargs.get("event_type") == "task.retrying"
    ]
    assert len(retry_events) == 1
    assert retry_events[0]["payload"] == {
        "task_id": "task_company_run_hybr",
        "next_attempt": 3,
        "delay_ms": 1000,
    }


@pytest.mark.asyncio
async def test_dynamic_task_keeps_missing_research_fallback_after_replans_exhaust():
    from src.orchestration.hybrid_nodes import build_hybrid_nodes

    ledger = _replan_ledger()
    node = build_hybrid_nodes(checkpointer=None, ledger=ledger)["execute_dynamic_task_node"]
    calls: list[dict] = []

    async def execute_agent_loop(**kwargs):
        calls.append(kwargs)
        raise RuntimeError("provider repeatedly unavailable")

    with patch(
        "src.agents.runtime.agent_runtime.execute_agent_loop",
        side_effect=execute_agent_loop,
    ), patch("asyncio.sleep", new_callable=AsyncMock):
        result = await node(_dynamic_task_inputs())

    assert result == {
        "failed_task_ids": ["task_company_run_hybr"],
        "missing_research": {"company": "provider repeatedly unavailable"},
    }
    assert len(calls) == 2
    assert calls[0]["recovery_context"] is None
    assert calls[1]["recovery_context"] is not None
    saved_tasks = [call.args[0] for call in ledger.save_task.call_args_list]
    assert saved_tasks[-1].attempt == 2
    assert saved_tasks[-1].status == "failed"
    failed_events = [
        call.kwargs["payload"]
        for call in ledger.append_event_transactional.call_args_list
        if call.kwargs.get("event_type") == "task.failed"
    ]
    assert failed_events[-1]["attempt"] == 2
    assert failed_events[-1]["can_retry"] is False


@pytest.mark.asyncio
async def test_synthesis_replans_with_prior_error_on_second_attempt():
    from src.orchestration.hybrid_nodes import build_hybrid_nodes

    ledger = _replan_ledger()
    node = build_hybrid_nodes(checkpointer=None, ledger=ledger)["synthesize_node"]
    state = HybridWorkflowState(
        run_id="run_hybr_synthesis",
        workspace_id="ws_hybr_synthesis",
        company_name="Acme",
        role_title="Staff Engineer",
        tasks=[{"id": "task_company_run_hybr"}],
    )
    calls: list[dict] = []

    async def execute_agent_loop(**kwargs):
        calls.append(kwargs)
        if len(calls) == 1:
            raise RuntimeError("match schema validation failed")
        return _execution("exec_eval_recovered"), {
            "overall_match_ratio": 0.74,
            "gaps": [],
        }

    with patch(
        "src.agents.runtime.agent_runtime.execute_agent_loop",
        side_effect=execute_agent_loop,
    ), patch("asyncio.sleep", new_callable=AsyncMock):
        result = await node(state)

    assert result == {
        "match_analysis": {"overall_match_ratio": 0.74, "gaps": []},
        "completed_task_ids": ["task_synthesize_run_hybr"],
    }
    assert [call["agent_id"] for call in calls] == ["evidence-evaluator", "evidence-evaluator"]
    assert calls[0]["recovery_context"] is None
    recovery = calls[1]["recovery_context"]
    assert recovery is not None
    assert recovery.attempt == 1
    assert "schema validation" in recovery.previous_error
    saved_tasks = [call.args[0] for call in ledger.save_task.call_args_list]
    assert saved_tasks[-1].attempt == 2
    assert saved_tasks[-1].status == "completed"


@pytest.mark.asyncio
async def test_curriculum_graph_generation_replans_before_units():
    from src.orchestration.hybrid_curriculum import execute_curriculum_node

    ledger = _replan_ledger()
    state = HybridWorkflowState(
        run_id="run_hybr_curriculum",
        workspace_id="ws_hybr_curriculum",
        company_name="Acme",
        role_title="Staff Engineer",
    )
    graph_calls: list[dict] = []
    all_calls: list[dict] = []

    async def execute_agent_loop(**kwargs):
        all_calls.append(kwargs)
        input_data = kwargs["input_data"]
        if input_data["curriculum_stage"] == "graph":
            graph_calls.append(kwargs)
            if len(graph_calls) == 1:
                raise RuntimeError("graph schema rejected unknown relations")
            return _execution("exec_graph_recovered"), _graph_output()
        skill = input_data["target_skills"][0]["name"]
        return _execution(f"exec_{skill}"), {
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
        }

    with patch(
        "src.agents.runtime.agent_runtime.execute_agent_loop",
        side_effect=execute_agent_loop,
    ), patch("asyncio.sleep", new_callable=AsyncMock):
        result = await execute_curriculum_node(state, ledger)

    assert len(graph_calls) == 2
    assert graph_calls[0]["recovery_context"] is None
    recovery = graph_calls[1]["recovery_context"]
    assert recovery is not None
    assert recovery.attempt == 1
    assert "unknown relations" in recovery.previous_error
    unit_calls = [
        call for call in all_calls if call["input_data"]["curriculum_stage"] != "graph"
    ]
    assert unit_calls
    assert all(call["recovery_context"] is None for call in unit_calls)
    assert len(result["curriculum"]["learning_units"]) == 6
    assert result["completed_task_ids"] == ["task_curriculum_run_hybr"]
    saved_tasks = [call.args[0] for call in ledger.save_task.call_args_list]
    assert saved_tasks[-1].attempt == 2
    retry_events = [
        call.kwargs
        for call in ledger.append_event_transactional.call_args_list
        if call.kwargs.get("event_type") == "task.retrying"
    ]
    assert len(retry_events) == 1
    assert retry_events[0]["payload"]["next_attempt"] == 2


@pytest.mark.asyncio
async def test_learning_unit_generation_replans_with_prior_error():
    from src.orchestration.hybrid_curriculum import execute_curriculum_node

    ledger = _replan_ledger()
    state = HybridWorkflowState(
        run_id="run_hybr_units",
        workspace_id="ws_hybr_units",
        company_name="Acme",
        role_title="Staff Engineer",
    )
    unit_calls: list[dict] = []

    async def execute_agent_loop(**kwargs):
        input_data = kwargs["input_data"]
        if input_data["curriculum_stage"] == "graph":
            return _execution("exec_graph"), _graph_output()
        skill = input_data["target_skills"][0]["name"]
        unit_calls.append(kwargs)
        if skill == "Skill 0" and len(
            [call for call in unit_calls if call["input_data"]["target_skills"][0]["name"] == skill]
        ) == 1:
            raise RuntimeError("learning unit omitted the target skill")
        return _execution(f"exec_{skill}"), {
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
        }

    with patch(
        "src.agents.runtime.agent_runtime.execute_agent_loop",
        side_effect=execute_agent_loop,
    ):
        result = await execute_curriculum_node(state, ledger)

    skill_zero_calls = [
        call for call in unit_calls if call["input_data"]["target_skills"][0]["name"] == "Skill 0"
    ]
    assert len(skill_zero_calls) == 2
    assert skill_zero_calls[0]["recovery_context"] is None
    recovery = skill_zero_calls[1]["recovery_context"]
    assert recovery is not None
    assert recovery.attempt == 1
    assert "target skill" in recovery.previous_error
    assert len(result["curriculum"]["learning_units"]) == 6


@pytest.mark.asyncio
async def test_interview_batches_use_second_attempt_as_replan():
    from src.orchestration.hybrid_outputs import build_hybrid_output_nodes

    ledger = _replan_ledger()
    node = build_hybrid_output_nodes(ledger, AsyncMock())["interview_node"]
    state = HybridWorkflowState(
        run_id="run_hybr_questions",
        workspace_id="ws_hybr_questions",
        company_name="Acme",
        role_title="Staff Engineer",
        curriculum={"skills": [{"name": "Distributed systems"}]},
    )
    calls: list[dict] = []
    attempts: dict[tuple[str, int], int] = {}

    async def execute_agent_loop(**kwargs):
        calls.append(kwargs)
        input_data = kwargs["input_data"]
        key = (input_data["target_category"], input_data["batch_part"])
        attempts[key] = attempts.get(key, 0) + 1
        if attempts[key] == 1:
            raise RuntimeError(f"forced batch failure {key[0]} {key[1]}")
        questions = [
            {
                "question": f"{key[0]} batch {key[1]} question {index}",
                "category": key[0],
                "difficulty": "medium",
                "skills_tested": ["Distributed systems"],
                "interviewer_intent": "Assess distributed systems reasoning.",
                "answer_outline": ["Define constraints", "Explain tradeoffs", "Describe recovery"],
                "expected_signals": ["Mentions quorum", "Covers failure isolation"],
                "pitfalls": ["Ignores partitions", "Assumes no data loss"],
                "follow_ups": ["How do you recover?", "What consistency is required?"],
            }
            for index in range(input_data["count"])
        ]
        return _execution(f"exec_{key[0]}_{key[1]}"), {"questions": questions}

    with patch(
        "src.agents.runtime.agent_runtime.execute_agent_loop",
        side_effect=execute_agent_loop,
    ):
        result = await node(state)

    assert len(calls) == 2 * len(_question_batch_plan())
    assert len(result["questions"]) == 105
    grouped: dict[tuple[str, int], list[dict]] = {}
    for call in calls:
        input_data = call["input_data"]
        grouped.setdefault(
            (input_data["target_category"], input_data["batch_part"]), []
        ).append(call)
    assert all(len(group) == 2 for group in grouped.values())
    for group in grouped.values():
        assert group[0]["recovery_context"] is None
        recovery = group[1]["recovery_context"]
        assert recovery is not None
        assert recovery.attempt == 1
        assert "forced batch failure" in recovery.previous_error
    saved_tasks = [call.args[0] for call in ledger.save_task.call_args_list]
    assert saved_tasks[-1].status == "completed"
    assert saved_tasks[-1].attempt == 1


@pytest.mark.asyncio
async def test_hybrid_finalize_enforces_output_contract_invariants():
    from src.evals import OutputQualityError
    from src.orchestration.hybrid_outputs import build_hybrid_output_nodes

    ledger = _replan_ledger()
    nodes = build_hybrid_output_nodes(ledger, AsyncMock())
    state = HybridWorkflowState(
        run_id="run_hybr_contract_check",
        workspace_id="ws_hybr_contract_check",
        company_name="Acme",
        role_title="Staff Engineer",
        research_results={
            "company": {
                "overview": "Overview",
                "claims": [
                    {
                        "statement": "Claim citing missing evidence",
                        "evidence_ids": ["ev_missing_from_run"],
                    }
                ],
            }
        },
    )

    with patch(
        "src.orchestration.hybrid_outputs._persist_hybrid_outputs",
        new_callable=AsyncMock,
    ) as persist:
        with pytest.raises(OutputQualityError) as exc_info:
            await nodes["finalize_node"](state)

    persist.assert_not_awaited()
    assert "citation_provenance" in str(exc_info.value)
    assert "ev_missing_from_run" in str(exc_info.value)
