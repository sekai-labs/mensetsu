import { useApiClient } from '~/services/api.client'
import type { LlmConfigItem, UpsertLlmConfigRequest } from '../models/settings.model'

/**
 * Service for managing user and organization LLM configurations.
 */
export const useSettingsService = () => {
  const { request } = useApiClient()

  /**
   * Retrieves list of all configured and available LLM providers.
   */
  const getLlmConfigs = async (): Promise<LlmConfigItem[]> => {
    const res = await request<LlmConfigItem[]>('/api/v1/settings/llm')
    return Array.isArray(res) ? res : []
  }

  /**
   * Saves or updates an LLM provider configuration.
   */
  const saveLlmConfig = async (payload: UpsertLlmConfigRequest): Promise<LlmConfigItem> => {
    return await request<LlmConfigItem>('/api/v1/settings/llm', {
      method: 'PUT',
      body: payload,
    })
  }

  /**
   * Deletes a specific LLM provider configuration.
   */
  const deleteLlmConfig = async (provider: string): Promise<void> => {
    await request<void>(`/api/v1/settings/llm/${encodeURIComponent(provider.toLowerCase())}`, {
      method: 'DELETE',
    })
  }

  return {
    getLlmConfigs,
    saveLlmConfig,
    deleteLlmConfig,
  }
}
