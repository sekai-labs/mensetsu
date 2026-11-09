"""LangGraph assembly for the deterministic static workflow."""
from typing import Any

from langgraph.graph import END, START, StateGraph

from src.db.client import SurrealDirectClient
from src.orchestration.ledger import SurrealLedgerRepository
from src.orchestration.static_curriculum import build_curriculum_nodes
from src.orchestration.static_input import build_input_nodes
from src.orchestration.static_outputs import build_output_nodes
from src.orchestration.static_questions import build_question_nodes
from src.orchestration.static_research import build_research_nodes
from src.orchestration.static_support import StaticWorkflowState, build_support
from src.orchestration.static_synthesis import build_synthesis_nodes

__all__ = ["StaticWorkflowState", "build_static_workflow"]


def build_static_workflow(
    checkpointer: Any,
    ledger: SurrealLedgerRepository | None = None,
    db_client: SurrealDirectClient | None = None,
    mode: str = "full",
):
    if mode not in ("full", "normalize_only", "outputs_only"):
        raise ValueError(f"Unknown static workflow mode '{mode}'.")
    if checkpointer is None:
        if mode == "normalize_only":
            from langgraph.checkpoint.memory import MemorySaver

            checkpointer = MemorySaver()
        else:
            raise ValueError("Checkpointer is required to compile static workflow.")
    ledger = ledger or SurrealLedgerRepository(db_client=db_client)
    from src.agents.runtime import current_ledger_var

    current_ledger_var.set(ledger)
    db = db_client or ledger.db

    support = build_support(ledger, db)
    normalize_input_node = build_input_nodes(support)
    company_research_node, job_analysis_node, technical_research_node = build_research_nodes(
        support
    )
    synthesize_node = build_synthesis_nodes(support)
    curriculum_node = build_curriculum_nodes(support)
    generate_questions_node, validate_questions_node, validate_condition = build_question_nodes(
        support
    )
    persist_outputs_node = build_output_nodes(support, generate_questions_node)

    builder = StateGraph(StaticWorkflowState)
    builder.add_node("synthesize", synthesize_node)
    builder.add_node("curriculum", curriculum_node)
    builder.add_node("generate_questions", generate_questions_node)
    builder.add_node("validate_questions", validate_questions_node)
    builder.add_node("persist_outputs", persist_outputs_node)
    if mode == "normalize_only":
        normalize_builder = StateGraph(StaticWorkflowState)
        normalize_builder.add_node("normalize", normalize_input_node)
        normalize_builder.add_edge(START, "normalize")
        normalize_builder.add_edge("normalize", END)
        return normalize_builder.compile(checkpointer=checkpointer)
    if mode == "outputs_only":
        builder.add_edge(START, "synthesize")
    else:
        builder.add_node("normalize", normalize_input_node)
        builder.add_node("company_research", company_research_node)
        builder.add_node("job_analysis", job_analysis_node)
        builder.add_node("technical_research", technical_research_node)
        builder.add_edge(START, "normalize")
        builder.add_edge("normalize", "company_research")
        builder.add_edge("normalize", "job_analysis")
        builder.add_edge("normalize", "technical_research")
        builder.add_edge("company_research", "synthesize")
        builder.add_edge("job_analysis", "synthesize")
        builder.add_edge("technical_research", "synthesize")
    builder.add_edge("synthesize", "curriculum")
    builder.add_edge("curriculum", "generate_questions")
    builder.add_edge("generate_questions", "validate_questions")
    builder.add_conditional_edges("validate_questions", validate_condition)
    builder.add_edge("persist_outputs", END)

    return builder.compile(checkpointer=checkpointer)
