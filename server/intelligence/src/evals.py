"""Deterministic output-quality metrics for workflow run deliverables.

The gate is intentionally side-effect free: callers pass the exact outputs they
are about to persist and receive a metric report.  A workflow raises only when a
present output is internally inconsistent; missing research or generation output
is reported as skipped so degraded runs keep their explicit degraded semantics.
"""

from __future__ import annotations

from collections.abc import Iterator, Mapping, Sequence
from dataclasses import dataclass
from typing import Any

COMPLEMENTS_RELATION = "complements"


@dataclass(frozen=True, slots=True)
class EvalMetricResult:
    """One deterministic quality metric."""

    metric_name: str
    passed: bool
    score: float
    reason: str


class OutputQualityError(ValueError):
    """Raised when a workflow attempts to persist invalid or unrecorded-missing outputs."""


@dataclass(frozen=True, slots=True)
class RunOutputValidation:
    """Metric report returned by :func:`validate_run_outputs`."""

    metrics: tuple[EvalMetricResult, ...]

    @property
    def passed(self) -> bool:
        return all(metric.passed for metric in self.metrics)

    @property
    def failures(self) -> tuple[EvalMetricResult, ...]:
        return tuple(metric for metric in self.metrics if not metric.passed)

    def raise_if_invalid(self, *, context: str = "workflow run") -> None:
        if self.passed:
            return
        details = "; ".join(
            f"{metric.metric_name}: {metric.reason}" for metric in self.failures
        )
        raise OutputQualityError(f"{context} failed output quality validation: {details}")

    def __iter__(self) -> Iterator[EvalMetricResult]:
        return iter(self.metrics)

    def __len__(self) -> int:
        return len(self.metrics)

    def __getitem__(self, index: int) -> EvalMetricResult:
        return self.metrics[index]


def _field(value: Any, name: str, default: Any = None) -> Any:
    if isinstance(value, Mapping):
        return value.get(name, default)
    return getattr(value, name, default)


def _mapping(value: Any) -> dict[str, Any]:
    if isinstance(value, Mapping):
        return dict(value)
    if hasattr(value, "model_dump"):
        dumped = value.model_dump()
        return dumped if isinstance(dumped, dict) else {}
    return {}


def _missing_reason(reasons: Mapping[str, Any] | None, key: str) -> str | None:
    if not reasons:
        return None
    reason = reasons.get(key)
    return str(reason).strip() if reason is not None and str(reason).strip() else None


def _iter_items(value: Any) -> list[Any]:
    return list(value or [])


def _normalise_text(value: Any) -> str:
    return " ".join(str(value or "").casefold().split())


def _normalise_skill(value: Any) -> str:
    return _normalise_text(str(value or "").replace("-", " ").replace("_", " "))


def _skill_keys(skills: Sequence[Any]) -> set[str]:
    keys: set[str] = set()
    for skill in skills:
        for field_name in ("name", "slug"):
            key = _normalise_skill(_field(skill, field_name))
            if key:
                keys.add(key)
    return keys


def _provenance_problem(artifact: Any) -> str | None:
    artifact_id = str(_field(artifact, "id") or "<unknown>")
    provenance = _field(artifact, "evidence")
    if provenance is None:
        return f"evidence {artifact_id} has no provenance record"
    data = _mapping(provenance)
    missing = [
        field
        for field in ("source_url", "title", "snippet")
        if not str(data.get(field) or "").strip()
    ]
    if missing:
        return f"evidence {artifact_id} provenance is missing {', '.join(missing)}"
    confidence = data.get("confidence")
    try:
        numeric_confidence = float(confidence)
    except (TypeError, ValueError):
        return f"evidence {artifact_id} provenance confidence is invalid"
    if not 0.0 <= numeric_confidence <= 1.0:
        return f"evidence {artifact_id} provenance confidence is outside [0, 1]"
    return None


