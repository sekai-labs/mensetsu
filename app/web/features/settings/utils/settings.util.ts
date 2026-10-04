import type { LlmProviderInfo, UpsertLlmConfigRequest } from '../models/settings.model'

export const KNOWN_LLM_PROVIDERS: readonly LlmProviderInfo[] = [
  {
    id: 'openai',
    name: 'OpenAI',
    description: 'Direct OpenAI API integration (GPT-4o, o1, embeddings)',
    defaultBaseUrl: 'https://api.openai.com/v1',
    defaultChatModel: 'gpt-4o',
    defaultSearchModel: 'gpt-4o-mini',
    defaultEmbedModel: 'text-embedding-3-small',
    docsUrl: 'https://platform.openai.com/docs',
  },
  {
    id: 'anthropic',
    name: 'Anthropic',
    description: 'Anthropic Claude model suite (Claude 3.7 Sonnet, Claude 3.5 Haiku)',
    defaultBaseUrl: 'https://api.anthropic.com/v1',
    defaultChatModel: 'claude-3-7-sonnet',
    defaultSearchModel: 'claude-3-5-haiku',
    docsUrl: 'https://docs.anthropic.com',
  },
  {
    id: 'openrouter',
    name: 'OpenRouter',
    description: 'Unified multi-provider API router with model fallback and unified billing',
    defaultBaseUrl: 'https://openrouter.ai/api/v1',
    defaultChatModel: 'openai/gpt-4o',
    defaultSearchModel: 'anthropic/claude-3.5-haiku',
    docsUrl: 'https://openrouter.ai/docs',
  },
  {
    id: 'ollama',
    name: 'Ollama (Local)',
    description: 'Locally hosted open-weights models (Llama 3, Qwen 2.5, DeepSeek R1)',
    defaultBaseUrl: 'http://localhost:11434/v1',
    defaultChatModel: 'llama3.3',
    defaultSearchModel: 'llama3.2',
    defaultEmbedModel: 'nomic-embed-text',
    docsUrl: 'https://ollama.com',
  },
  {
    id: 'custom',
    name: 'Custom OpenAI-Compatible',
    description: 'Self-hosted vLLM, TGI, LocalAI, or custom inference proxy endpoints',
    defaultBaseUrl: 'http://localhost:8000/v1',
  },
] as const

/**
 * Resolves metadata and defaults for a given provider id.
 */
export const resolveProviderMetadata = (providerId: string): LlmProviderInfo => {
  const normalized = (providerId || '').toLowerCase().trim()
  const found = KNOWN_LLM_PROVIDERS.find(p => p.id === normalized)
  if (found) return found

  const formattedName = normalized
    ? normalized.charAt(0).toUpperCase() + normalized.slice(1)
    : 'Unknown'

  return {
    id: normalized,
    name: formattedName,
    description: `${formattedName} provider configuration`,
  }
}

/**
 * Masks an API key for safe display, revealing only prefix and suffix.
 */
export const maskApiKey = (rawKey: string): string => {
  if (!rawKey || typeof rawKey !== 'string') return ''
  const trimmed = rawKey.trim()
  if (trimmed.length <= 8) {
    return '••••••••'
  }
  const prefix = trimmed.slice(0, 4)
  const suffix = trimmed.slice(-4)
  return `${prefix}••••••••${suffix}`
}

export interface ValidationIssue {
  field: string
  message: string
}

export interface ConfigValidationResult {
  valid: boolean
  errors: ValidationIssue[]
}

/**
 * Validates LLM provider form input before dispatching update requests.
 */
export const validateLlmConfigInput = (config: UpsertLlmConfigRequest): ConfigValidationResult => {
  const errors: ValidationIssue[] = []

  if (!config.provider || !config.provider.trim()) {
    errors.push({ field: 'provider', message: 'Provider identifier is required' })
  }

  if (config.base_url && config.base_url.trim()) {
    const url = config.base_url.trim()
    const isValidHttp = /^https?:\/\/.+/i.test(url)
    if (!isValidHttp) {
      errors.push({
        field: 'base_url',
        message: 'Base URL must start with http:// or https://',
      })
    }
  }

  return {
    valid: errors.length === 0,
    errors,
  }
}
