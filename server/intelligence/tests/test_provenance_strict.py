from unittest.mock import AsyncMock, patch

import pytest

from src.agents.runtime import AutonomousAgentRuntime
from src.contracts.v1 import AgentExecution


@pytest.mark.asyncio
async def test_save_evidence_unfetched_and_fabricated_snippet():
    from src.contracts.v1 import WorkflowRun
    from src.orchestration.ledger import SurrealLedgerRepository

    ledger = SurrealLedgerRepository()
    await ledger.create_workflow_run(
        WorkflowRun(
            id="run_test_prov",
            workflow_id="interview-preparation-static",
            workflow_name="Provenance Test",
            workspace_id="ws_prov",
            user_id="usr_prov",
            type="static",
            status="running",
        )
    )
    runtime = AutonomousAgentRuntime(ledger=ledger)
    exec_rec = AgentExecution(
        id="exec_test_prov",
        run_id="run_test_prov",
        task_id="task_test_prov",
        agent_id="company-research",
        objective="Test provenance guardrails",
    )
    tools = runtime.create_tools_for_execution(
        exec_rec, "run_test_prov", depth=0, ancestors=["company-research"]
    )
    tool_map = {t.name: t for t in tools}

    res_unfetched = await tool_map["save_evidence"].ainvoke(
        {
            "source_url": "https://unfetched.com/about",
            "title": "Unfetched Title",
            "snippet": "Some snippet",
            "confidence": 0.9,
        }
    )
    assert "was never fetched by 'read_source'" in res_unfetched

    with patch(
        "src.agents.runtime.SafeResearchFetcher.fetch_clean_text", new_callable=AsyncMock
    ) as mock_fetch:
        mock_fetch.return_value = ("Blog", "Real fetched content about distributed consensus.")
        read_res = await tool_map["read_source"].ainvoke({"url": "https://www.python.org/blogs/"})
        assert "Title:" in read_res

        res_fabricated = await tool_map["save_evidence"].ainvoke(
            {
                "source_url": "https://www.python.org/blogs/",
                "title": "Blog",
                "snippet": "Completely fabricated claim that does not exist in the fetched content.",
                "confidence": 0.9,
            }
        )
        assert "Snippet does not match text from" in res_fabricated
        assert "must be a verbatim excerpt" in res_fabricated

        res_valid = await tool_map["save_evidence"].ainvoke(
            {
                "source_url": "https://www.python.org/blogs/",
                "title": "Blog",
                "snippet": "Real fetched content about distributed consensus.",
                "confidence": 0.95,
            }
        )
        assert "Evidence saved with ID:" in res_valid