def _citation_metric(
    company: Any,
    evidence_artifacts: Sequence[Any],
    missing_research: Mapping[str, Any] | None,
) -> EvalMetricResult:
    company_data = _mapping(company) if company else {}
    if not company_data:
        reason = _missing_reason(missing_research, "company")
        if reason:
            return EvalMetricResult(
                "citation_provenance",
                True,
                0.0,
                f"Company dossier absent after recorded company-research failure: {reason}",
            )
        return EvalMetricResult(
            "citation_provenance",
            False,
            0.0,
            "Company dossier is absent without a recorded company-research failure.",
        )
    known_evidence = {
        str(_field(artifact, "id")): artifact
        for artifact in evidence_artifacts
        if _field(artifact, "id")
    }
    problems: list[str] = []
    checked_references = 0

    def check_reference(evidence_id: Any, label: str) -> None:
        nonlocal checked_references
        checked_references += 1
        key = str(evidence_id or "").strip()
        if not key:
            problems.append(f"{label} has an empty evidence id")
            return
        artifact = known_evidence.get(key)
        if artifact is None:
            problems.append(f"{label} cites unknown evidence {key}")
            return
        problem = _provenance_problem(artifact)
        if problem:
            problems.append(f"{label}: {problem}")

    claims = _iter_items(company_data.get("claims"))
    if not claims and not _iter_items(company_data.get("tech_stack")):
        problems.append("company dossier has no evidence-cited claims or stack items")

    for index, claim in enumerate(claims, start=1):
        claim_data = _mapping(claim)
        evidence_ids = _iter_items(claim_data.get("evidence_ids"))
        if not evidence_ids:
            problems.append(f"claim {index} has no evidence_ids")
            continue
        for evidence_id in evidence_ids:
            check_reference(evidence_id, f"claim {index}")

    for index, item in enumerate(_iter_items(company_data.get("tech_stack")), start=1):
        item_data = _mapping(item)
        for evidence_id in _iter_items(item_data.get("evidence_ids")):
            check_reference(evidence_id, f"tech_stack item {index}")

    passed = not problems
    if passed:
        score = 1.0
    elif checked_references == 0:
        score = 0.0
    else:
        score = max(0.0, 1.0 - len(problems) / checked_references)
    reason = (
        "All company claims and stack items cite valid evidence provenance."
        if passed
        else "; ".join(problems[:5])
    )
    return EvalMetricResult("citation_provenance", passed, score, reason)


def _question_metric(
    questions: Sequence[Any], missing_outputs: Mapping[str, Any] | None
) -> EvalMetricResult:
    if not questions:
        reason = _missing_reason(missing_outputs, "interview question bank")
        if reason:
            return EvalMetricResult(
                "question_quality",
                True,
                0.0,
                f"Question bank absent after recorded generation failure: {reason}",
            )
        return EvalMetricResult(
            "question_quality",
            False,
            0.0,
            "Question bank is absent without a recorded generation failure.",
        )
    seen: dict[str, int] = {}
    problems: list[str] = []
    for index, question in enumerate(questions, start=1):
        text = _normalise_text(_field(question, "question"))
        if not text:
            problems.append(f"question {index} has empty text")
            continue
        if text in seen:
            problems.append(f"question {index} duplicates question {seen[text]}")
            continue
        seen[text] = index

    passed = not problems
    score = 1.0 if passed else max(0.0, 1.0 - len(problems) / len(questions))
    reason = (
        "All questions are non-empty and uniquely normalized."
        if passed
        else "; ".join(problems[:5])
    )
    return EvalMetricResult("question_quality", passed, score, reason)


