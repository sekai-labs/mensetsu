import re

from pydantic import BaseModel, Field
from temporalio import activity

from src.activities.persistence import persist_record
from src.db.client import SurrealDirectClient
from src.extractor.models import PromptInjectionDefense
from src.llm.roles import ModelRole
from src.llm.router import router

db_client = SurrealDirectClient()

class ResumeEvidence(BaseModel):
    skill_or_topic: str
    quote: str = Field(description='Exact snippet or quote from the candidate resume')
    context: str = Field(description='Context of the experience or project')
    confidence: float = Field(ge=0.0, le=1.0, default=0.9)

class ExtractedResumeSkill(BaseModel):
    name: str
    category: str = Field(default='General', description='e.g. Systems, Frontend, Cloud, Tooling')
    evidence: list[ResumeEvidence] = Field(default_factory=list)
    confidence: float = Field(ge=0.0, le=1.0, default=0.9)

class DynamicResumeExtraction(BaseModel):
    candidate_name: str | None = None
    summary: str | None = None
    skills: list[ExtractedResumeSkill] = Field(default_factory=list)
    experience_highlights: list[str] = Field(default_factory=list)

def heuristic_resume_fallback(resume_text: str) -> DynamicResumeExtraction:
    known_tech = [('Rust', 'Systems'), ('Python', 'Backend'), ('Kubernetes', 'Cloud'), ('Distributed Systems', 'Architecture'), ('PostgreSQL', 'Database'), ('Temporal', 'Workflow'), ('Docker', 'DevOps'), ('Linux', 'Systems'), ('gRPC', 'Networking'), ('SurrealDB', 'Database'), ('TypeScript', 'Frontend'), ('Vue', 'Frontend'), ('Go', 'Systems'), ('Kafka', 'Messaging'), ('AWS', 'Cloud')]
    extracted: list[ExtractedResumeSkill] = []
    lines = resume_text.splitlines()
    candidate_name = None
    for line in lines[:5]:
        stripped = line.strip()
        if stripped and len(stripped) < 40 and (not any((kw in stripped.lower() for kw in ['resume', 'curriculum', 'cv']))):
            candidate_name = stripped
            break
    for tech, cat in known_tech:
        pattern = re.compile(f'\\b{re.escape(tech)}\\b', re.IGNORECASE)
        match = pattern.search(resume_text)
        if match:
            start = max(0, match.start() - 60)
            end = min(len(resume_text), match.end() + 60)
            snippet = resume_text[start:end].strip()
            extracted.append(ExtractedResumeSkill(name=tech, category=cat, evidence=[ResumeEvidence(skill_or_topic=tech, quote=snippet, context='Experience demonstrated in resume text.', confidence=0.95)], confidence=0.95))
    extra_matches = re.findall('\\b([A-Z][a-zA-Z0-9_+#\\.\\-]{2,15})\\b', resume_text)
    seen = {s.name.lower() for s in extracted}
    common_noise = {'and', 'the', 'for', 'with', 'this', 'that', 'from', 'engineer', 'senior', 'lead', 'developer', 'experience', 'education'}
    for word in extra_matches:
        if word.lower() not in seen and word.lower() not in common_noise and (len(word) > 2):
            seen.add(word.lower())
            extracted.append(ExtractedResumeSkill(name=word, category='Technical Skill', evidence=[ResumeEvidence(skill_or_topic=word, quote=f'Referenced {word} in resume.', context='Mentioned technical skill or tool', confidence=0.8)], confidence=0.8))
    return DynamicResumeExtraction(candidate_name=candidate_name or 'Candidate', summary=f"Technical candidate with proficiency in {', '.join([s.name for s in extracted[:5]])}.", skills=extracted, experience_highlights=[line.strip() for line in lines if len(line.strip()) > 30][:4])

@activity.defn
async def parse_resume_activity(
    workspace_id: str,
    resume_text: str = "",
    resume_pdf_base64: str | None = None,
    resume_filename: str | None = None,
) -> dict:
    final_text = resume_text or ""
    if resume_pdf_base64:
        try:
            import base64

            from src.activities.documents import DocumentParser
            pdf_bytes = base64.b64decode(resume_pdf_base64)
            parsed_pdf_text = DocumentParser.extract_text_from_bytes(
                pdf_bytes, resume_filename or "resume.pdf"
            )
            if parsed_pdf_text and parsed_pdf_text.strip():
                final_text = parsed_pdf_text.strip()
        except Exception as e:
            print(f"Error decoding or parsing resume PDF for {workspace_id}: {e}")
            if not final_text.strip():
                final_text = resume_text or ""

    if not final_text.strip():
        final_text = "Technical Software Engineer with experience in systems, backend architecture, and infrastructure."
    contained_text = PromptInjectionDefense.wrap_untrusted_input(final_text, label='CANDIDATE_RESUME')
    system_prompt = 'You are an expert technical interviewer and resume analyst. Extract technical skills, tools, frameworks, and architecture competencies with verifiable evidence quotes from the provided candidate resume. Do NOT invent skills. Every extracted skill MUST have an exact quote from the resume text.'
    try:
        extraction = await router.generate_structured(schema=DynamicResumeExtraction, system_prompt=system_prompt, user_prompt=f'Resume:\n\n{contained_text}', role=ModelRole.EXTRACTION)
        if not extraction.skills:
            extraction = heuristic_resume_fallback(final_text)
    except Exception:
        extraction = heuristic_resume_fallback(final_text)
    extracted_skills_list = []
    for s in extraction.skills:
        first_evidence = s.evidence[0].quote if s.evidence else f'Mentioned experience with {s.name} in technical resume.'
        extracted_skills_list.append({'name': s.name, 'category': s.category, 'evidence_snippet': first_evidence, 'confidence': s.confidence, 'evidence': [e.model_dump() for e in s.evidence]})
    skills_names = [s['name'] for s in extracted_skills_list]
    await persist_record(
        db_client=db_client,
        table='resume',
        record_id=f'res_{workspace_id}',
        fields={
            'workspace_id': workspace_id,
            'candidate_name': extraction.candidate_name,
            'summary': extraction.summary,
            'raw_text': final_text,
            'parsed_skills': skills_names,
            'extracted_skills': extracted_skills_list,
            'experience': [],
            'education': [],
            'experience_highlights': extraction.experience_highlights,
        },
    )
    return {'extracted_skills': extracted_skills_list, 'candidate_name': extraction.candidate_name, 'summary': extraction.summary, 'raw_text': final_text}
