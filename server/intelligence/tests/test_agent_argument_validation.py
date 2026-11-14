import pytest

from src.agents.agent_tools import parse_confidence


@pytest.mark.parametrize("value", [None, True, "invalid", float("nan"), float("inf"), -0.1, 1.1])
def test_parse_confidence_returns_model_facing_error_for_invalid_values(value):
    result = parse_confidence(value, "save_finding")

    assert isinstance(result, str)
    assert "finite number between 0.0 and 1.0" in result


@pytest.mark.parametrize(("value", "expected"), [(0, 0.0), (0.75, 0.75), ("0.25", 0.25)])
def test_parse_confidence_accepts_finite_values_in_range(value, expected):
    assert parse_confidence(value, "save_evidence") == expected
