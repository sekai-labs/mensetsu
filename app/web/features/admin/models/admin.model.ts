import type { WorkspaceStatus, PipelinePhase } from '~/models/common.model'

export type LogLevel = 'info' | 'warn' | 'error' | 'debug'

export type StepState = 'completed' | 'active' | 'failed' | 'pending'

export interface AdminPipelineExecution {
  id?: string
  workspace_id?: string
  status?: WorkspaceStatus | string
  current_step?: string
  total_steps?: number
  completed_steps?: number
  error_message?: string | null
  started_at?: string
  completed_at?: string | null
}

export interface AdminPipelineItem {
  workspace_id: string
  company_name: string
  role_title: string
  status: WorkspaceStatus
  current_phase: PipelinePhase
  progress_percent: number
  created_at: string
  execution?: AdminPipelineExecution | null
}

export interface AdminWorkspaceDetail {
  id: string
  user_id: string
  company_name: string
  role_title: string
  status: WorkspaceStatus
  current_phase: PipelinePhase
  progress_percent: number
  created_at: string
  updated_at: string
  error_message?: string | null
}

export interface AdminPipelineDetail {
  workspace_id: string
  workspace: AdminWorkspaceDetail | null
  execution: AdminPipelineExecution | null
}

export interface AdminPipelineLog {
  id?: string
  workspace_id?: string
  execution_id?: string
  step: string
  level: LogLevel | string
  message: string
  details?: Record<string, unknown> | null
  timestamp: string
}

export interface AdminPipelineListResponse {
  pipelines: AdminPipelineItem[]
}

export interface AdminPipelineLogsResponse {
  workspace_id: string
  logs: AdminPipelineLog[]
}

export interface AdminPipelinePhaseStep {
  id: string
  name: string
  description?: string
}
