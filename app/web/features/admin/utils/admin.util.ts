import type {
  AdminPipelineLog,
  AdminPipelinePhaseStep,
  LogLevel,
  StepState,
} from '../models/admin.model'

export const DEFAULT_PIPELINE_STEPS: readonly AdminPipelinePhaseStep[] = [
  { id: 'preparing_input', name: 'Input Preparation' },
  { id: 'analyzing_resume', name: 'Resume Entity Extraction' },
  { id: 'reading_job_description', name: 'Job Description Parsing' },
  { id: 'researching_company', name: 'Deep Company Research' },
  { id: 'mapping_resume_evidence', name: 'Match & Gap Analysis' },
  { id: 'constructing_knowledge_graph', name: 'Knowledge Graph DAG' },
  { id: 'generating_lessons', name: 'Curriculum Generation' },
  { id: 'building_interview_question_book', name: 'Question Book Generation' },
] as const

/**
 * Returns Tailwind CSS classes for pipeline status representation.
 */
export const getPipelineStatusStyle = (status: string): string => {
  switch (status?.toLowerCase()) {
    case 'ready':
      return 'bg-emerald-50 text-emerald-700 dark:bg-emerald-950/50 dark:text-emerald-300 border-emerald-200 dark:border-emerald-800'
    case 'processing':
      return 'bg-indigo-50 text-indigo-700 dark:bg-indigo-950/50 dark:text-indigo-300 border-indigo-200 dark:border-indigo-800 animate-pulse'
    case 'failed':
      return 'bg-rose-50 text-rose-700 dark:bg-rose-950/50 dark:text-rose-300 border-rose-200 dark:border-rose-800'
    default:
      return 'bg-slate-100 text-slate-700 dark:bg-slate-800 dark:text-slate-300 border-slate-200 dark:border-slate-700'
  }
}

/**
 * Returns Tailwind CSS badge classes for log levels in terminal stream.
 */
export const getLogLevelBadgeStyle = (level?: LogLevel | string): string => {
  switch (level?.toLowerCase()) {
    case 'error':
      return 'bg-rose-950 text-rose-300 border-rose-800'
    case 'warn':
    case 'warning':
      return 'bg-amber-950 text-amber-300 border-amber-800'
    case 'info':
      return 'bg-blue-950 text-blue-300 border-blue-800'
    case 'debug':
      return 'bg-slate-800 text-slate-400 border-slate-700'
    default:
      return 'bg-slate-800 text-slate-400 border-slate-700'
  }
}

/**
 * Calculates execution state (completed, active, failed, pending) for a step in the pipeline.
 */
export const calculateStepState = (
  stepId: string,
  currentPhase: string,
  status: string,
  steps: readonly AdminPipelinePhaseStep[] = DEFAULT_PIPELINE_STEPS
): StepState => {
  const normalizedStatus = status?.toLowerCase()
  if (normalizedStatus === 'ready') {
    return 'completed'
  }

  if (normalizedStatus === 'failed' && currentPhase === stepId) {
    return 'failed'
  }

  const stepIdx = steps.findIndex((s) => s.id === stepId)
  const currentIdx = steps.findIndex((s) => s.id === currentPhase)

  if (stepIdx === -1 || currentIdx === -1) {
    return 'pending'
  }

  if (stepIdx < currentIdx) {
    return 'completed'
  }

  if (stepIdx === currentIdx) {
    return normalizedStatus === 'failed' ? 'failed' : 'active'
  }

  return 'pending'
}

/**
 * Returns styling classes for a step card in the stepper based on its execution state.
 */
export const getStepCardStyle = (state: StepState): string => {
  switch (state) {
    case 'completed':
      return 'border-emerald-300 dark:border-emerald-800 bg-emerald-50/50 dark:bg-emerald-950/30 text-emerald-800 dark:text-emerald-300'
    case 'active':
      return 'border-indigo-500 bg-indigo-50/60 dark:bg-indigo-950/50 text-indigo-700 dark:text-indigo-300 font-bold ring-1 ring-indigo-500'
    case 'failed':
      return 'border-rose-300 dark:border-rose-800 bg-rose-50 dark:bg-rose-950/40 text-rose-700 dark:text-rose-300'
    case 'pending':
    default:
      return 'border-slate-200 dark:border-slate-800 bg-slate-50/40 dark:bg-slate-850/40 text-slate-400'
  }
}

/**
 * Formats ISO timestamp to HH:mm:ss.SSS for display in the terminal log stream.
 */
export const formatLogTimestamp = (timestamp: string): string => {
  if (!timestamp) return ''
  const tIndex = timestamp.indexOf('T')
  if (tIndex !== -1) {
    return timestamp.slice(tIndex + 1).replace('Z', '')
  }
  return timestamp
}

/**
 * Filters a list of pipeline logs by keyword and log level.
 */
export const filterPipelineLogs = (
  logs: readonly AdminPipelineLog[],
  search: string,
  level: string
): AdminPipelineLog[] => {
  const query = search.trim().toLowerCase()
  const filterLevel = level.trim().toLowerCase()

  return logs.filter((log) => {
    const matchesSearch =
      !query ||
      log.message.toLowerCase().includes(query) ||
      log.step.toLowerCase().includes(query)

    const matchesLevel =
      filterLevel === 'all' ||
      !filterLevel ||
      log.level.toLowerCase() === filterLevel

    return matchesSearch && matchesLevel
  })
}

/**
 * Serializes log collection into human-readable raw terminal text output for file export.
 */
export const formatLogsForExport = (
  logs: readonly AdminPipelineLog[],
  workspaceId: string
): string => {
  const header = `# Pipeline Execution Log Export\n# Workspace ID: ${workspaceId}\n# Exported At: ${new Date().toISOString()}\n# Total Records: ${logs.length}\n${'='.repeat(80)}\n`
  const body = logs
    .map((log) => {
      const lvl = (log.level || 'INFO').toUpperCase().padEnd(5)
      const stp = log.step || 'general'
      return `[${log.timestamp}] [${lvl}] [${stp}]: ${log.message}`
    })
    .join('\n')

  return `${header}\n${body}\n`
}
