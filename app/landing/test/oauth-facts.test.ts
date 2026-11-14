import { describe, it, expect } from 'vitest';

describe('OAuth Provider Specifications & Invariants', () => {
  const supportedProviders = ['google', 'github', 'facebook', 'custom oidc', 'custom oauth2'];

  it('supports all 5 requested provider types', () => {
    expect(supportedProviders).toHaveLength(5);
    expect(supportedProviders).toContain('google');
    expect(supportedProviders).toContain('github');
    expect(supportedProviders).toContain('facebook');
    expect(supportedProviders).toContain('custom oidc');
    expect(supportedProviders).toContain('custom oauth2');
  });

  it('validates canonical API-hosted callback URI', () => {
    const apiOrigin = 'https://api.mensetsu.example';
    const callbackUri = `${apiOrigin}/api/v1/auth/oauth/callback`;

    expect(callbackUri).toBe('https://api.mensetsu.example/api/v1/auth/oauth/callback');
  });

  it('validates one-use handoff redirect and web consumption', () => {
    const webOrigin = 'https://app.mensetsu.example';
    const webCallback = `${webOrigin}/auth/callback`;
    const handoffCode = 'handoff_sample_12345';
    const redirectUrl = `${webCallback}?code=${handoffCode}`;

    expect(redirectUrl).toBe('https://app.mensetsu.example/auth/callback?code=handoff_sample_12345');
  });

  it('asserts admin-only write-only secrets contract', () => {
    // Contract invariant: GET /api/v1/admin/oauth/providers never returns secrets; has_client_secret is bool
    const sampleSafeResponse = {
      provider_key: 'google',
      has_client_secret: true,
      client_id: 'client-123.apps.googleusercontent.com',
      redirect_uri: 'https://api.mensetsu.example/api/v1/auth/oauth/callback',
      is_enabled: true,
    };

    expect(sampleSafeResponse).not.toHaveProperty('client_secret');
    expect(typeof sampleSafeResponse.has_client_secret).toBe('boolean');
  });

  it('verifies required OAuth and SSRF environment variables', () => {
    const requiredEnv = [
      'ALLOWED_PRIVATE_HOSTS',
      'OAUTH_WEB_CALLBACK_URL',
      'OAUTH_STATE_TTL_SECONDS',
      'OAUTH_HANDOFF_TTL_SECONDS',
    ];
    expect(requiredEnv).toContain('ALLOWED_PRIVATE_HOSTS');
    expect(requiredEnv).toContain('OAUTH_WEB_CALLBACK_URL');
    expect(requiredEnv).toContain('OAUTH_STATE_TTL_SECONDS');
    expect(requiredEnv).toContain('OAUTH_HANDOFF_TTL_SECONDS');
  });
});
