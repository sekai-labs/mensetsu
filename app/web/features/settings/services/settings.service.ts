import { useApiClient } from '~/services/api.client'
import type { LlmConfigItem, UpsertLlmConfigRequest } from '../models/settings.model'

export const useSettingsService = () => {
  const { request } = useApiClient()

  const listLlmConfigs = async (): Promise<LlmConfigItem[]> => {
    return await request<LlmConfigItem[]>('/api/v1/settings/llm')
  }

  const upsertLlmConfig = async (payload: UpsertLlmConfigRequest): Promise<LlmConfigItem> => {
    return await request<LlmConfigItem>('/api/v1/settings/llm', {
      method: 'PUT',
      body: payload,
    })
  }

  return {
    listLlmConfigs,
    upsertLlmConfig,
  }
}
