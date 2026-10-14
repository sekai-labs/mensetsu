import pytest

from src.agents.registry import AgentDefinition, AgentRegistry


def test_registry_strict_validation():

    registry = AgentRegistry()
    with pytest.raises(KeyError):
        registry.get("nonexistent-agent-id")

    agents = registry.list_all()
    assert len(agents) >= 6

    for a in agents:
        assert isinstance(a, AgentDefinition)
        assert len(a.instructions.strip()) > 0
        assert len(a.allowed_tools) > 0
        for tool_name in a.allowed_tools:
            assert tool_name in (
                "web_search",
                "read_source",
                "save_evidence",
                "save_finding",
                "update_notes",
                "delegate_task",
                "evaluate_findings",
            )
