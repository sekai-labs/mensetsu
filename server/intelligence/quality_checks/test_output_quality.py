"""Offline determinism checks for workflow output quality gates."""

from types import SimpleNamespace
from unittest.mock import AsyncMock, MagicMock, patch

import pytest

from src.contracts.v1 import Artifact, EvidenceProvenance
from src.evals import OutputQualityError, validate_run_outputs
from src.orchestration.hybrid_outputs import build_hybrid_output_nodes
from src.orchestration.hybrid_support import HybridWorkflowState


def _evidence(evidence_id: str = "ev_1", **overrides) -> Artifact:
    provenance = {
        "source_url": f"https://example.com/{evidence_id}",
        "title": f"Evidence {evidence_id}",
        "snippet": "Verbatim supporting passage.",
        "confidence": 0.9,
        **overrides,
    }
    return Artifact(
        id=evidence_id,
        run_id="run_quality",
        type="evidence",
        title=f"Evidence {evidence_id}",
        evidence=EvidenceProvenance(**provenance),
        content={"snippet": provenance["snippet"]},
    )


def _company(evidence_ids: list[str]) -> dict:
    return {
        "overview": "Evidence-backed company overview.",
        "claims": [
            {
                "statement": "The company operates a distributed platform.",
                "category": "architecture",
                "confidence": 0.9,
                "evidence_ids": evidence_ids,
            }
        ],
        "tech_stack": [],
    }


def _valid_payload(**overrides) -> dict:
    payload = {
        "company": _company(["ev_1"]),
        "evidence_artifacts": [_evidence()],
        "questions": [
            {"question": "How does quorum work?", "skills_tested": ["Rust"]},
            {"question": "Why is backpressure needed?", "skills_tested": ["rust"]},
        ],
        "skills": [{"name": "Rust", "slug": "rust"}],
        "relations": [{"source": "Rust", "target": "Rust", "relation": "complements"}],
    }
    payload.update(overrides)
    return payload


def test_valid_cited_dossier_passes_all_metrics():
    report = validate_run_outputs(**_valid_payload())

    assert report.passed is True
    assert report.failures == ()
    assert {metric.metric_name for metric in report} == {
        "citation_provenance",
        "question_quality",
        "skill_reference_integrity",
        "prerequisite_graph_acyclic",
    }


def test_missing_and_unknown_evidence_rejects_dossier():
    missing = validate_run_outputs(**_valid_payload(company=_company([])))
    unknown = validate_run_outputs(**_valid_payload(company=_company(["ev_missing"])))

    assert missing.passed is False
    assert unknown.passed is False
    assert "no evidence_ids" in missing.failures[0].reason
    assert "unknown evidence ev_missing" in unknown.failures[0].reason


def test_missing_provenance_rejects_referenced_evidence():
    evidence = Artifact(
        id="ev_1",
        run_id="run_quality",
        type="evidence",
        title="No provenance",
        content={"snippet": "text"},
    )
    report = validate_run_outputs(**_valid_payload(evidence_artifacts=[evidence]))

    assert report.passed is False
    assert "has no provenance record" in report.failures[0].reason


def test_normalized_duplicate_and_empty_questions_reject():
    duplicate = validate_run_outputs(
        **_valid_payload(
            questions=[
                {"question": "How does  Quorum work?", "skills_tested": ["Rust"]},
                {"question": "how does quorum WORK?", "skills_tested": ["Rust"]},
            ]
        )
    )
    empty = validate_run_outputs(
        **_valid_payload(questions=[{"question": "   ", "skills_tested": ["Rust"]}])
    )

    assert duplicate.passed is False
    assert "duplicates question 1" in duplicate.failures[0].reason
    assert empty.passed is False
    assert "has empty text" in empty.failures[0].reason


