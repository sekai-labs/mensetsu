import { useApiClient } from '~/services/api.client'
import type {
  WorkspaceResponse,
  CreateWorkspaceRequest,
  MatchAnalysisResponse,
  CompanyResearchResponse,
  LearningUnitResponse,
  InterviewQuestionResponse,
  KnowledgeGraphResponse,
  WorkspaceRoleResponse,
  ToggleLessonResponse,
  SourceCitation,
} from '../models/interview.model'

export const useInterviewService = () => {
  const { request } = useApiClient()

  const cleanWorkspaceId = (id: string): string => {
    return id.replace(/^workspace:/, '').replace(/`/g, '')
  }

  const createWorkspace = async (payload: CreateWorkspaceRequest): Promise<WorkspaceResponse> => {
    return await request<WorkspaceResponse>('/api/v1/workspaces', {
      method: 'POST',
      body: payload,
    })
  }

  const listWorkspaces = async (): Promise<WorkspaceResponse[]> => {
    return await request<WorkspaceResponse[]>('/api/v1/workspaces')
  }

  const getWorkspace = async (id: string): Promise<WorkspaceResponse> => {
    const cleanId = cleanWorkspaceId(id)
    return await request<WorkspaceResponse>(`/api/v1/workspaces/${cleanId}`)
  }

  const updateWorkspaceObjective = async (
    id: string,
    prompt: string
  ): Promise<{ workspace_id: string; current_objective: string; status: string }> => {
    const cleanId = cleanWorkspaceId(id)
    return await request<{ workspace_id: string; current_objective: string; status: string }>(
      `/api/v1/workspaces/${cleanId}/objective`,
      {
        method: 'PATCH',
        body: { prompt },
      }
    )
  }

  const getWorkspaceRole = async (id: string): Promise<WorkspaceRoleResponse> => {
    const cleanId = cleanWorkspaceId(id)
    return await request<WorkspaceRoleResponse>(`/api/v1/workspaces/${cleanId}/role`)
  }

  const getWorkspaceMatch = async (id: string): Promise<MatchAnalysisResponse> => {
    const cleanId = cleanWorkspaceId(id)
    return await request<MatchAnalysisResponse>(`/api/v1/workspaces/${cleanId}/match`)
  }

  const getWorkspaceCompany = async (id: string): Promise<CompanyResearchResponse> => {
    const cleanId = cleanWorkspaceId(id)
    return await request<CompanyResearchResponse>(`/api/v1/workspaces/${cleanId}/company`)
  }

  const getWorkspaceKnowledge = async (id: string): Promise<KnowledgeGraphResponse> => {
    const cleanId = cleanWorkspaceId(id)
    return await request<KnowledgeGraphResponse>(`/api/v1/workspaces/${cleanId}/knowledge`)
  }

  const getWorkspaceLearning = async (id: string): Promise<LearningUnitResponse[]> => {
    const cleanId = cleanWorkspaceId(id)
    return await request<LearningUnitResponse[]>(`/api/v1/workspaces/${cleanId}/learning`)
  }

  const toggleLearningUnitCompleted = async (
    workspaceId: string,
    lessonId: string
  ): Promise<ToggleLessonResponse> => {
    const cleanWsId = cleanWorkspaceId(workspaceId)
    const cleanLid = lessonId.replace(/^learning_unit:/, '').replace(/`/g, '')
    return await request<ToggleLessonResponse>(
      `/api/v1/workspaces/${cleanWsId}/learning/${cleanLid}/toggle`,
      {
        method: 'PUT',
      }
    )
  }

  const getWorkspaceQuestions = async (id: string): Promise<InterviewQuestionResponse[]> => {
    const cleanId = cleanWorkspaceId(id)
    return await request<InterviewQuestionResponse[]>(`/api/v1/workspaces/${cleanId}/questions`)
  }

  const askWorkspaceKnowledgeQa = async (
    id: string,
    query: string
  ): Promise<{
    query: string
    answer: string
    cited_sources: SourceCitation[]
    cited_claims: Record<string, unknown>[]
    has_sufficient_evidence: boolean
  }> => {
    const cleanId = cleanWorkspaceId(id)
    return await request(`/api/v1/workspaces/${cleanId}/qa`, {
      method: 'POST',
      body: { query },
    })
  }

  const importJobDescriptionUrl = async (
    id: string,
    url: string,
    customTitle?: string
  ): Promise<{ workspace_id: string; status: string; source_url: string }> => {
    const cleanId = cleanWorkspaceId(id)
    return await request(`/api/v1/workspaces/${cleanId}/import-jd`, {
      method: 'POST',
      body: { url, custom_title: customTitle },
    })
  }

  const getWorkspaceSources = async (
    id: string
  ): Promise<{ workspace_id: string; sources: SourceCitation[] }> => {
    const cleanId = cleanWorkspaceId(id)
    return await request(`/api/v1/workspaces/${cleanId}/sources`)
  }

  const getWorkspaceClaims = async (
    id: string
  ): Promise<{ workspace_id: string; claims: Record<string, unknown>[] }> => {
    const cleanId = cleanWorkspaceId(id)
    return await request(`/api/v1/workspaces/${cleanId}/claims`)
  }

  return {
    cleanWorkspaceId,
    createWorkspace,
    listWorkspaces,
    getWorkspace,
    getWorkspaceRole,
    updateWorkspaceObjective,
    getWorkspaceMatch,
    getWorkspaceCompany,
    getWorkspaceKnowledge,
    getWorkspaceLearning,
    toggleLearningUnitCompleted,
    getWorkspaceQuestions,
    askWorkspaceKnowledgeQa,
    importJobDescriptionUrl,
    getWorkspaceSources,
    getWorkspaceClaims,
  }
}
