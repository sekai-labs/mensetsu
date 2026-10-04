import { useApiClient } from '~/services/api.client'
import type {
  AdminPipelineDetail,
  AdminPipelineListResponse,
  AdminPipelineLogsResponse,
} from '../models/admin.model'

/**
 * Service for administrative operations on pipelines and execution logs.
 */
export const useAdminService = () => {
  const { request } = useApiClient()

  /**
   * Fetches list of all pipeline executions across workspaces.
   */
  const listAdminPipelines = async (): Promise<AdminPipelineListResponse> => {
    return await request<AdminPipelineListResponse>('/api/v1/admin/pipelines')
  }

  /**
   * Retrieves detailed status and workspace information for a specific pipeline.
   */
  const getAdminPipelineDetail = async (workspaceId: string): Promise<AdminPipelineDetail> => {
    const cleanId = workspaceId.replace(/^workspace:/, '').replace(/`/g, '')
    return await request<AdminPipelineDetail>(`/api/v1/admin/pipelines/${cleanId}`)
  }

  /**
   * Fetches recorded execution logs for a specific pipeline run.
   */
  const getAdminPipelineLogs = async (workspaceId: string): Promise<AdminPipelineLogsResponse> => {
    const cleanId = workspaceId.replace(/^workspace:/, '').replace(/`/g, '')
    return await request<AdminPipelineLogsResponse>(`/api/v1/admin/pipelines/${cleanId}/logs`)
  }

  return {
    listAdminPipelines,
    getAdminPipelineDetail,
    getAdminPipelineLogs,
  }
}
