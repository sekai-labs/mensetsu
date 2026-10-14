import asyncio

from langchain.agents import create_agent
from langchain.agents.structured_output import ToolStrategy
from langgraph.errors import GraphRecursionError
from pydantic import BaseModel, Field, create_model, model_validator

from src.agents.runtime import (
    LLM_REQUEST_TIMEOUT_S,
    agent_runtime,
    build_llm_retry_middleware,
    build_provider_slot_middleware,
)
from src.db.client import SurrealDirectClient
from src.extractor.models import PromptInjectionDefense
from src.generation.persistence import persist_record
from src.orchestration.limits import provider_key

db_client = SurrealDirectClient()

RESUME_EXTRACTION_RECURSION_LIMIT = 8


class ResumeExtractionError(RuntimeError):
    pass


class ResumeEvidence(BaseModel):
    skill_or_topic: str
    quote: str = Field(description="Verbatim excerpt copied from the candidate resume text")
    context: str = Field(description="Context of the experience or project")
    confidence: float = Field(ge=0.0, le=1.0, default=0.9)


class ExtractedResumeSkill(BaseModel):
    name: str
    category: str = Field(default="General", description="e.g. Systems, Frontend, Cloud, Tooling")
    evidence: list[ResumeEvidence] = Field(min_length=1)
    confidence: float = Field(ge=0.0, le=1.0, default=0.9)


class DynamicResumeExtraction(BaseModel):
    candidate_name: str | None = Field(
        default=None, description="Candidate name exactly as written; null if the resume does not state it"
    )
    summary: str | None = Field(
        default=None, description="Short factual summary of the resume; null if it cannot be derived"
    )
    skills: list[ExtractedResumeSkill] = Field(default_factory=list)
    experience_highlights: list[str] = Field(default_factory=list)


def normalize_whitespace(text: str) -> str:
    return " ".join(text.split())


def grounded_resume_schema(resume_text: str) -> type[DynamicResumeExtraction]:
    flat_resume = normalize_whitespace(resume_text)

    def quotes_are_verbatim(self: DynamicResumeExtraction) -> DynamicResumeExtraction:
        missing = [
            f"{skill.name}: {ev.quote!r}"
            for skill in self.skills
            for ev in skill.evidence
            if not normalize_whitespace(ev.quote) or normalize_whitespace(ev.quote) not in flat_resume
        ]
        if missing:
            raise ValueError(
                "Every evidence quote must be copied verbatim from the resume text. "
                f"These quotes do not appear in the resume: {'; '.join(missing[:10])}"
            )
        return self

    return create_model(
        DynamicResumeExtraction.__name__,
        __base__=DynamicResumeExtraction,
        __validators__={"quotes_are_verbatim": model_validator(mode="after")(quotes_are_verbatim)},
    )


async def extract_resume(final_text: str, workspace_id: str) -> DynamicResumeExtraction:
    contained_text = PromptInjectionDefense.wrap_untrusted_input(
        final_text, label="CANDIDATE_RESUME"
    )
    system_prompt = (
        "You are an expert technical interviewer and resume analyst. Extract technical skills, tools, "
        "frameworks, and architecture competencies from the provided candidate resume. Do NOT invent skills. "
        "Every skill MUST cite at least one evidence quote copied verbatim from the resume text. "
        "Set candidate_name and summary to null when the resume does not support them."
    )
    chat_model = agent_runtime.get_chat_model()
    agent = create_agent(
        model=chat_model,
        tools=[],
        system_prompt=system_prompt,
        response_format=ToolStrategy(grounded_resume_schema(final_text), handle_errors=True),
        middleware=[
            build_llm_retry_middleware(f"Resume extraction ({workspace_id})"),
            build_provider_slot_middleware(provider_key(chat_model)),
        ],
        checkpointer=False,
    )
    try:
        async with asyncio.timeout(LLM_REQUEST_TIMEOUT_S * 2):
            res = await agent.ainvoke(
                {"messages": [{"role": "user", "content": f"Resume:\n\n{contained_text}"}]},
                config={"recursion_limit": RESUME_EXTRACTION_RECURSION_LIMIT},
            )
    except GraphRecursionError as e:
        raise ResumeExtractionError(
            f"Resume extraction for workspace {workspace_id} produced no valid structured output "
            f"within {RESUME_EXTRACTION_RECURSION_LIMIT} graph steps."
        ) from e
    except TimeoutError as e:
        raise ResumeExtractionError(
            f"Resume extraction for workspace {workspace_id} timed out after {LLM_REQUEST_TIMEOUT_S * 2:.0f}s."
        ) from e
    structured = res.get("structured_response") if isinstance(res, dict) else None
    if not isinstance(structured, DynamicResumeExtraction):
        raise ResumeExtractionError(
            f"Resume extraction for workspace {workspace_id} returned no structured response."
        )
    return structured


async def parse_resume_activity(
    workspace_id: str,
    resume_text: str,
    db_client: SurrealDirectClient | None = None,
) -> dict:
    final_text = resume_text.strip()
    if not final_text:
        raise ValueError(f"No resume text available to parse for workspace {workspace_id}.")
    extraction = await extract_resume(final_text, workspace_id)
    extracted_skills_list = [
        {
            "name": s.name,
            "category": s.category,
            "evidence_snippet": s.evidence[0].quote,
            "confidence": s.confidence,
            "evidence": [e.model_dump() for e in s.evidence],
        }
        for s in extraction.skills
    ]
    skills_names = [s["name"] for s in extracted_skills_list]
    client_to_use = db_client or globals()["db_client"]
    await persist_record(
        db_client=client_to_use,
        table="resume",
        record_id=f"res_{workspace_id}",
        fields={
            "workspace_id": workspace_id,
            "candidate_name": extraction.candidate_name,
            "summary": extraction.summary,
            "raw_text": final_text,
            "parsed_skills": skills_names,
            "extracted_skills": extracted_skills_list,
            "experience": [],
            "education": [],
            "experience_highlights": extraction.experience_highlights,
        },
    )
    return {
        "extracted_skills": extracted_skills_list,
        "candidate_name": extraction.candidate_name,
        "summary": extraction.summary,
        "experience_highlights": extraction.experience_highlights,
        "raw_text": final_text,
    }


async def load_parsed_resume(workspace_id: str, db_client: SurrealDirectClient | None = None) -> dict | None:
    client_to_use = db_client or globals()["db_client"]
    res = await client_to_use.execute_query(
        "SELECT * FROM type::record('resume', $rid);", vars={"rid": f"res_{workspace_id}"}
    )
    rows = [r for item in res if isinstance(item, dict) for r in (item.get("result") or []) if isinstance(r, dict)]
    if not rows or not rows[-1].get("raw_text"):
        return None
    row = rows[-1]
    return {
        "extracted_skills": row.get("extracted_skills") or [],
        "candidate_name": row.get("candidate_name"),
        "summary": row.get("summary"),
        "experience_highlights": row.get("experience_highlights") or [],
        "raw_text": row["raw_text"],
    }
