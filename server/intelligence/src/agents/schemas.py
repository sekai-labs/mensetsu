import contextvars
from typing import Literal

from pydantic import BaseModel, Field, model_validator

current_saved_evidence_ids_var: contextvars.ContextVar[set[str] | None] = contextvars.ContextVar(
    "current_saved_evidence_ids", default=None
)
current_execution_saved_evidence_ids_var: contextvars.ContextVar[set[str] | None] = (
    contextvars.ContextVar("current_execution_saved_evidence_ids", default=None)
)


def _validate_evidence_id_list(evidence_ids: list[str], field_name: str) -> None:
    valid_ids = current_saved_evidence_ids_var.get()
    if valid_ids is None:
        return
    unknown = [eid for eid in evidence_ids if eid not in valid_ids]
    if unknown:
        raise ValueError(
            f"Field '{field_name}' references unknown or unsaved evidence IDs: {unknown}. "
            f"You must only reference evidence IDs returned by 'save_evidence'. "
            f"Currently saved evidence IDs for this execution: {sorted(list(valid_ids))}."
        )


class AgentTaskOutput(BaseModel):
    summary: str = Field(description="Concise, factual summary of findings and research deduction.")
    claims: list[str] = Field(default_factory=list, description="Verified claims extracted.")
    tools_used: list[str] = Field(default_factory=list, description="Tools used during execution.")
    status: Literal["completed", "failed", "blocked"] = "completed"
    confidence: float = Field(default=0.9, ge=0.0, le=1.0)


class SystemsDeepDiveOutput(BaseModel):
    summary: str = Field(description="Exhaustive architectural breakdown of the delegated subsystem.")
    subsystem: str = Field(description="The specific subsystem investigated, e.g. networking or storage.")
    evidence_ids: list[str] = Field(
        min_length=2,
        description="At least 2 distinct evidence IDs saved BY THIS EXECUTION demonstrating concrete implementation details.",
    )
    findings: list[str] = Field(
        min_length=2,
        description="Core operational findings, tradeoffs, and failure modes.",
    )

    @model_validator(mode="after")
    def validate_child_provenance(self) -> "SystemsDeepDiveOutput":
        execution_saved = current_execution_saved_evidence_ids_var.get()
        if execution_saved is not None:
            invalid = [eid for eid in self.evidence_ids if eid not in execution_saved]
            if invalid:
                saved_list = sorted(list(execution_saved))
                raise ValueError(
                    f"Validation failed: systems-deep-dive MUST cite at least 2 distinct evidence items saved BY THIS EXECUTION. "
                    f"Evidence IDs {invalid} were not saved in this execution (cannot cite parent evidence). "
                    f"Evidence saved by this execution: {saved_list}. "
                    f"You MUST call 'read_source' on real sources and 'save_evidence' before emitting this output."
                )
        distinct = set(self.evidence_ids)
        if len(distinct) < 2:
            raise ValueError(
                f"Requires at least 2 distinct evidence_ids saved by this execution, got {len(distinct)} ({distinct})."
            )
        return self


class CompanyTechStackItem(BaseModel):
    name: str = Field(
        description="Technology or framework name (e.g. Tokio, PostgreSQL, Kubernetes)."
    )
    category: str = Field(
        description="Category (e.g. Language, Database, Infrastructure, Observability)."
    )
    evidence_ids: list[str] = Field(
        default_factory=list, description="IDs of saved evidence supporting this stack item."
    )
    verified: bool = Field(
        default=True, description="Whether this stack item is verified by primary evidence."
    )
    notes: str = Field(default="", description="Operational observations or production notes.")


class CompanyClaim(BaseModel):
    statement: str = Field(
        description="Verifiable factual statement about the company, architecture, or team."
    )
    category: Literal["architecture", "engineering_culture", "business", "operations"] = Field(
        description="architecture | engineering_culture | business | operations",
    )
    confidence: float = Field(ge=0.0, le=1.0, description="Confidence score.")
    evidence_ids: list[str] = Field(description="IDs of saved evidence citations.")


