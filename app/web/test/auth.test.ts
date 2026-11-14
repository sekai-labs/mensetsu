import { describe, it, expect } from 'vitest'
import {
  isPublicAuthRoute,
  resolvePostAuthRedirect,
} from '../features/auth/utils/auth.util'

describe('Auth Contracts and Composable logic', () => {


  it('validates authentication route access logic with auth.util', () => {
    expect(isPublicAuthRoute('/')).toBe(false)
    expect(isPublicAuthRoute('/login')).toBe(true)
    expect(isPublicAuthRoute('/register')).toBe(true)
    expect(isPublicAuthRoute('/dashboard')).toBe(false)
    expect(isPublicAuthRoute('/interviews/123/overview')).toBe(false)
    expect(isPublicAuthRoute('/settings')).toBe(false)
  })

  it('correctly redirects based on auth state using resolvePostAuthRedirect', () => {
    expect(resolvePostAuthRedirect(true, '/login')).toBe('/dashboard')
    expect(resolvePostAuthRedirect(true, '/register')).toBe('/dashboard')
    expect(resolvePostAuthRedirect(true, '/login', '/interviews')).toBe('/interviews')
    expect(resolvePostAuthRedirect(true, '/')).toBe('/dashboard')

    expect(resolvePostAuthRedirect(false, '/dashboard')).toBe('/login?redirect=%2Fdashboard')
    expect(resolvePostAuthRedirect(false, '/interviews/new')).toBe('/login?redirect=%2Finterviews%2Fnew')

    expect(resolvePostAuthRedirect(false, '/')).toBe('/login?redirect=%2F')
    expect(resolvePostAuthRedirect(false, '/login')).toBeNull()
    expect(resolvePostAuthRedirect(false, '/register')).toBeNull()
  })
})
