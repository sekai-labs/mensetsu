from dataclasses import dataclass
from typing import Any


@dataclass
class EvalMetricResult:
    metric_name: str
    passed: bool
    score: float
    reason: str


class ResearchEvaluationHarness:
    @staticmethod
    def evaluate_citation_provenance(
        sources: list[dict[str, Any]], claims: list[dict[str, Any]]
    ) -> EvalMetricResult:
        if not claims:
            return EvalMetricResult("citation_provenance", True, 1.0, "No claims to evaluate.")
        valid_source_ids = {s.get("id") or s.get("url") for s in sources}
        unsupported = [
            c
            for c in claims
            if not c.get("source_url") and c.get("source_id") not in valid_source_ids
        ]
        passed = len(unsupported) == 0
        score = (len(claims) - len(unsupported)) / len(claims)
        reason = (
            f"{len(unsupported)} claims missing verifiable citation provenance."
            if not passed
            else "All claims backed by citations."
        )
        return EvalMetricResult("citation_provenance", passed, score, reason)

    @staticmethod
    def evaluate_question_deduplication(questions: list[dict[str, Any]]) -> EvalMetricResult:
        seen_texts = set()
        duplicates = []
        for q in questions:
            norm = q.get("question", "").strip().lower()
            if norm in seen_texts:
                duplicates.append(norm)
            seen_texts.add(norm)
        passed = len(duplicates) == 0
        score = 1.0 if passed else max(0.0, 1.0 - len(duplicates) / max(1, len(questions)))
        return EvalMetricResult(
            "question_deduplication", passed, score, f"Found {len(duplicates)} duplicates."
        )

    @staticmethod
    def evaluate_prerequisite_graph_acyclic(edges: list[dict[str, str]]) -> EvalMetricResult:
        adj: dict[str, list[str]] = {}
        for e in edges:
            adj.setdefault(e["source"], []).append(e["target"])
        visited: set[str] = set()
        recursion_stack: set[str] = set()

        def has_cycle(node: str) -> bool:
            visited.add(node)
            recursion_stack.add(node)
            for neighbor in adj.get(node, []):
                if neighbor not in visited:
                    if has_cycle(neighbor):
                        return True
                elif neighbor in recursion_stack:
                    return True
            recursion_stack.remove(node)
            return False

        has_err = False
        for node in list(adj.keys()):
            if node not in visited:
                if has_cycle(node):
                    has_err = True
                    break
        return EvalMetricResult(
            "prerequisite_graph_acyclic",
            not has_err,
            0.0 if has_err else 1.0,
            "Cycle detected in learning prerequisites."
            if has_err
            else "Prerequisite graph is a valid DAG.",
        )


def run_evaluation_suite(payload: dict[str, Any]) -> list[EvalMetricResult]:
    harness = ResearchEvaluationHarness()
    results = [
        harness.evaluate_citation_provenance(payload.get("sources", []), payload.get("claims", [])),
        harness.evaluate_question_deduplication(payload.get("questions", [])),
        harness.evaluate_prerequisite_graph_acyclic(payload.get("edges", [])),
    ]
    return results
