export type WorkspaceStatus = 'pending' | 'processing' | 'ready' | 'failed'

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
  | 'failed'

export type SeniorityLevel = 'junior' | 'mid' | 'senior' | 'staff' | 'principal' | 'lead'

export type UserRole = 'admin' | 'member'
