import type {
  WorkspaceResponse,
  InterviewQuestionResponse,
  LearningUnitResponse,
} from '../models/interview.model'

export const filterWorkspaces = (
  workspaces: WorkspaceResponse[],
  query: string,
  statusFilter: string
): WorkspaceResponse[] => {
  return workspaces.filter((ws) => {
    const matchesStatus = !statusFilter || statusFilter === 'all' || ws.status === statusFilter
    const matchesQuery =
      !query ||
      ws.company_name.toLowerCase().includes(query.toLowerCase()) ||
      ws.role_title.toLowerCase().includes(query.toLowerCase())
    return matchesStatus && matchesQuery
  })
}

export const calculateWorkspaceStatistics = (
  workspaces: WorkspaceResponse[]
): {
  total: number
  ready: number
  active: number
  failed: number
} => {
  return {
    total: workspaces.length,
    ready: workspaces.filter((w) => w.status === 'ready').length,
    active: workspaces.filter((w) => w.status === 'processing' || w.status === 'pending').length,
    failed: workspaces.filter((w) => w.status === 'failed').length,
  }
}

export const groupQuestionsByCategory = (
  questions: InterviewQuestionResponse[]
): Record<string, InterviewQuestionResponse[]> => {
  const result: Record<string, InterviewQuestionResponse[]> = {}
  for (const q of questions) {
    if (!result[q.category]) {
      result[q.category] = []
    }
    result[q.category].push(q)
  }
  return result
}

export interface QuestionFilterCriteria {
  searchQuery?: string
  difficulty?: string
  category?: string
}

export const filterQuestions = (
  questions: InterviewQuestionResponse[],
  criteria: QuestionFilterCriteria
): InterviewQuestionResponse[] => {
  const query = (criteria.searchQuery || '').toLowerCase()
  const diff = criteria.difficulty || 'all'
  const cat = criteria.category || 'all'

  return questions.filter((q) => {
    const matchesCat = cat === 'all' || q.category === cat
    const matchesDiff = diff === 'all' || q.difficulty === diff

    const skills = (q.skills_tested || []).join(' ').toLowerCase()
    const questionText = (q.question || '').toLowerCase()
    const trigger = (q.context_or_trigger || '').toLowerCase()
    const intent = (q.interviewer_intent || '').toLowerCase()

    const matchesQuery =
      !query ||
      questionText.includes(query) ||
      skills.includes(query) ||
      trigger.includes(query) ||
      intent.includes(query)

    return matchesCat && matchesDiff && matchesQuery
  })
}

export const calculateLearningProgress = (
  units: LearningUnitResponse[]
): {
  completedCount: number
  totalCount: number
  percentage: number
  totalMinutes: number
} => {
  if (units.length === 0) {
    return { completedCount: 0, totalCount: 0, percentage: 0, totalMinutes: 0 }
  }
  const completedCount = units.filter((u) => u.completed || u.is_completed).length
  const totalCount = units.length
  const totalMinutes = units.reduce((acc, u) => acc + (u.estimated_minutes || 0), 0)

  return {
    completedCount,
    totalCount,
    percentage: Math.round((completedCount / totalCount) * 100),
    totalMinutes,
  }
}
