import re
from typing import Literal

from pydantic import BaseModel, Field
from temporalio import activity

from src.activities.persistence import persist_record
from src.db.client import SurrealDirectClient
from src.llm.roles import ModelRole
from src.llm.router import router

db_client = SurrealDirectClient()


class GraphSkillCandidate(BaseModel):
    name: str
    category: str = Field(default="Technical", description="Domain/category of skill")
    importance: Literal["mandatory", "preferred", "foundational"] = "mandatory"
    candidate_status: Literal["mastered", "gap", "exploring"] = "mastered"


class GraphEdgeCandidate(BaseModel):
    source: str
    target: str
    relation: str = Field(default="prerequisite_of", description="Relationship type")


class GraphGenerationOutput(BaseModel):
    skills: list[GraphSkillCandidate] = Field(default_factory=list)
    edges: list[GraphEdgeCandidate] = Field(default_factory=list)


def slugify(text: str) -> str:
    cleaned = re.sub(r"[^\w\s-]", "", text).strip().lower()
    return re.sub(r"[-\s]+", "_", cleaned) or "skill"


def heuristic_graph_fallback(company_name: str, skills: list[str]) -> GraphGenerationOutput:
    nodes = []
    edges = []
    prev_slug = None

    for s in skills:
        slug = slugify(s)
        nodes.append(
            GraphSkillCandidate(
                name=s,
                category="Systems" if any(w in s.lower() for w in ["rust", "linux", "distributed", "systems"]) else "Application",
                importance="mandatory",
                candidate_status="mastered",
            )
        )
        if prev_slug:
            edges.append(
                GraphEdgeCandidate(
                    source=prev_slug,
                    target=slug,
                    relation="relates_to",
                )
            )
        prev_slug = slug

    return GraphGenerationOutput(skills=nodes, edges=edges)


@activity.defn
async def construct_knowledge_graph_activity(
    workspace_id: str,
    company_name: str,
    skills: list[str],
) -> dict:
    system_prompt = (
        f"You are a knowledge graph architect mapping technical competencies for {company_name}. "
        "Build a directed acyclic competency graph connecting prerequisite foundational skills to advanced architecture concepts. "
        "Each node must have a clear category and candidate_status."
    )

    try:
        graph_output = await router.generate_structured(
            schema=GraphGenerationOutput,
            system_prompt=system_prompt,
            user_prompt=f"Identified Skills for {company_name}:\n" + "\n".join([f"- {s}" for s in skills]),
            role=ModelRole.SYNTHESIS,
        )
        if not graph_output.skills:
            graph_output = heuristic_graph_fallback(company_name, skills)
    except Exception:
        graph_output = heuristic_graph_fallback(company_name, skills)

    for skill_node in graph_output.skills:
        slug = slugify(skill_node.name)
        skill_record_id = f"{workspace_id}_{slug}"
        await persist_record(
            db_client=db_client,
            table="skill",
            record_id=skill_record_id,
            fields={
                "workspace_id": workspace_id,
                "name": skill_node.name,
                "slug": slug,
                "category": skill_node.category,
                "candidate_status": skill_node.candidate_status,
                "importance": skill_node.importance,
            },
        )

                                   
    for edge in graph_output.edges:
        source_slug = slugify(edge.source)
        target_slug = slugify(edge.target)
        edge_id = f"rel_{source_slug}_{target_slug}"
        await persist_record(
            db_client=db_client,
            table="skill_relation",
            record_id=edge_id,
            fields={
                "workspace_id": workspace_id,
                "source": source_slug,
                "target": target_slug,
                "relation": edge.relation,
            },
        )

    return {
        "nodes": [s.model_dump() for s in graph_output.skills],
        "edges": [e.model_dump() for e in graph_output.edges],
    }