class CompanyResearchOutput(BaseModel):
    overview: str = Field(
        description="Comprehensive technical executive overview of company and mission."
    )
    business_model: str = Field(
        description="How the company operates, monetization, and target market."
    )
    engineering_culture: list[str] = Field(
        description="Observable traits of the engineering organization and practices."
    )
    tech_stack: list[CompanyTechStackItem] = Field(
        description="Verified technologies in production."
    )
    architecture_notes: list[str] = Field(
        description="System design patterns, consensus mechanisms, and data topologies."
    )
    interview_process_notes: list[str] = Field(
        description="Documented stages, rounds, and candidate evaluation criteria."
    )
    claims: list[CompanyClaim] = Field(
        min_length=8,
        description="At least 8 evidence-backed claims extracted from authoritative sources.",
    )
    summary: str = Field(
        description="Concise synthesis for workflow ledger and execution overview."
    )

    @model_validator(mode="after")
    def validate_provenance(self) -> "CompanyResearchOutput":
        all_ev_ids = []
        for claim in self.claims:
            _validate_evidence_id_list(claim.evidence_ids, f"claim: {claim.statement[:30]}")
            all_ev_ids.extend(claim.evidence_ids)
        for item in self.tech_stack:
            if item.evidence_ids:
                _validate_evidence_id_list(item.evidence_ids, f"tech_stack: {item.name}")
                all_ev_ids.extend(item.evidence_ids)
        distinct_ev = set(all_ev_ids)
        if len(distinct_ev) < 4:
            raise ValueError(
                f"CompanyResearchOutput requires at least 4 distinct saved evidence IDs, but only {len(distinct_ev)} provided ({distinct_ev}). "
                f"You MUST use 'read_source' and 'save_evidence' to collect at least 4 authoritative citations."
            )
        return self


class JobRequirementItem(BaseModel):
    skill: str = Field(description="Skill, technology, or architectural competency.")
    level: str = Field(
        default="proficient", description="foundational | proficient | expert | master"
    )
    required: Literal["required", "preferred"] = Field(
        description="Whether explicitly required or preferred."
    )
    quote: str = Field(description="Verbatim snippet from the job description.")


class JobAnalysisOutput(BaseModel):
    role_summary: str = Field(description="Structured summary of the role, seniority, and mandate.")
    requirements: list[JobRequirementItem] = Field(
        description="Exhaustive structured list of technical competencies."
    )
    responsibilities: list[str] = Field(
        description="Core operational and architectural duties expected."
    )
    seniority_signals: list[str] = Field(
        description="Indicators of seniority scope, ownership, and leadership expectations."
    )
    evidence_ids: list[str] = Field(
        min_length=1,
        description="At least 1 saved evidence citation from the job description or primary role documentation.",
    )
    summary: str = Field(description="Executive summary of candidate expectations.")

    @model_validator(mode="after")
    def validate_provenance(self) -> "JobAnalysisOutput":
        _validate_evidence_id_list(self.evidence_ids, "job_analysis: evidence_ids")
        return self


class TechnicalTopicItem(BaseModel):
    name: str = Field(
        description="Core technical or architectural topic (e.g. Distributed Consensus, LSM-Trees)."
    )
    why_it_matters: str = Field(
        description="Why this topic is crucial for the company's systems or target role."
    )
    key_concepts: list[str] = Field(
        description="Underlying mechanisms, tradeoffs, and theoretical foundations."
    )
    pitfalls: list[str] = Field(
        description="Common architectural failure modes, anti-patterns, or scaling bottlenecks."
    )
    evidence_ids: list[str] = Field(
        default_factory=list, description="IDs of saved evidence supporting this topic."
    )


class TechnicalResearchOutput(BaseModel):
    topics: list[TechnicalTopicItem] = Field(
        description="Deep dive technical topics relevant to the engineering stack."
    )
    architecture_topology: dict[str, str] = Field(
        default_factory=dict,
        description="Topology key-values: system_type, sync_mechanism, data_stores, components.",
    )
    summary: str = Field(description="Summary of technical stack research findings.")

    @model_validator(mode="after")
    def validate_provenance(self) -> "TechnicalResearchOutput":
        all_ev_ids = []
        for topic in self.topics:
            if topic.evidence_ids:
                _validate_evidence_id_list(topic.evidence_ids, f"topic: {topic.name}")
                all_ev_ids.extend(topic.evidence_ids)
        distinct_ev = set(all_ev_ids)
        if len(distinct_ev) < 4:
            raise ValueError(
                f"TechnicalResearchOutput requires at least 4 distinct saved evidence IDs across topics, but only {len(distinct_ev)} provided ({distinct_ev}). "
                f"You MUST use 'read_source' and 'save_evidence' to collect at least 4 technical citations."
            )
        return self