def test_unknown_skill_references_reject():
    relation = validate_run_outputs(
        **_valid_payload(
            relations=[{"source": "Rust", "target": "Kubernetes", "relation": "prerequisite"}]
        )
    )
    question = validate_run_outputs(
        **_valid_payload(
            questions=[{"question": "Design a queue.", "skills_tested": ["Kafka"]}]
        )
    )

    assert relation.passed is False
    assert "unknown skill 'Kubernetes'" in relation.failures[0].reason
    assert question.passed is False
    assert "unknown skill 'Kafka'" in question.failures[0].reason


def test_prerequisite_cycle_rejects_but_complements_cycle_is_accepted():
    cycle = validate_run_outputs(
        **_valid_payload(
            relations=[
                {"source": "Rust", "target": "Kubernetes", "relation": "prerequisite"},
                {"source": "Kubernetes", "target": "Rust", "relation": "builds_on"},
            ],
            skills=[{"name": "Rust"}, {"name": "Kubernetes"}],
        )
    )
    complements = validate_run_outputs(
        **_valid_payload(
            relations=[
                {"source": "Rust", "target": "Kubernetes", "relation": "complements"},
                {"source": "Kubernetes", "target": "Rust", "relation": "complements"},
            ],
            skills=[{"name": "Rust"}, {"name": "Kubernetes"}],
        )
    )

    assert cycle.passed is False
    assert cycle.failures[0].metric_name == "prerequisite_graph_acyclic"
    assert complements.passed is True


def test_missing_sections_require_recorded_degradation_reason():
    absent = {
        "company": None,
        "evidence_artifacts": [],
        "questions": [],
        "skills": [],
        "relations": [],
    }
    unexplained = validate_run_outputs(**absent)

    assert {metric.metric_name for metric in unexplained.failures} == {
        "citation_provenance",
        "question_quality",
        "skill_reference_integrity",
    }

    company_only = validate_run_outputs(
        **absent,
        missing_research={"company": "company research task failed"},
    )
    assert {metric.metric_name for metric in company_only.failures} == {
        "question_quality",
        "skill_reference_integrity",
    }

    degraded = validate_run_outputs(
        **absent,
        missing_research={"company": "company research task failed"},
        missing_outputs={
            "interview question bank": "question generation task failed",
            "learning plan": "curriculum task failed",
        },
    )
    assert degraded.passed is True
    scores = {metric.metric_name: metric.score for metric in degraded}
    assert scores["citation_provenance"] == 0.0
    assert scores["question_quality"] == 0.0
    assert scores["skill_reference_integrity"] == 0.0
    assert scores["prerequisite_graph_acyclic"] == 1.0

    unrelated_reasons = validate_run_outputs(
        **absent,
        missing_research={"tech": "technical research task failed"},
        missing_outputs={"match analysis": "synthesis task failed"},
    )
    assert {metric.metric_name for metric in unrelated_reasons.failures} == {
        "citation_provenance",
        "question_quality",
        "skill_reference_integrity",
    }


@pytest.mark.asyncio
async def test_workflow_rejects_invalid_present_outputs_before_completion():
    company_artifact = Artifact(
        id="art_company_quality",
        run_id="run_quality",
        task_id="task_company_run_qual",
        type="research_report",
        title="Company research",
        content=_company(["ev_missing"]),
    )
    ledger = MagicMock()
    ledger.list_tasks_for_run = AsyncMock(return_value=[])
    ledger.get_workflow_run = AsyncMock(
        return_value=SimpleNamespace(status="running", started_at=None)
    )
    ledger.list_artifacts_for_run = AsyncMock(return_value=[company_artifact])
    ledger.transition_workflow_run_status = AsyncMock()
    nodes = build_hybrid_output_nodes(ledger, AsyncMock())
    state = HybridWorkflowState(
        run_id="run_quality",
        workspace_id="ws_quality",
        company_name="Quality Corp",
        role_title="Staff Engineer",
    )

    with patch(
        "src.orchestration.hybrid_outputs._persist_hybrid_outputs", new_callable=AsyncMock
    ) as persist:
        with pytest.raises(OutputQualityError):
            await nodes["finalize_node"](state)

    persist.assert_not_awaited()
    ledger.transition_workflow_run_status.assert_not_awaited()
