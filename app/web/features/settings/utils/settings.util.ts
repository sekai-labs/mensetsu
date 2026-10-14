import type { LlmProviderInfo } from '../models/settings.model'

export const KNOWN_LLM_PROVIDERS: LlmProviderInfo[] = [
  {
    id: 'openai',
    name: 'OpenAI',
    description: 'Direct access to GPT-4o, o1, and embeddings via official API.',
    defaultBaseUrl: 'https://api.openai.com/v1',
    defaultChatModel: 'gpt-4o',
    defaultSearchModel: 'gpt-4o-mini',
    defaultEmbedModel: 'text-embedding-3-small',
    docsUrl: 'https://platform.openai.com/docs',
  },
  {
    id: 'anthropic',
    name: 'Anthropic',
    description: 'Claude 3.5 Sonnet and Haiku models with deep technical reasoning.',
    defaultBaseUrl: 'https://api.anthropic.com/v1',
    defaultChatModel: 'claude-3-5-sonnet-latest',
    defaultSearchModel: 'claude-3-5-haiku-latest',
    docsUrl: 'https://docs.anthropic.com',
  },
  {
    id: 'gemini',
    name: 'Google Gemini',
    description: 'Gemini 1.5 Pro and Flash with large context windows.',
    defaultBaseUrl: 'https://generativelanguage.googleapis.com/v1beta',
    defaultChatModel: 'gemini-1.5-pro',
    defaultSearchModel: 'gemini-1.5-flash',
    docsUrl: 'https://ai.google.dev',
  },
  {
    id: 'groq',
    name: 'Groq',
    description: 'Ultra-low-latency LPU inference for Llama 3 models.',
    defaultBaseUrl: 'https://api.groq.com/openai/v1',
    defaultChatModel: 'llama-3.3-70b-versatile',
    defaultSearchModel: 'llama-3.1-8b-instant',
    docsUrl: 'https://groq.com',
  },
  {
    id: 'ollama',
    name: 'Ollama (Local)',
    description: 'Local models running on your machine or private network.',
    defaultBaseUrl: 'http://localhost:11434/v1',
    defaultChatModel: 'llama3.1:8b',
    defaultSearchModel: 'llama3.1:8b',
    defaultEmbedModel: 'nomic-embed-text',
    docsUrl: 'https://ollama.ai',
  },
]

export const maskApiKey = (key: string): string => {
  if (!key) return ''
  if (key.length <= 8) return '••••••••'
  return `${key.slice(0, 4)}••••${key.slice(-4)}`
}
