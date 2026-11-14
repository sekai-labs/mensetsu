import { useApiClient } from '~/services/api.client'
import type {
  WorkspaceResponse,
  CreateWorkspaceRequest,
  CreateResumeUploadRequestDto,
  CreateResumeUploadResponseDto,
  CompleteResumeUploadResponseDto,
  ResumeUploadItemDto,
  UserProfileResponseDto,
  CandidateProfileDto,
  UpdateCandidateProfileRequestDto,
  JobApplicationResponseDto,
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
  const createResumeUpload = async (
    payload: CreateResumeUploadRequestDto
  ): Promise<CreateResumeUploadResponseDto> => {
    return await request<CreateResumeUploadResponseDto>('/api/v1/resume-uploads', {
      method: 'POST',
      body: payload,
    })
  }

  const completeResumeUpload = async (
    uploadId: string
  ): Promise<CompleteResumeUploadResponseDto> => {
    const cleanUploadId = uploadId.replace(/^resume_upload:/, '').replace(/`/g, '')
    return await request<CompleteResumeUploadResponseDto>(
      `/api/v1/resume-uploads/${encodeURIComponent(cleanUploadId)}/complete`,
      {
        method: 'POST',
      }
    )
  }
  const uploadResumeFileToSignedUrl = async (
    uploadUrl: string,
    file: File | Blob
  ): Promise<void> => {
    const res = await fetch(uploadUrl, {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/pdf',
      },
      body: file,
      credentials: 'omit',
      redirect: 'error',
    })
    if (!res.ok) {
      throw new Error(`Direct storage upload failed with status ${res.status}`)
    }
  }

  const listWorkspaces = async (): Promise<WorkspaceResponse[]> => {
    return await request<WorkspaceResponse[]>('/api/v1/workspaces')
  }

  const getWorkspace = async (id: string): Promise<WorkspaceResponse> => {
    const cleanId = cleanWorkspaceId(id)
    return await request<WorkspaceResponse>(`/api/v1/workspaces/${encodeURIComponent(cleanId)}`)
  }

  const updateWorkspaceObjective = async (
    id: string,
    prompt: string
  ): Promise<{ workspace_id: string; current_objective: string; status: string }> => {
    const cleanId = cleanWorkspaceId(id)
    return await request<{ workspace_id: string; current_objective: string; status: string }>(
      `/api/v1/workspaces/${encodeURIComponent(cleanId)}/objective`,
      {
        method: 'PATCH',
        body: { prompt },
      }
    )
  }

  const getWorkspaceRole = async (id: string): Promise<WorkspaceRoleResponse> => {
    const cleanId = cleanWorkspaceId(id)
    return await request<WorkspaceRoleResponse>(`/api/v1/workspaces/${encodeURIComponent(cleanId)}/role`)
  }

  const getWorkspaceMatch = async (id: string): Promise<MatchAnalysisResponse> => {
    const cleanId = cleanWorkspaceId(id)
    return await request<MatchAnalysisResponse>(`/api/v1/workspaces/${encodeURIComponent(cleanId)}/match`)
  }

  const getWorkspaceCompany = async (id: string): Promise<CompanyResearchResponse> => {
    const cleanId = cleanWorkspaceId(id)
    return await request<CompanyResearchResponse>(`/api/v1/workspaces/${encodeURIComponent(cleanId)}/company`)
  }

  const getWorkspaceKnowledge = async (id: string): Promise<KnowledgeGraphResponse> => {
    const cleanId = cleanWorkspaceId(id)
    return await request<KnowledgeGraphResponse>(`/api/v1/workspaces/${encodeURIComponent(cleanId)}/knowledge`)
  }

  const getWorkspaceLearning = async (id: string): Promise<LearningUnitResponse[]> => {
    const cleanId = cleanWorkspaceId(id)
    return await request<LearningUnitResponse[]>(`/api/v1/workspaces/${encodeURIComponent(cleanId)}/learning`)
  }

  const toggleLearningUnitCompleted = async (
    workspaceId: string,
    lessonId: string
  ): Promise<ToggleLessonResponse> => {
    const cleanWsId = cleanWorkspaceId(workspaceId)
    const cleanLid = lessonId.replace(/^learning_unit:/, '').replace(/`/g, '')
    return await request<ToggleLessonResponse>(
      `/api/v1/workspaces/${encodeURIComponent(cleanWsId)}/learning/${encodeURIComponent(cleanLid)}/toggle`,
      {
        method: 'PUT',
      }
    )
  }

  const getWorkspaceQuestions = async (id: string): Promise<InterviewQuestionResponse[]> => {
    const cleanId = cleanWorkspaceId(id)
    return await request<InterviewQuestionResponse[]>(`/api/v1/workspaces/${encodeURIComponent(cleanId)}/questions`)
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
    return await request(`/api/v1/workspaces/${encodeURIComponent(cleanId)}/qa`, {
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
    return await request(`/api/v1/workspaces/${encodeURIComponent(cleanId)}/import-jd`, {
      method: 'POST',
      body: { url, custom_title: customTitle },
    })
  }

  const getWorkspaceSources = async (
    id: string
  ): Promise<{ workspace_id: string; sources: SourceCitation[] }> => {
    const cleanId = cleanWorkspaceId(id)
    return await request(`/api/v1/workspaces/${encodeURIComponent(cleanId)}/sources`)
  }

  const getWorkspaceClaims = async (
    id: string
  ): Promise<{ workspace_id: string; claims: Record<string, unknown>[] }> => {
    const cleanId = cleanWorkspaceId(id)
    return await request(`/api/v1/workspaces/${encodeURIComponent(cleanId)}/claims`)
  }

  const listResumeUploads = async (): Promise<ResumeUploadItemDto[]> => {
    return await request<ResumeUploadItemDto[]>('/api/v1/resume-uploads')
  }

  const deleteResumeUpload = async (uploadId: string): Promise<void> => {
    const cleanUploadId = uploadId.replace(/^resume_upload:/, '').replace(/`/g, '')
    await request<void>(`/api/v1/resume-uploads/${encodeURIComponent(cleanUploadId)}`, {
      method: 'DELETE',
    })
  }

  const getResumeDownloadUrl = (uploadId: string): string => {
    const cleanUploadId = uploadId.replace(/^resume_upload:/, '').replace(/`/g, '')
    return `/api/backend/resume-uploads/${encodeURIComponent(cleanUploadId)}/download`
  }

  const getUserProfile = async (): Promise<UserProfileResponseDto> => {
    return await request<UserProfileResponseDto>('/api/v1/settings/profile')
  }

  const getCandidateProfile = async (): Promise<CandidateProfileDto> => {
    return await request<CandidateProfileDto>('/api/v1/candidate-profile')
  }

  const updateCandidateProfile = async (
    payload: UpdateCandidateProfileRequestDto
  ): Promise<CandidateProfileDto> => {
    return await request<CandidateProfileDto>('/api/v1/candidate-profile', {
      method: 'PUT',
      body: payload,
    })
  }

  const listApplications = async (): Promise<JobApplicationResponseDto[]> => {
    return await request<JobApplicationResponseDto[]>('/api/v1/applications')
  }

  const getApplication = async (id: string): Promise<JobApplicationResponseDto> => {
    const cleanAppId = id.replace(/^job_application:/, '').replace(/`/g, '')
    return await request<JobApplicationResponseDto>(`/api/v1/applications/${encodeURIComponent(cleanAppId)}`)
  }

  const reconcileApplication = async (
    id: string,
    submitted: boolean
  ): Promise<JobApplicationResponseDto> => {
    const cleanAppId = id.replace(/^job_application:/, '').replace(/`/g, '')
    return await request<JobApplicationResponseDto>(`/api/v1/applications/${encodeURIComponent(cleanAppId)}/reconcile`, {
      method: 'POST',
      body: { submitted },
    })
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
    createResumeUpload,
    completeResumeUpload,
    uploadResumeFileToSignedUrl,
    listResumeUploads,
    deleteResumeUpload,
    getResumeDownloadUrl,
    getUserProfile,
    getCandidateProfile,
    updateCandidateProfile,
    listApplications,
    getApplication,
    reconcileApplication,
  }
}
