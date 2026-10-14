import type {
  CreateWorkspaceRequest as ContractCreateWorkspaceRequest,
} from '@mensetsu/contracts'
import type {
  WorkspaceStatus,
  PipelinePhase,
  SeniorityLevel,
} from '~/models/common.model'

export type { WorkspaceStatus, PipelinePhase, SeniorityLevel }
export type CreateWorkspaceRequest = ContractCreateWorkspaceRequest

export interface WorkspaceResponse {
  id: string
  user_id: string
  company_name: string
  role_title: string
  seniority?: SeniorityLevel
  target_interview_date?: string
  status: WorkspaceStatus
  current_phase: PipelinePhase
  progress_percent: number
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
  importance: 'required' | 'preferred' | 'optional'
  recommended_focus: string
}

export interface WordingImprovement {
  original_bullet: string
  improved_bullet: string
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
  source_url: string
  title: string
  retrieved_at: string
  snippet: string
  confidence: number
}

export interface TechnologyEvidenceItem {
  name: string
  category: string
  verified: boolean
  source_citation_id?: string
  notes?: string
}

export interface ArchitectureTopology {
  system_type: string
  components: string[]
  data_stores: string[]
  consensus_or_sync_mechanism?: string
}

export interface EvidenceMatrixItem {
  tier: 'FACT' | 'INFERENCE' | 'CANDIDATE'
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
  operational_realities?: string[]
  tech_stack: TechnologyEvidenceItem[]
  architecture_topology?: ArchitectureTopology
  interview_process_notes: string[]
  sources: SourceCitation[]
  evidence_matrix?: EvidenceMatrixItem[]
}

export interface LessonChapter {
  id: string
  title: string
  content: string
}

export interface VisualDiagram {
  diagram_type: 'dependency_graph' | 'architecture_diagram' | 'data_flow' | 'comparison_matrix'
  caption: string
  data: Record<string, unknown>
}

export interface LearningUnitResponse {
  id: string
  workspace_id: string
  skill?: string
  title: string
  summary?: string
  order_index?: number
  prerequisites: string[]
  estimated_minutes: number
  core_concepts?: string[]
  deep_dive_markdown?: string
  content_markdown?: string
  chapters?: LessonChapter[]
  visual_diagram?: VisualDiagram
  is_completed?: boolean
  completed?: boolean
  created_at?: string
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
  category: QuestionCategory
  difficulty: QuestionDifficulty
  question: string
  context_or_trigger?: string
  skills_tested?: string[]
  interviewer_intent?: string
  answer_outline?: string | string[]
  strong_answer_rubric?: string[]
  common_weaknesses?: string[]
  common_pitfalls?: string[]
  evaluation_rubric?: string[]
  follow_up_questions?: string[]
  source_citation_id?: string
  created_at?: string
}

export interface KnowledgeSkillNode {
  id: string
  label: string
  category: 'language' | 'framework' | 'distributed_system' | 'database' | 'cloud' | 'architecture'
  importance: 'core' | 'supporting' | 'peripheral'
  is_matched: boolean
}

export interface KnowledgeEdge {
  source: string
  target: string
  relationship: 'requires' | 'implements' | 'runs_on' | 'complements'
}

export interface KnowledgeGraphResponse {
  workspace_id: string
  nodes: KnowledgeSkillNode[]
  edges: KnowledgeEdge[]
}

export interface RoleRequirement {
  category: string
  description: string
  is_mandatory: boolean
}

export interface WorkspaceRoleResponse {
  workspace_id: string
  title: string
  seniority: SeniorityLevel
  responsibilities: string[]
  requirements: RoleRequirement[]
}

export interface ProgressEventPayload {
  workspace_id: string
  phase: PipelinePhase
  progress_percent: number
  message?: string
}

export interface ToggleLessonResponse {
  lesson_id: string
  is_completed: boolean
}
