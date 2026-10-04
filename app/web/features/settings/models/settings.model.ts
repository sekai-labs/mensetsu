export interface LlmConfigItem {
  provider: string
  base_url?: string
  model_chat?: string
  model_search?: string
  model_embed?: string
  is_shared: boolean
  has_key: boolean
  masked_key?: string
  is_inherited: boolean
}

export interface UpsertLlmConfigRequest {
  provider: string
  api_key?: string
  base_url?: string
  model_chat?: string
  model_search?: string
  model_embed?: string
  is_shared?: boolean
}

export interface LlmProviderInfo {
  id: string
  name: string
  description: string
  defaultBaseUrl?: string
  defaultChatModel?: string
  defaultSearchModel?: string
  defaultEmbedModel?: string
  docsUrl?: string
}
