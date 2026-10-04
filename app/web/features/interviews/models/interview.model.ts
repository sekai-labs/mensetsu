import type {
  WorkspaceStatus,
  PipelinePhase,
  SeniorityLevel,
} from '~/models/common.model'

export type { WorkspaceStatus, PipelinePhase, SeniorityLevel }

export interface CreateWorkspaceRequest {
  company_name: string
  role_title: string
  job_description?: string
  jd_url?: string
  company_urls: string[]
  seniority?: SeniorityLevel
  target_date?: string
  prep_time_weeks?: number
  resume_text?: string
  resume_pdf_base64?: string
  resume_filename?: string
}

export interface WorkspaceResponse {
  id: string
  user_id: string
  company_name: string
  role_title: string
  status: WorkspaceStatus
  current_phase: PipelinePhase
  progress_percent: number
  prep_time_weeks?: number
  target_date?: string
  created_at: string
  updated_at: string
}

export interface StrengthEvidence {
  skill: string
  evidence_quote: string
  relevance: string
}

export interface GapEvidence {
  skill: string
  requirement_quote: string
  importance: 'mandatory' | 'preferred' | string
  recommended_focus: string
}

export interface WordingImprovement {
  original_text: string
  suggested_revision: string
  rationale: string
}

export interface MatchAnalysisResponse {
  workspace_id: string
  overall_match_ratio: number
  strengths: StrengthEvidence[]
  gaps: GapEvidence[]
  risks: string[]
  wording_improvements: WordingImprovement[]
}

export interface SourceCitation {
  id: string
  url: string
  title: string
  domain: string
  retrieved_at: string
  snippet: string
  confidence: number
  source_type?: string
}

export interface TechnologyEvidenceItem {
  domain: string
  tool_or_tech: string
  evidence_tier: string
  production_role: string
}

export interface ArchitectureTopology {
  edge_and_ingress?: string
  compute_and_orchestration?: string
  storage_and_consensus?: string
  observability_and_resilience?: string
}

export interface EvidenceMatrixItem {
  tier: 'FACT' | 'INFERENCE' | 'CANDIDATE' | string
  domain: string
  statement: string
  architectural_role?: string
}

export interface CompanyResearchResponse {
  workspace_id: string
  company_name: string
  overview: string
  business_model: string
  engineering_culture: string[]
  technology_stack: string[]
  recent_developments: string[]
  interview_process_insights: string[]
  bar_raiser_focus_areas?: string[]
  compliance_and_governance?: string[]
  operational_realities?: string[]
  evidence_matrix?: EvidenceMatrixItem[]
  architecture_topology?: ArchitectureTopology
  technology_evidence_map?: TechnologyEvidenceItem[]
  sources: SourceCitation[]
}

export interface LessonChapter {
  timestamp: string
  title: string
  summary?: string
}

export interface VisualDiagram {
  type: 'dependency_graph' | 'architecture_diagram' | 'data_flow' | 'comparison_matrix' | string
  caption: string
  data: Record<string, unknown>
}

export interface LearningUnitResponse {
  id: string
  workspace_id: string
  skill: string
  title: string
  prerequisites: string[]
  estimated_minutes: number
  summary: string
  content_markdown: string
  diagram?: VisualDiagram
  video_url?: string
  video_title?: string
  chapters?: LessonChapter[]
  completed?: boolean
  created_at: string
}

export type QuestionCategory =
  | 'resume_deep_dive'
  | 'project_deep_dive'
  | 'technical_fundamentals'
  | 'target_stack'
  | 'system_design'
  | 'debugging'
  | 'behavioral'

export type QuestionDifficulty = 'easy' | 'medium' | 'hard'

export interface InterviewQuestionResponse {
  id: string
  workspace_id: string
  category: QuestionCategory | string
  question: string
  difficulty: QuestionDifficulty | string
  skills_tested: string[]
  skill_tested?: string
  interviewer_intent: string
  answer_outline: string[]
  strong_answer_rubric: string[]
  common_weaknesses: string[]
  follow_up_questions: string[]
  created_at: string
}

export interface KnowledgeSkillNode {
  id: string
  name: string
  slug: string
  category: string
  candidate_status: 'mastered' | 'in_progress' | 'gap' | string
  importance: 'mandatory' | 'preferred' | string
}

export interface KnowledgeEdge {
  source: string
  target: string
  relation: string
}

export interface KnowledgeGraphResponse {
  nodes: KnowledgeSkillNode[]
  edges: KnowledgeEdge[]
}

export interface RoleRequirement {
  skill: string
  requirement_quote: string
  importance: 'mandatory' | 'preferred' | string
}

export interface WorkspaceRoleResponse {
  workspace_id: string
  title: string
  company_name: string
  role_summary: string
  raw_text?: string
  requirements?: RoleRequirement[]
  responsibilities?: string[]
}

export interface ProgressEventPayload {
  workspace_id: string
  phase: PipelinePhase | string
  percent: number
  status: WorkspaceStatus | string
  message: string
  timestamp: string
}

export interface ToggleLessonResponse {
  lesson_id: string
  completed: boolean
}
