import { describe, it, expect } from 'vitest'
import type { User, AuthResponse } from '../features/auth/models/auth.model'
import {
  isPublicAuthRoute,
  isValidEmail,
  isValidPassword,
  validatePasswordStrength,
  resolvePostAuthRedirect,
} from '../features/auth/utils/auth.util'

describe('Auth Contracts and Composable logic', () => {
  const sampleUser: User = {
    id: 'user_123',
    email: 'engineer@mensetsu.ai',
    name: 'Ada Lovelace',
    role: 'member',
    status: 'active',
    totp_enabled: false,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  }

  it('correctly models AuthResponse contract', () => {
    const authRes: AuthResponse = {
      user: sampleUser,
      access_token: 'jwt.token.mock',
      refresh_token: 'opaque_refresh_token_mock',
    }

    expect(authRes.user?.id).toBe('user_123')
    expect(authRes.user?.email).toBe('engineer@mensetsu.ai')
    expect(authRes.user?.name).toBe('Ada Lovelace')
    expect(authRes.access_token).toBeDefined()
    expect(authRes.refresh_token).toBeDefined()
  })

  it('validates authentication route access logic with auth.util', () => {
    expect(isPublicAuthRoute('/')).toBe(true)
    expect(isPublicAuthRoute('/login')).toBe(true)
    expect(isPublicAuthRoute('/register')).toBe(true)
    expect(isPublicAuthRoute('/dashboard')).toBe(false)
    expect(isPublicAuthRoute('/interviews/123/overview')).toBe(false)
    expect(isPublicAuthRoute('/settings')).toBe(false)
  })

  it('validates email and password logic with auth.util', () => {
    expect(isValidEmail('test@example.com')).toBe(true)
    expect(isValidEmail('not-an-email')).toBe(false)
    expect(isValidEmail('')).toBe(false)

    expect(isValidPassword('12345678', 8)).toBe(true)
    expect(isValidPassword('short', 8)).toBe(false)

    const strong = validatePasswordStrength('Secret123!', 8)
    expect(strong.valid).toBe(true)
    expect(strong.hasMinLength).toBe(true)
    expect(strong.hasLetter).toBe(true)
    expect(strong.hasNumberOrSpecial).toBe(true)
  })

  it('correctly redirects based on auth state using resolvePostAuthRedirect', () => {
    expect(resolvePostAuthRedirect(true, '/login')).toBe('/dashboard')
    expect(resolvePostAuthRedirect(true, '/register')).toBe('/dashboard')
    expect(resolvePostAuthRedirect(true, '/login', '/interviews')).toBe('/interviews')
    expect(resolvePostAuthRedirect(true, '/dashboard')).toBeNull()

    expect(resolvePostAuthRedirect(false, '/dashboard')).toBe('/login?redirect=%2Fdashboard')
    expect(resolvePostAuthRedirect(false, '/interviews/new')).toBe('/login?redirect=%2Finterviews%2Fnew')

    expect(resolvePostAuthRedirect(false, '/')).toBeNull()
    expect(resolvePostAuthRedirect(false, '/login')).toBeNull()
    expect(resolvePostAuthRedirect(false, '/register')).toBeNull()
  })
})