class InterviewQuestionItem(BaseModel):
    question: str = Field(description="Clear, realistic interview question prompt.")
    category: Literal[
        "resume_deep_dive",
        "project_deep_dive",
        "technical_fundamentals",
        "target_stack",
        "system_design",
        "debugging",
        "behavioral",
    ]
    difficulty: Literal["easy", "medium", "hard"]
    rationale: str = Field(
        description="Why this question is targeted for the company's stack and candidate level."
    )
    skills_tested: list[str] = Field(
        min_length=1,
        description="Specific technical skills or architectural concepts evaluated by this question.",
    )
    interviewer_intent: str = Field(
        description="What the interviewer intends to evaluate with this question."
    )
    expected_signals: list[str] = Field(
        min_length=2,
        description="At least 2 positive and negative candidate evaluation criteria.",
    )
    answer_outline: list[str] = Field(
        min_length=3,
        description="Pragmatic structured blueprint for a distinguished answer (at least 3 bullets).",
    )
    pitfalls: list[str] = Field(
        min_length=2,
        description="At least 2 red flags and superficial answer traps to avoid.",
    )
    follow_ups: list[str] = Field(
        min_length=2,
        description="At least 2 probing follow-up questions an interviewer may ask.",
    )
    evidence_ids: list[str] = Field(
        min_length=1,
        description="At least 1 evidence ID from input cited as provenance for this question.",
    )


class CategoryQuestionBatchOutput(BaseModel):
    questions: list[InterviewQuestionItem] = Field(
        min_length=2,
        description="Targeted interview questions for this category with rubrics; exactly the requested count.",
    )
    summary: str = Field(
        description="Summary of generated question portfolio and evaluation rubrics."
    )


class InterviewGenerationOutput(BaseModel):
    questions: list[InterviewQuestionItem] = Field(
        min_length=15,
        description="Comprehensive question book containing at least 15 targeted, evidence-grounded interview questions spanning at least 4 categories.",
    )
    summary: str = Field(
        description="Summary of generated question portfolio and evaluation rubrics."
    )


class ClaimVerdict(BaseModel):
    claim_statement: str = Field(description="The claim evaluated.")
    verified: bool = Field(description="True if proven by authoritative evidence sources.")
    verdict_explanation: str = Field(description="Analysis of claim validity against citations.")
    evidence_ids: list[str] = Field(
        default_factory=list, description="Associated evidence citation record IDs."
    )
    confidence: float = Field(default=0.9, ge=0.0, le=1.0)


class CandidateStrengthItem(BaseModel):
    skill: str
    evidence_quote: str
    relevance: Literal["high", "medium", "low"] = "high"


class CandidateGapItem(BaseModel):
    skill: str
    requirement_quote: str
    importance: Literal["required", "preferred", "optional"] = "required"
    recommended_focus: str


class EvidenceEvaluatorOutput(BaseModel):
    overall_match_ratio: float = Field(
        ge=0.0, le=1.0, description="Quantified candidate readiness ratio."
    )
    strengths: list[CandidateStrengthItem] = Field(
        min_length=3,
        description="At least 3 verified candidate competencies matching requirements with quotes.",
    )
    gaps: list[CandidateGapItem] = Field(
        min_length=3,
        description="At least 3 identified competencies where candidate evidence is lacking with quotes.",
    )
    risks: list[str] = Field(
        default_factory=list, description="Potential interview risks or areas of vulnerability."
    )
    wording_improvements: list[dict[str, str]] = Field(
        default_factory=list,
        description="Actionable resume wording improvements.",
    )
    claim_verdicts: list[ClaimVerdict] = Field(
        default_factory=list,
        description="Formal verification verdicts on extracted claims.",
    )
    summary: str = Field(description="Synthesis of candidate match analysis and readiness.")

    @model_validator(mode="after")
    def validate_provenance(self) -> "EvidenceEvaluatorOutput":
        for verdict in self.claim_verdicts:
            if verdict.evidence_ids:
                _validate_evidence_id_list(
                    verdict.evidence_ids, f"verdict: {verdict.claim_statement[:30]}"
                )
        return self


