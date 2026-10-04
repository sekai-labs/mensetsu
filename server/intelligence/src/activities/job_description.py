import re
from typing import Literal

from pydantic import BaseModel, Field
from temporalio import activity

from src.activities.persistence import persist_record
from src.db.client import SurrealDirectClient
from src.extractor.models import PromptInjectionDefense
from src.llm.roles import ModelRole
from src.llm.router import router

db_client = SurrealDirectClient()

class JobRequirement(BaseModel):
    skill_or_concept: str
    importance: Literal['mandatory', 'preferred']
    raw_requirement_text: str
    category: str = Field(default='Technical', description='e.g. Language, Architecture, Systems, Database, Domain')

class DynamicJDExtraction(BaseModel):
    title: str = 'Target Role'
    role_summary: str | None = None
    requirements: list[JobRequirement] = Field(default_factory=list)
    responsibilities: list[str] = Field(default_factory=list)

def heuristic_jd_fallback(jd_text: str) -> DynamicJDExtraction:
    known_tech = [('Distributed Systems', 'mandatory', 'Architecture'), ('Rust', 'mandatory', 'Language'), ('Kubernetes', 'mandatory', 'Infrastructure'), ('Temporal', 'preferred', 'Workflow'), ('SurrealDB', 'preferred', 'Database'), ('Linux', 'mandatory', 'Systems'), ('Python', 'preferred', 'Language'), ('Docker', 'mandatory', 'DevOps'), ('PostgreSQL', 'preferred', 'Database'), ('gRPC', 'preferred', 'Networking'), ('Go', 'mandatory', 'Language'), ('TypeScript', 'preferred', 'Language')]
    reqs: list[JobRequirement] = []
    lines = jd_text.splitlines()
    for tech, imp, cat in known_tech:
        pattern = re.compile(f'\\b{re.escape(tech)}\\b', re.IGNORECASE)
        match = pattern.search(jd_text)
        if match:
            start = max(0, match.start() - 60)
            end = min(len(jd_text), match.end() + 60)
            snippet = jd_text[start:end].strip()
            reqs.append(JobRequirement(skill_or_concept=tech, importance=imp, raw_requirement_text=snippet or f'Demonstrated proficiency in {tech} for building resilient cloud systems.', category=cat))
    if not reqs:
        for line in lines:
            line_str = line.strip()
            if (line_str.startswith('-') or line_str.startswith('*') or line_str.startswith('•')) and len(line_str) > 10:
                clean_item = line_str.lstrip('-*• ').strip()
                reqs.append(JobRequirement(skill_or_concept=clean_item[:40], importance='mandatory', raw_requirement_text=clean_item, category='Technical'))
    title = 'Software Engineer'
    for line in lines[:3]:
        s = line.strip()
        if s and len(s) < 50 and any((w in s.lower() for w in ['engineer', 'developer', 'architect', 'lead'])):
            title = s
            break
    return DynamicJDExtraction(title=title, role_summary='Target engineering position requiring hands-on technical and architectural mastery.', requirements=reqs, responsibilities=[line.strip() for line in lines if len(line.strip()) > 30][:5])

@activity.defn
async def parse_job_description_activity(
    workspace_id: str,
    jd_text: str = "",
    jd_url: str | None = None,
) -> dict:
    final_text = jd_text
    fetched_title = None
    if jd_url and not final_text.strip():
        try:
            from src.fetcher.reader import SafeReaderFetcher
            reader = SafeReaderFetcher(timeout_seconds=10.0)
            page_title, page_md = await reader.fetch_markdown(jd_url)
            if page_md.strip():
                final_text = page_md.strip()
                fetched_title = page_title
        except Exception as e:
            print(f"Error fetching JD URL {jd_url} for {workspace_id}: {e}")

    if not final_text.strip():
        final_text = "Software Engineer responsible for architecture, system design, high availability, and distributed cloud services."

    contained_text = PromptInjectionDefense.wrap_untrusted_input(final_text, label='JOB_DESCRIPTION')
    system_prompt = "You are an expert technical recruiter and systems architect analyzing a Job Description. Extract core competencies, required technologies, architectural principles, and classify each as 'mandatory' or 'preferred'. Include the raw excerpt sentence as raw_requirement_text."
    try:
        extraction = await router.generate_structured(schema=DynamicJDExtraction, system_prompt=system_prompt, user_prompt=f'Job Description:\n\n{contained_text}', role=ModelRole.EXTRACTION)
        if not extraction.requirements:
            extraction = heuristic_jd_fallback(final_text)
    except Exception:
        extraction = heuristic_jd_fallback(final_text)
    if fetched_title and extraction.title in ['Software Engineer', 'Target Role']:
        extraction.title = fetched_title
    extracted_reqs_list = [req.model_dump() for req in extraction.requirements]
    await persist_record(
        db_client=db_client,
        table='job_description',
        record_id=f'jd_{workspace_id}',
        fields={
            'workspace_id': workspace_id,
            'title': extraction.title,
            'role_summary': extraction.role_summary,
            'raw_text': final_text,
            'jd_url': jd_url,
            'requirements': extracted_reqs_list,
            'responsibilities': extraction.responsibilities,
        },
    )
    return {'title': extraction.title, 'requirements': extracted_reqs_list, 'responsibilities': extraction.responsibilities, 'raw_text': final_text}
