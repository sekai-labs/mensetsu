
export type WorkspaceStatus = 'pending' | 'processing' | 'ready' | 'failed';

export type PipelinePhase =
  | 'preparing_input'
  | 'analyzing_resume'
  | 'reading_job_description'
  | 'researching_company'
  | 'researching_engineering_environment'
  | 'extracting_role_requirements'
  | 'mapping_resume_evidence'
  | 'constructing_knowledge_graph'
  | 'building_learning_plan'
  | 'generating_lessons'
  | 'building_interview_question_book'
  | 'final_validation'
  | 'ready'
  | 'failed';

export type SeniorityLevel = 'junior' | 'mid' | 'senior' | 'staff' | 'principal' | 'lead';

export interface CreateWorkspaceRequest {
  company_name: string;
  role_title: string;
  job_description?: string;
  company_urls: string[];
  seniority?: SeniorityLevel;
  target_date?: string;
  prep_time_weeks?: number;
  resume_text?: string;
}

export interface WorkspaceResponse {
  id: string;
  user_id: string;
  company_name: string;
  role_title: string;
  status: WorkspaceStatus;
  current_phase: PipelinePhase;
  progress_percent: number;
  created_at: string;
  updated_at: string;
}

export interface WorkspaceProgressEvent {
  workspace_id: string;
  phase: PipelinePhase;
  percent: number;
  message: string;
  timestamp: string;
}

export interface MatchAnalysisResponse {
  workspace_id: string;
  overall_match_ratio: number;
  strengths: {
    skill: string;
    evidence_quote: string;
    relevance: string;
  }[];
  gaps: {
    skill: string;
    requirement_quote: string;
    importance: 'mandatory' | 'preferred';
    recommended_focus: string;
  }[];
  risks: string[];
  wording_improvements: {
    original_text: string;
    suggested_revision: string;
    rationale: string;
  }[];
}

export interface CompanyResearchResponse {
  workspace_id: string;
  company_name: string;
  overview: string;
  business_model: string;
  engineering_culture: string[];
  technology_stack: string[];
  recent_developments: string[];
  interview_process_insights: string[];
  sources: {
    id: string;
    url: string;
    title: string;
    domain: string;
    retrieved_at: string;
    snippet: string;
    confidence: number;
  }[];
}

export interface LearningUnitResponse {
  id: string;
  workspace_id: string;
  skill: string;
  title: string;
  prerequisites: string[];
  estimated_minutes: number;
  summary: string;
  content_markdown: string;
  diagram?: {
    type: 'dependency_graph' | 'architecture_diagram' | 'data_flow' | 'comparison_matrix';
    caption: string;
    data: Record<string, unknown>;
  };
}

export interface InterviewQuestionResponse {
  id: string;
  workspace_id: string;
  category:
    | 'resume_deep_dive'
    | 'project_deep_dive'
    | 'technical_fundamentals'
    | 'target_stack'
    | 'system_design'
    | 'debugging'
    | 'behavioral';
  question: string;
  difficulty: 'easy' | 'medium' | 'hard';
  skills_tested: string[];
  interviewer_intent: string;
  answer_outline: string[];
  strong_answer_rubric: string[];
  common_weaknesses: string[];
  follow_up_questions: string[];
}
