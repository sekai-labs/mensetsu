from typing import Literal

from pydantic import BaseModel, Field
from temporalio import activity

from src.activities.persistence import persist_record
from src.db.client import SurrealDirectClient
from src.llm.roles import ModelRole
from src.llm.router import router

db_client = SurrealDirectClient()

class CandidateStrength(BaseModel):
    skill: str
    evidence_quote: str
    relevance: str = Field(description='Why this strength maps to the role')

class CandidateGap(BaseModel):
    skill: str
    requirement_quote: str
    importance: Literal['mandatory', 'preferred']
    recommended_focus: str = Field(description='Targeted study focus or mental model to master')

class WordingImprovement(BaseModel):
    original_text: str
    suggested_revision: str
    rationale: str

class MatchAnalysisSynthesis(BaseModel):
    strengths: list[CandidateStrength] = Field(default_factory=list)
    gaps: list[CandidateGap] = Field(default_factory=list)
    risks: list[str] = Field(default_factory=list)
    wording_improvements: list[WordingImprovement] = Field(default_factory=list)

def calculate_deterministic_readiness(strengths: list[CandidateStrength], gaps: list[CandidateGap]) -> float:
    matched_skills = {s.skill.lower() for s in strengths}
    total_weight = 0.0
    earned_weight = 0.0
    for s in strengths:
        earned_weight += 1.5
        total_weight += 1.5
    for g in gaps:
        weight = 2.0 if g.importance == 'mandatory' else 1.0
        total_weight += weight
        if g.skill.lower() in matched_skills:
            earned_weight += weight
    if total_weight == 0:
        return 1.0
    return round(min(1.0, max(0.0, earned_weight / total_weight)), 2)

def heuristic_match_fallback(resume_skills: list[str], required_skills: list[dict]) -> MatchAnalysisSynthesis:
    strengths: list[CandidateStrength] = []
    gaps: list[CandidateGap] = []
    res_set = {s.lower() for s in resume_skills}
    for req in required_skills:
        skill = req.get('skill_or_concept', '')
        imp = req.get('importance', 'mandatory')
        raw_text = req.get('raw_requirement_text', f'Experience with {skill}')
        if any((r in skill.lower() or skill.lower() in r for r in res_set)):
            strengths.append(CandidateStrength(skill=skill, evidence_quote=f'Demonstrated background with {skill} in prior work.', relevance=f'Direct match for {imp} requirement.'))
        else:
            gaps.append(CandidateGap(skill=skill, requirement_quote=raw_text, importance=imp, recommended_focus=f'Review core fundamentals and common architectural failure modes for {skill}.'))
    for r in resume_skills[:3]:
        if not any((r.lower() == s.skill.lower() for s in strengths)):
            strengths.append(CandidateStrength(skill=r, evidence_quote=f'Strong background in {r}.', relevance='Valuable complementary technical capability.'))
    return MatchAnalysisSynthesis(strengths=strengths, gaps=gaps, risks=['Interviewers will probe deeply into systems scale and failure domain isolation.', 'Candidate should clearly articulate distributed consensus tradeoffs.'], wording_improvements=[WordingImprovement(original_text='Worked on database scaling.', suggested_revision='Architected low-latency distributed storage layer handling 50k req/sec with p99 < 15ms.', rationale='Quantifies operational impact and gives interviewers exact conversational anchors.')])

@activity.defn
async def calculate_match_analysis_activity(workspace_id: str, resume_skills: list[str], required_skills: list[dict]) -> dict:
    req_context = '\n'.join([f"- {r.get('skill_or_concept')} ({r.get('importance')}): {r.get('raw_requirement_text')}" for r in required_skills])
    resume_context = ', '.join(resume_skills)
    system_prompt = 'You are an elite engineering hiring committee chair. Analyze candidate resume skills against target job description requirements. Classify matched competencies into strengths (with evidence quotes) and missing requirements into gaps. Provide actionable resume bullet rewrites and critical interview risks.'
    try:
        synthesis = await router.generate_structured(schema=MatchAnalysisSynthesis, system_prompt=system_prompt, user_prompt=f'Candidate Resume Skills:\n{resume_context}\n\nJob Requirements:\n{req_context}', role=ModelRole.MATCHING)
        if not synthesis.strengths and (not synthesis.gaps):
            synthesis = heuristic_match_fallback(resume_skills, required_skills)
    except Exception:
        synthesis = heuristic_match_fallback(resume_skills, required_skills)
    readiness_score = calculate_deterministic_readiness(synthesis.strengths, synthesis.gaps)
    strengths_list = [s.model_dump() for s in synthesis.strengths]
    gaps_list = [g.model_dump() for g in synthesis.gaps]
    wording_list = [w.model_dump() for w in synthesis.wording_improvements]
    await persist_record(db_client=db_client, table='match_analysis', record_id=f'match_{workspace_id}', fields={'workspace_id': workspace_id, 'overall_match_ratio': readiness_score, 'strengths': strengths_list, 'gaps': gaps_list, 'risks': synthesis.risks, 'wording_improvements': wording_list})
    return {'overall_match_ratio': readiness_score, 'gaps': gaps_list, 'strengths': strengths_list, 'risks': synthesis.risks, 'wording_improvements': wording_list}
