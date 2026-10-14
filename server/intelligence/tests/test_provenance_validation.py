import pytest

from src.agents.schemas import (
    CompanyClaim,
    CompanyResearchOutput,
    current_saved_evidence_ids_var,
)


def test_evidence_id_validation_catches_unknown_provenance():

    current_saved_evidence_ids_var.set({f"art_ev_{i}" for i in range(1, 5)})

    valid_claims = [
        CompanyClaim(
            statement=f"Acme uses Rust {i}",
            category="architecture",
            confidence=0.95,
            evidence_ids=[f"art_ev_{k}" for k in range(1, 5)],
        )
        for i in range(8)
    ]
    output = CompanyResearchOutput(
        overview="Acme Overview",
        business_model="SaaS",
        engineering_culture=["Fast"],
        tech_stack=[],
        architecture_notes=[],
        interview_process_notes=[],
        claims=valid_claims,
        summary="Acme summary",
    )
    assert len(output.claims) == 8

    invalid_claims = [
        CompanyClaim(
            statement=f"Acme uses Go {i}",
            category="architecture",
            confidence=0.9,
            evidence_ids=["art_nonexistent_999"] if i == 0 else ["art_ev_123"],
        )
        for i in range(8)
    ]
    with pytest.raises(ValueError) as exc:
        CompanyResearchOutput(
            overview="Acme Overview",
            business_model="SaaS",
            engineering_culture=["Fast"],
            tech_stack=[],
            architecture_notes=[],
            interview_process_notes=[],
            claims=invalid_claims,
            summary="Acme summary",
        )
    assert "art_nonexistent_999" in str(exc.value)
    assert "unknown or unsaved evidence IDs" in str(exc.value)
