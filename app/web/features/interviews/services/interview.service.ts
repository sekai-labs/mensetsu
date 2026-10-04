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
} from '../models/interview.model'

/**
 * Service for interview workspaces, match analysis, company dossiers,
 * role expectations, knowledge graphs, learning units, and question banks.
 */
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

  return {
    cleanWorkspaceId,
    createWorkspace,
    listWorkspaces,
    getWorkspace,
    getWorkspaceRole,
    getWorkspaceMatch,
    getWorkspaceCompany,
    getWorkspaceKnowledge,
    getWorkspaceLearning,
    toggleLearningUnitCompleted,
    getWorkspaceQuestions,
  }
}
