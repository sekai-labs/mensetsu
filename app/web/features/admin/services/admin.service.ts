import { useApiClient } from '~/services/api.client'
import type {
  AdminPipelineListResponse,
  AdminPipelineDetail,
  AdminPipelineLogsResponse,
} from '../models/admin.model'

export const useAdminService = () => {
  const { request } = useApiClient()

  const listPipelines = async (): Promise<AdminPipelineListResponse> => {
    return await request<AdminPipelineListResponse>('/api/v1/admin/pipelines')
  }

  const getPipelineDetail = async (id: string): Promise<AdminPipelineDetail> => {
    const cleanId = id.replace(/^workspace:/, '').replace(/`/g, '')
    return await request<AdminPipelineDetail>(`/api/v1/admin/pipelines/${cleanId}`)
  }

  const getPipelineLogs = async (id: string): Promise<AdminPipelineLogsResponse> => {
    const cleanId = id.replace(/^workspace:/, '').replace(/`/g, '')
    return await request<AdminPipelineLogsResponse>(`/api/v1/admin/pipelines/${cleanId}/logs`)
  }

  return {
    listPipelines,
    getPipelineDetail,
    getPipelineLogs,
  }
}
