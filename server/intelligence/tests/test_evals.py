from src.evals import ResearchEvaluationHarness


def test_citation_provenance_validation():
    sources = [{"id": "src_1", "url": "https://example.com"}]
    valid_claims = [{"statement": "Uses Rust", "source_id": "src_1"}]
    res = ResearchEvaluationHarness.evaluate_citation_provenance(sources, valid_claims)
    assert res.passed is True
    assert res.score == 1.0

    invalid_claims = [{"statement": "Secret project", "source_id": "src_nonexistent"}]
    res_invalid = ResearchEvaluationHarness.evaluate_citation_provenance(sources, invalid_claims)
    assert res_invalid.passed is False


def test_prerequisite_cycle_detection():
    valid_edges = [
        {"source": "A", "target": "B"},
        {"source": "B", "target": "C"},
    ]
    res_valid = ResearchEvaluationHarness.evaluate_prerequisite_graph_acyclic(valid_edges)
    assert res_valid.passed is True

    cycle_edges = [
        {"source": "A", "target": "B"},
        {"source": "B", "target": "A"},
    ]
    res_cycle = ResearchEvaluationHarness.evaluate_prerequisite_graph_acyclic(cycle_edges)
    assert res_cycle.passed is False