class SkillNodeItem(BaseModel):
    name: str = Field(description="Name of the skill or technical concept.")
    slug: str = Field(description="Unique slug for the skill.")
    category: str = Field(description="e.g. Systems, Backend, Architecture, DevOps, Database.")
    candidate_status: Literal["verified", "in_progress", "gap"] = "in_progress"
    importance: Literal["mandatory", "preferred", "optional"] = "mandatory"


class SkillEdgeItem(BaseModel):
    source: str = Field(description="Source skill name.")
    target: str = Field(description="Target skill name.")
    relation: str = Field(
        default="prerequisite", description="prerequisite | builds_on | complements"
    )


class LearningUnitItem(BaseModel):
    skill: str = Field(description="Skill name.")
    title: str = Field(description="Unit title.")
    prerequisites: list[str] = Field(default_factory=list, description="Prerequisite skills.")
    estimated_minutes: int = Field(default=45, ge=10)
    summary: str = Field(description="Short summary of learning unit.")
    deep_dive_markdown: str = Field(
        min_length=400,
        description="Comprehensive in-depth technical explanation (at least 400 characters) covering architecture mechanics, code patterns, trade-offs, and failure modes.",
    )
    evidence_ids: list[str] = Field(
        min_length=1,
        description="At least 1 evidence ID from input cited as provenance for this learning unit.",
    )


class CurriculumOutput(BaseModel):
    skills: list[SkillNodeItem] = Field(
        min_length=12,
        description="At least 12 interconnected skill nodes for the knowledge graph.",
    )
    relations: list[SkillEdgeItem] = Field(
        min_length=10,
        description="At least 10 directed relationships between skills.",
    )
    learning_units: list[LearningUnitItem] = Field(
        min_length=6,
        description="At least 6 prerequisite-ordered learning units with deep dive content.",
    )
    summary: str = Field(description="Summary of knowledge graph and curriculum path.")


class CurriculumGraphOutput(BaseModel):
    skills: list[SkillNodeItem] = Field(
        min_length=12,
        description="At least 12 interconnected skill nodes for the knowledge graph.",
    )
    relations: list[SkillEdgeItem] = Field(
        min_length=10,
        description="At least 10 directed relationships between skills.",
    )
    summary: str = Field(description="Summary of the knowledge graph.")


class LearningUnitBatchOutput(BaseModel):
    learning_units: list[LearningUnitItem] = Field(
        min_length=1,
        description="One prerequisite-aware learning unit per requested target skill.",
    )
    summary: str = Field(description="Summary of this batch of learning units.")


CURRICULUM_STAGE_SCHEMAS: dict[str, type[BaseModel]] = {
    "graph": CurriculumGraphOutput,
    "units": LearningUnitBatchOutput,
}


class PlanTaskItem(BaseModel):
    id: str = Field(description="Unique task identifier, e.g. task_research_1.")
    name: str = Field(description="Action-oriented task title.")
    assigned_agent: str = Field(
        description="Registered agent id responsible for this task: company-research | job-analysis | technical-research."
    )
    dependencies: list[str] = Field(
        default_factory=list,
        description="IDs of prerequisite tasks that must complete before this task executes.",
    )
    required: bool = Field(
        description="True if the run must fail when this task fails after retries; False if the run may continue with partial research. job-analysis must be required.",
    )


class PlanOutput(BaseModel):
    tasks: list[PlanTaskItem] = Field(
        min_length=1,
        max_length=15,
        description="Executable DAG tasks with strict dependencies and bounded depth.",
    )
    summary: str = Field(description="Strategic summary of the orchestration plan.")


AGENT_OUTPUT_SCHEMAS: dict[str, type[BaseModel]] = {
    "planner": PlanOutput,
    "company-research": CompanyResearchOutput,
    "job-analysis": JobAnalysisOutput,
    "technical-research": TechnicalResearchOutput,
    "interview-generation": InterviewGenerationOutput,
    "evidence-evaluator": EvidenceEvaluatorOutput,
    "curriculum": CurriculumOutput,
    "systems-deep-dive": SystemsDeepDiveOutput,
}
