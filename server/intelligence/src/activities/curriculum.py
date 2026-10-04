from typing import Any, Literal

from pydantic import BaseModel, ConfigDict, Field
from temporalio import activity

from src.activities.persistence import persist_record
from src.db.client import SurrealDirectClient
from src.llm.roles import ModelRole
from src.llm.router import router

db_client = SurrealDirectClient()

class DiagramSpec(BaseModel):
    model_config = ConfigDict(populate_by_name=True)

    diagram_type: Literal["dependency_graph", "data_flow", "state_machine", "sequence_diagram"] = Field(
        alias="type", default="dependency_graph"
    )
    caption: str
    data: dict[str, Any] = Field(default_factory=dict)

class ChapterMark(BaseModel):
    timestamp: str
    title: str
    summary: str | None = None


class LearningUnitSpec(BaseModel):
    skill: str
    title: str
    prerequisites: list[str] = Field(default_factory=list)
    estimated_minutes: int = Field(default=30, ge=5, le=120)
    summary: str
    content_markdown: str
    diagram: DiagramSpec | None = None
    video_url: str | None = None
    video_title: str | None = None
    chapters: list[ChapterMark] = Field(default_factory=list)
    completed: bool = False


class CurriculumOutput(BaseModel):
    units: list[LearningUnitSpec] = Field(default_factory=list)

def heuristic_curriculum_fallback(gaps: list[dict]) -> CurriculumOutput:
    units = []
    if not gaps:
        gaps = [
            {"skill": "Distributed Systems", "importance": "mandatory"},
            {"skill": "Reliable Concurrency", "importance": "mandatory"},
        ]

    for g in gaps:
        skill = g.get("skill", "Core Systems Concept")
        units.append(
            LearningUnitSpec(
                skill=skill,
                title=f"Mastering {skill}: Architectural Patterns & Production Realities",
                prerequisites=["Operating Systems Basics", "Asynchronous Programming"],
                estimated_minutes=35,
                summary=f"In-depth guide to {skill} trade-offs, failure domain isolation, and performance optimization.",
                content_markdown=(
                    f"# Mastering {skill}\n\n"
                    f"## Core Invariants\n"
                    f"Understanding {skill} requires grasping fundamental failure modes and consistency guarantees.\n\n"
                    f"## Production Trade-offs\n"
                    f"Examine latency vs throughput, partition handling, and observable invariants."
                ),
                diagram=DiagramSpec(
                    type="data_flow",
                    caption=f"{skill} Execution Pipeline and State Boundaries",
                    data={
                        "nodes": ["Client Request", "Gateway Router", f"{skill} Engine", "Durable Storage"],
                        "edges": [
                            {"from": "Client Request", "to": "Gateway Router"},
                            {"from": "Gateway Router", "to": f"{skill} Engine"},
                            {"from": f"{skill} Engine", "to": "Durable Storage"},
                        ],
                    },
                ),
                video_url="https://www.youtube-nocookie.com/embed/t9fT7n8k_uM",
                video_title=f"Technical Deep Dive: {skill}",
                chapters=[
                    ChapterMark(timestamp="00:00", title="Architectural Overview", summary="High-level topology"),
                    ChapterMark(timestamp="05:20", title="Invariants & Fault Domains", summary="Failure model analysis"),
                    ChapterMark(timestamp="14:40", title="Live Implementation & Pitfalls", summary="Real-world case studies"),
                ],
                completed=False,
            )
        )
    return CurriculumOutput(units=units)

@activity.defn
async def generate_learning_units_activity(
    workspace_id: str,
    gaps: list[dict],
) -> list[dict]:
    gap_descriptions = "\n".join([f"- Skill: {g.get('skill')} (importance: {g.get('importance', 'mandatory')})" for g in gaps])

    system_prompt = (
        "You are a distinguished staff engineer and technical educator. "
        "Create high-yield, comprehensive learning units targeted at candidate gaps. "
        "Each unit must have detailed Markdown content, realistic prerequisites, and an informative visual diagram specification (DiagramSpec) with structured nodes and edges."
    )

    try:
        curriculum = await router.generate_structured(
            schema=CurriculumOutput,
            system_prompt=system_prompt,
            user_prompt=f"Identified Skill Gaps:\n{gap_descriptions}",
            role=ModelRole.CURRICULUM,
        )
        if not curriculum.units:
            curriculum = heuristic_curriculum_fallback(gaps)
    except Exception:
        curriculum = heuristic_curriculum_fallback(gaps)

    persisted_units = []
    for idx, u in enumerate(curriculum.units):
        lu_id = f"lu_{workspace_id}_{idx}"
        diagram_dump = u.diagram.model_dump(by_alias=True) if u.diagram else None

        unit_dict = {
            "id": lu_id,
            "workspace_id": workspace_id,
            "skill": u.skill,
            "title": u.title,
            "prerequisites": u.prerequisites,
            "estimated_minutes": u.estimated_minutes,
            "summary": u.summary,
            "content_markdown": u.content_markdown,
            "diagram": diagram_dump,
            "video_url": u.video_url or "https://www.youtube-nocookie.com/embed/t9fT7n8k_uM",
            "video_title": u.video_title or f"Technical Lecture: {u.skill}",
            "chapters": [c.model_dump() for c in u.chapters] if u.chapters else [],
            "completed": False,
        }
        await persist_record(
            db_client=db_client,
            table="learning_unit",
            record_id=lu_id,
            fields=unit_dict,
        )
        persisted_units.append(unit_dict)

    return persisted_units
