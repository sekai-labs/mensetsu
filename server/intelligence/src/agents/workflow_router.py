import logging
import time
from typing import Any

from langchain_core.messages import HumanMessage, SystemMessage

from src.agents.model_router import resolve_chat_model_for_role
from src.agents.schemas import WorkflowRoutingDecision

logger = logging.getLogger("mensetsu.agents.workflow_router")

ROUTER_PROMPT = """You are the Workflow Router Mini-Agent for Mensetsu (AI-driven interview prep platform).
Your task is to analyze the candidate's natural language preparation goals, resume, target company, role title, and optional job description.

Choose between three workflow execution modes:
1. "hybrid" (Default for custom interview preparation):
   - Combines deterministic milestones (CV/JD entity extraction, Match Analysis, Knowledge Graph, Question Book) with an autonomous Planner Agent that constructs targeted, deep-dive research tasks tailored to the prompt.
   - Recommended whenever the user provides a specific preparation goal, custom tech stack focus, or architectural curiosity alongside interview preparation.

2. "static":
   - Standard, predictable 9-task deterministic pipeline.
   - Recommended when the user provides standard company + role with no special custom prompt, or when prompt only requests standard CV/JD evaluation without deep custom investigation.

3. "dynamic":
   - Pure open-ended multi-agent research DAG without fixed business milestones or question books.
   - Recommended only when the user explicitly requests research-only dossiers or pure architecture study paths without interview prep/curriculum.

Also, if the company name or role title are not provided or are generic placeholders ("Target Company", "Target Role"), extract the real target company and role title directly from the prompt and resume context.
"""


async def route_workflow_request(
    user_prompt: str,
    company_name: str,
    role_title: str,
    resume_text: str = "",
    job_description: str = "",
    seniority: str | None = None,
    llm_config: dict[str, Any] | None = None,
    timeout_s: float = 3.0,
) -> WorkflowRoutingDecision:
    """
    Executes single-turn workflow classification under tight deadline (<= 1.5 - 3.0s).
    """
    start_t = time.perf_counter()

    if not user_prompt or not user_prompt.strip():
        return WorkflowRoutingDecision(
            selected_workflow="static",
            confidence=1.0,
            reasoning="No custom prompt provided; defaulted to static pipeline.",
            extracted_company=company_name if company_name != "Target Company" else None,
            extracted_role=role_title if role_title != "Target Role" else None,
        )

    try:
        chat_model = resolve_chat_model_for_role("router", llm_config, timeout_s=timeout_s)
        structured_model = chat_model.with_structured_output(WorkflowRoutingDecision)

        user_content = f"""User Prompt: {user_prompt}
Target Company: {company_name}
Target Role: {role_title}
Seniority: {seniority or "Unspecified"}
Resume excerpt: {resume_text[:1000]}
Job Description excerpt: {job_description[:1000]}
"""

        messages = [
            SystemMessage(content=ROUTER_PROMPT),
            HumanMessage(content=user_content),
        ]

        decision = await structured_model.ainvoke(messages)
        dur = (time.perf_counter() - start_t) * 1000
        logger.info(
            f"Workflow Router decided {decision.selected_workflow} in {dur:.1f}ms (confidence={decision.confidence})"
        )
        return decision
    except Exception as e:
        logger.warning(f"Workflow Router evaluation failed ({e}); falling back to hybrid workflow.")
        return WorkflowRoutingDecision(
            selected_workflow="hybrid",
            confidence=0.8,
            reasoning=f"Fallback to hybrid after router exception: {e}",
            extracted_company=company_name if company_name != "Target Company" else None,
            extracted_role=role_title if role_title != "Target Role" else None,
        )
