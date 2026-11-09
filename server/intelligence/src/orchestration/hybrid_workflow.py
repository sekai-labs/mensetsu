"""LangGraph assembly for the adaptive hybrid workflow."""
from langgraph.graph import END, START, StateGraph

from src.orchestration.hybrid_nodes import build_hybrid_nodes
from src.orchestration.hybrid_outputs import build_hybrid_output_nodes
from src.orchestration.hybrid_support import HybridWorkflowState
from src.orchestration.ledger import SurrealLedgerRepository

__all__ = ["HybridWorkflowState", "build_hybrid_workflow"]


def build_hybrid_workflow(checkpointer=None, ledger: SurrealLedgerRepository | None = None):
    if ledger is None:
        from src.orchestration.shared import default_ledger

        ledger = default_ledger

    nodes = build_hybrid_nodes(checkpointer, ledger)
    output_nodes = build_hybrid_output_nodes(ledger, nodes["set_run_phase"])
    route_and_normalize_node = nodes["route_and_normalize_node"]
    planner_node = nodes["planner_node"]
    dynamic_router = nodes["dynamic_router"]
    execute_dynamic_task_node = nodes["execute_dynamic_task_node"]
    synthesize_node = nodes["synthesize_node"]
    curriculum_node = output_nodes["curriculum_node"]
    interview_node = output_nodes["interview_node"]
    finalize_node = output_nodes["finalize_node"]

    builder = StateGraph(HybridWorkflowState)
    builder.add_node("route_and_normalize", route_and_normalize_node)
    builder.add_node("planner", planner_node)
    builder.add_node("execute_dynamic_task", execute_dynamic_task_node)
    builder.add_node("synthesize", synthesize_node)
    builder.add_node("curriculum", curriculum_node)
    builder.add_node("interview", interview_node)
    builder.add_node("finalize", finalize_node)

    builder.add_edge(START, "route_and_normalize")
    builder.add_edge("route_and_normalize", "planner")
    builder.add_conditional_edges("planner", dynamic_router, ["execute_dynamic_task", "synthesize"])
    builder.add_conditional_edges(
        "execute_dynamic_task", dynamic_router, ["execute_dynamic_task", "synthesize"]
    )
    builder.add_edge("synthesize", "curriculum")
    builder.add_edge("curriculum", "interview")
    builder.add_edge("interview", "finalize")
    builder.add_edge("finalize", END)

    return builder.compile(checkpointer=checkpointer)
