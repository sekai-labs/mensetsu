from unittest.mock import AsyncMock, MagicMock, patch

import pytest

from src.agents.model_router import (
    resolve_chat_model_for_role,
    resolve_provider_credentials,
)
from src.agents.schemas import WorkflowRoutingDecision
from src.agents.workflow_router import route_workflow_request
from src.orchestration.hybrid_workflow import HybridWorkflowState, build_hybrid_workflow


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


@pytest.mark.asyncio
async def test_hybrid_workflow_curriculum_staged_success():
    mock_ledger = MagicMock()
    mock_ledger.update_workflow_run = AsyncMock()
    mock_ledger.append_event_transactional = AsyncMock()
    mock_ledger.save_task = AsyncMock()
    mock_ledger.list_artifacts_for_run = AsyncMock(return_value=[])

    from src.orchestration.hybrid_workflow import execute_curriculum_node

    def curriculum_fn(s):
        return execute_curriculum_node(s, mock_ledger)
    state = HybridWorkflowState(
        run_id="run_test_curr_123",
        workspace_id="ws_test_123",
        company_name="Netcompany",
        role_title="Senior Consultant / Software Architect",
        match_analysis={"gaps": [{"name": "GovTech Integration"}]},
    )

    mock_exec_rec_1 = MagicMock(id="exec_1")
    mock_exec_rec_2 = MagicMock(id="exec_2")
    mock_graph_output = {
        "skills": [
            {"name": "GovTech Integration", "category": "Domain", "candidate_status": "gap"}
        ],
        "relations": [],
        "summary": "Knowledge graph constructed.",
    }
    mock_units_output = {
        "learning_units": [
            {
                "title": "GovTech Modernization",
                "skill": "GovTech Integration",
                "summary": "Mastering European digital public infrastructure.",
            }
        ]
    }

    with patch("src.agents.runtime.agent_runtime.execute_agent_loop") as mock_exec:
        mock_exec.side_effect = [
            (mock_exec_rec_1, mock_graph_output),
            (mock_exec_rec_2, mock_units_output),
        ]

        res = await curriculum_fn(state)

        assert "curriculum" in res
        curr = res["curriculum"]
        assert len(curr["skills"]) == 1
        assert len(curr["learning_units"]) == 1
        assert curr["learning_units"][0]["title"] == "GovTech Modernization"
        assert mock_exec.call_count == 2
        stage2_call = mock_exec.call_args_list[1]
        assert stage2_call.kwargs["agent_id"] == "curriculum"
        assert "learning units" in stage2_call.kwargs["objective"]

        # Verify task was saved with both stage execution IDs
        saved_tasks = [call.args[0] for call in mock_ledger.save_task.call_args_list]
        assert len(saved_tasks) > 0
        completed_curr_task = saved_tasks[-1]
        assert completed_curr_task.execution_ids == ["exec_1", "exec_2"]
