import type { AdminPipelineLog, AdminPipelinePhaseStep, StepState } from '../models/admin.model'
import type { PipelinePhase, WorkspaceStatus } from '~/models/common.model'

export const DEFAULT_PIPELINE_STEPS: AdminPipelinePhaseStep[] = [
  { id: 'preparing_input', name: 'Preparing Input', description: 'Validating payload and extracting files' },
  { id: 'analyzing_resume', name: 'Analyzing Resume', description: 'Parsing candidate experience and claims' },
  { id: 'reading_job_description', name: 'Reading JD', description: 'Extracting role expectations' },
  { id: 'researching_company', name: 'Researching Company', description: 'Gathering company architecture and data' },
  { id: 'researching_engineering_environment', name: 'Engineering Env', description: 'Analyzing tech stack and practices' },
  { id: 'extracting_role_requirements', name: 'Role Requirements', description: 'Mapping core competencies' },
  { id: 'mapping_resume_evidence', name: 'Mapping Evidence', description: 'Aligning candidate proof points' },
  { id: 'constructing_knowledge_graph', name: 'Knowledge Graph', description: 'Building semantic skill graph' },
  { id: 'building_learning_plan', name: 'Learning Plan', description: 'Generating structured modules' },
  { id: 'generating_lessons', name: 'Generating Lessons', description: 'Synthesizing technical lessons' },
  { id: 'building_interview_question_book', name: 'Question Book', description: 'Creating targeted question bank' },
  { id: 'final_validation', name: 'Final Validation', description: 'Verifying evidence consistency' },
]

export const getPipelineStatusStyle = (status: WorkspaceStatus | string): string => {
  switch (status) {
    case 'ready':
      return 'bg-emerald-50 text-emerald-700 border-emerald-200 dark:bg-emerald-950/40 dark:text-emerald-400 dark:border-emerald-800'
    case 'processing':
      return 'bg-indigo-50 text-indigo-700 border-indigo-200 dark:bg-indigo-950/40 dark:text-indigo-400 dark:border-indigo-800'
    case 'failed':
      return 'bg-rose-50 text-rose-700 border-rose-200 dark:bg-rose-950/40 dark:text-rose-400 dark:border-rose-800'
    case 'pending':
    default:
      return 'bg-slate-50 text-slate-700 border-slate-200 dark:bg-slate-800/40 dark:text-slate-400 dark:border-slate-700'
  }
}

export const getLogLevelBadgeStyle = (level: string): string => {
  switch (level?.toLowerCase()) {
    case 'error':
      return 'bg-rose-50 text-rose-700 border-rose-200 dark:bg-rose-950/50 dark:text-rose-400 dark:border-rose-800'
    case 'warn':
      return 'bg-amber-50 text-amber-700 border-amber-200 dark:bg-amber-950/50 dark:text-amber-400 dark:border-amber-800'
    case 'info':
      return 'bg-blue-50 text-blue-700 border-blue-200 dark:bg-blue-950/50 dark:text-blue-400 dark:border-blue-800'
    case 'debug':
    default:
      return 'bg-slate-50 text-slate-600 border-slate-200 dark:bg-slate-800 dark:text-slate-400 dark:border-slate-700'
  }
}

export const calculateStepState = (
  stepId: string,
  currentPhase: PipelinePhase | string,
  status: WorkspaceStatus | string,
  steps: AdminPipelinePhaseStep[] = DEFAULT_PIPELINE_STEPS
): StepState => {
  if (status === 'ready') return 'completed'
  if (status === 'failed' && currentPhase === stepId) return 'failed'

  const currentIndex = steps.findIndex((s) => s.id === currentPhase)
  const stepIndex = steps.findIndex((s) => s.id === stepId)

  if (stepIndex < 0 || currentIndex < 0) return 'pending'
  if (stepIndex < currentIndex) return 'completed'
  if (stepIndex === currentIndex) return status === 'failed' ? 'failed' : 'active'
  return 'pending'
}

export const filterPipelineLogs = (
  logs: AdminPipelineLog[],
  query: string,
  levelFilter: string
): AdminPipelineLog[] => {
  return logs.filter((log) => {
    const matchesLevel = !levelFilter || levelFilter === 'all' || log.level.toLowerCase() === levelFilter.toLowerCase()
    const matchesQuery =
      !query ||
      log.message.toLowerCase().includes(query.toLowerCase()) ||
      log.step.toLowerCase().includes(query.toLowerCase())
    return matchesLevel && matchesQuery
  })
}

export const formatLogTimestamp = (iso: string): string => {
  if (iso.includes('T') && iso.endsWith('Z')) {
    const timePart = iso.split('T')[1]
    return timePart ? timePart.replace('Z', '') : iso
  }
  return iso
}

export const formatLogsForExport = (logs: AdminPipelineLog[], workspaceId?: string): string => {
  const header = workspaceId ? `Workspace ID: ${workspaceId}\n-----------------------------------\n` : ''
  const body = logs
    .map((l) => `[${l.timestamp}] [${l.level.toUpperCase()}] [${l.step}]: ${l.message}`)
    .join('\n')
  return header + body
}

export const getStepCardStyle = (state: StepState): string => {
  switch (state) {
    case 'completed':
      return 'border-emerald-500 bg-emerald-50/20 dark:bg-emerald-950/10'
    case 'active':
      return 'border-indigo-500 bg-indigo-50/20 dark:bg-indigo-950/10 ring-1 ring-indigo-500'
    case 'failed':
      return 'border-rose-500 bg-rose-50/20 dark:bg-rose-950/10'
    case 'pending':
    default:
      return 'border-slate-200 dark:border-slate-800 opacity-60'
  }
}
