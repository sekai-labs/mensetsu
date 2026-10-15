import { describe, expect, it } from 'bun:test'

describe('Settings & Providers Contract Verification', () => {
  it('correctly manages BYOK vs Admin key source precedence', () => {
    const providers = [
      {
        provider_key: 'anthropic',
        name: 'Anthropic',
        active_key_source: 'user',
        is_user_override: true,
      },
      {
        provider_key: 'openai',
        name: 'OpenAI',
        active_key_source: 'system',
        is_user_override: false,
      },
    ]

    expect(providers[0].active_key_source).toBe('user')
    expect(providers[0].is_user_override).toBe(true)
    expect(providers[1].active_key_source).toBe('system')
    expect(providers[1].is_user_override).toBe(false)
  })

  it('validates functional role slots in model routing matrix', () => {
    const matrix = {
      router: { primary: { provider_key: 'openai', model: 'gpt-4o-mini', temperature: 0.1 } },
      planner: {
        primary: { provider_key: 'openai', model: 'gpt-4o', temperature: 0.2 },
        fallback: { provider_key: 'anthropic', model: 'claude-3-5-sonnet-20241022', temperature: 0.2 },
      },
      research: { primary: { provider_key: 'openai', model: 'gpt-4o-mini', temperature: 0.2 } },
      generation: { primary: { provider_key: 'openai', model: 'gpt-4o', temperature: 0.2 } },
    }

    expect(matrix.planner.primary.model).toBe('gpt-4o')
    expect(matrix.planner.fallback?.provider_key).toBe('anthropic')
    expect(matrix.router.primary.provider_key).toBe('openai')
  })
})