def _skill_metric(
    questions: Sequence[Any],
    skills: Sequence[Any],
    relations: Sequence[Any],
    missing_outputs: Mapping[str, Any] | None,
) -> EvalMetricResult:
    if not skills:
        question_skill_refs = any(
            _iter_items(_field(question, "skills_tested")) for question in questions
        )
        if relations or question_skill_refs:
            return EvalMetricResult(
                "skill_reference_integrity",
                False,
                0.0,
                "Relations or questions reference skills but the curriculum has no skill nodes.",
            )
        reason = _missing_reason(missing_outputs, "knowledge graph") or _missing_reason(
            missing_outputs, "learning plan"
        )
        if reason:
            return EvalMetricResult(
                "skill_reference_integrity",
                True,
                0.0,
                f"Curriculum absent after recorded generation failure: {reason}",
            )
        return EvalMetricResult(
            "skill_reference_integrity",
            False,
            0.0,
            "Curriculum skills are absent without a recorded generation failure.",
        )

    known = _skill_keys(skills)
    problems: list[str] = []
    references = 0

    def check_skill(value: Any, label: str) -> None:
        nonlocal references
        references += 1
        key = _normalise_skill(value)
        if not key:
            problems.append(f"{label} has an empty skill reference")
        elif key not in known:
            problems.append(f"{label} references unknown skill {value!r}")

    for index, relation in enumerate(relations, start=1):
        relation_data = _mapping(relation)
        check_skill(relation_data.get("source"), f"relation {index} source")
        check_skill(relation_data.get("target"), f"relation {index} target")

    for index, question in enumerate(questions, start=1):
        question_data = _mapping(question)
        for skill in _iter_items(question_data.get("skills_tested")):
            check_skill(skill, f"question {index} skills_tested")

    passed = not problems
    score = (
        1.0 if references == 0 else max(0.0, 1.0 - len(problems) / references)
    )
    reason = (
        "All question and relation skill references resolve to curriculum skills."
        if passed
        else "; ".join(problems[:5])
    )
    return EvalMetricResult("skill_reference_integrity", passed, score, reason)


def _is_complements(relation: Any) -> bool:
    return _normalise_text(_field(relation, "relation", "prerequisite")) == COMPLEMENTS_RELATION


def _prerequisite_metric(relations: Sequence[Any]) -> EvalMetricResult:
    adjacency: dict[str, list[str]] = {}
    for relation in relations:
        if _is_complements(relation):
            continue
        source = _normalise_skill(_field(relation, "source"))
        target = _normalise_skill(_field(relation, "target"))
        if source and target:
            adjacency.setdefault(source, []).append(target)

    if not adjacency:
        return EvalMetricResult(
            "prerequisite_graph_acyclic",
            True,
            1.0,
            "No directed prerequisite relations to validate.",
        )

    state: dict[str, int] = {}

    def visit(node: str) -> bool:
        state[node] = 1
        for neighbour in adjacency.get(node, []):
            if state.get(neighbour) == 1:
                return False
            if state.get(neighbour, 0) == 0 and not visit(neighbour):
                return False
        state[node] = 2
        return True

    for node in list(adjacency):
        if state.get(node, 0) == 0 and not visit(node):
            return EvalMetricResult(
                "prerequisite_graph_acyclic",
                False,
                0.0,
                "Prerequisite cycle detected in directed skill relations.",
            )
    return EvalMetricResult(
        "prerequisite_graph_acyclic",
        True,
        1.0,
        "Prerequisite graph is a valid directed acyclic graph.",
    )


def validate_run_outputs(
    *,
    company: Any,
    evidence_artifacts: Sequence[Any],
    questions: Sequence[Any],
    skills: Sequence[Any],
    relations: Sequence[Any],
    missing_research: Mapping[str, Any] | None = None,
    missing_outputs: Mapping[str, Any] | None = None,
) -> RunOutputValidation:
    """Validate the exact run outputs a workflow is about to persist.

    Missing company research, interview questions, or curriculum skills pass
    only when the workflow records the corresponding degradation reason.
    Present outputs are validated for provenance, references, and structure.
    """

    safe_questions = _iter_items(questions)
    safe_skills = _iter_items(skills)
    safe_relations = _iter_items(relations)
    return RunOutputValidation(
        metrics=(
            _citation_metric(company, _iter_items(evidence_artifacts), missing_research),
            _question_metric(safe_questions, missing_outputs),
            _skill_metric(safe_questions, safe_skills, safe_relations, missing_outputs),
            _prerequisite_metric(safe_relations),
        )
    )
