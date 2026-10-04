import type {
  WorkspaceResponse,
  InterviewQuestionResponse,
  LearningUnitResponse,
} from '../models/interview.model'

export interface QuestionFilterCriteria {
  searchQuery?: string
  category?: string
  difficulty?: string
}

export interface LearningProgressSummary {
  completedCount: number
  totalCount: number
  percentage: number
  totalMinutes: number
}

export interface WorkspaceCounts {
  total: number
  ready: number
  active: number
  failed: number
}

/**
 * Filters a list of workspaces by text query against company name and role title,
 * and optionally narrows by workspace status.
 */
export const filterWorkspaces = (
  workspaces: WorkspaceResponse[],
  query = '',
  statusFilter = 'all'
): WorkspaceResponse[] => {
  const normalizedQuery = query.trim().toLowerCase()
  const normalizedStatus = statusFilter.trim().toLowerCase()

  return workspaces.filter((ws) => {
    const matchesSearch =
      !normalizedQuery ||
      ws.company_name.toLowerCase().includes(normalizedQuery) ||
      ws.role_title.toLowerCase().includes(normalizedQuery)

    const matchesStatus =
      normalizedStatus === 'all' ||
      !normalizedStatus ||
      ws.status.toLowerCase() === normalizedStatus

    return matchesSearch && matchesStatus
  })
}

/**
 * Computes high-level aggregated metrics across workspaces.
 */
export const calculateWorkspaceStatistics = (workspaces: WorkspaceResponse[]): WorkspaceCounts => {
  let ready = 0
  let active = 0
  let failed = 0

  for (const ws of workspaces) {
    if (ws.status === 'ready') {
      ready += 1
    } else if (ws.status === 'processing' || ws.status === 'pending') {
      active += 1
    } else if (ws.status === 'failed') {
      failed += 1
    }
  }

  return {
    total: workspaces.length,
    ready,
    active,
    failed,
  }
}

/**
 * Groups interview questions into bucketed lists keyed by category.
 */
export const groupQuestionsByCategory = (
  questions: InterviewQuestionResponse[]
): Record<string, InterviewQuestionResponse[]> => {
  const grouped: Record<string, InterviewQuestionResponse[]> = {}

  for (const question of questions) {
    const category = question.category || 'general'
    if (!grouped[category]) {
      grouped[category] = []
    }
    grouped[category].push(question)
  }

  return grouped
}

/**
 * Filters interview questions matching optional search terms, category, and difficulty level.
 */
export const filterQuestions = (
  questions: InterviewQuestionResponse[],
  criteria: QuestionFilterCriteria
): InterviewQuestionResponse[] => {
  const query = criteria.searchQuery?.trim().toLowerCase() || ''
  const category = criteria.category?.trim().toLowerCase() || 'all'
  const difficulty = criteria.difficulty?.trim().toLowerCase() || 'all'

  return questions.filter((item) => {
    const matchesQuery =
      !query ||
      item.question.toLowerCase().includes(query) ||
      (item.skill_tested && item.skill_tested.toLowerCase().includes(query)) ||
      (item.skills_tested && item.skills_tested.some((s) => s.toLowerCase().includes(query)))

    const matchesCategory = category === 'all' || item.category?.toLowerCase() === category
    const matchesDifficulty =
      difficulty === 'all' || item.difficulty?.toLowerCase() === difficulty

    return matchesQuery && matchesCategory && matchesDifficulty
  })
}

/**
 * Calculates curriculum completion metrics, counts, and total estimated duration.
 */
export const calculateLearningProgress = (
  units: LearningUnitResponse[]
): LearningProgressSummary => {
  if (!units || units.length === 0) {
    return {
      completedCount: 0,
      totalCount: 0,
      percentage: 0,
      totalMinutes: 0,
    }
  }

  let completedCount = 0
  let totalMinutes = 0

  for (const unit of units) {
    if (unit.completed) {
      completedCount += 1
    }
    totalMinutes += unit.estimated_minutes || 0
  }

  const percentage = Math.round((completedCount / units.length) * 100)

  return {
    completedCount,
    totalCount: units.length,
    percentage,
    totalMinutes,
  }
}
