import { describe, it, expect } from 'vitest'
import { resolvePostAuthRedirect, sanitizeRedirectPath } from '../features/auth/utils/auth.util'

describe('protected account routes and redirect safety', () => {
  it.each(['/cvs', '/profile', '/applications', '/settings?tab=usage'])(
    'redirects unauthenticated visits to %s through login',
    (path) => {
      expect(resolvePostAuthRedirect(false, path)).toBe(
        `/login?redirect=${encodeURIComponent(path)}`
      )
    }
  )

  it('defaults successful login to /dashboard when no explicit redirect is provided', () => {
    expect(resolvePostAuthRedirect(true, '/login', null)).toBe('/dashboard')
    expect(resolvePostAuthRedirect(true, '/login', undefined)).toBe('/dashboard')
    expect(resolvePostAuthRedirect(true, '/login', '')).toBe('/dashboard')
    expect(sanitizeRedirectPath(null)).toBe('/dashboard')
    expect(sanitizeRedirectPath('')).toBe('/dashboard')
    expect(sanitizeRedirectPath('   ')).toBe('/dashboard')
  })

  it('preserves valid explicit redirect destination upon login', () => {
    expect(resolvePostAuthRedirect(true, '/login', '/settings?tab=usage')).toBe('/settings?tab=usage')
    expect(resolvePostAuthRedirect(true, '/login', '/interviews/new')).toBe('/interviews/new')
    expect(sanitizeRedirectPath('/settings?tab=usage')).toBe('/settings?tab=usage')
    expect(sanitizeRedirectPath('/settings?tab=security#totp')).toBe('/settings?tab=security#totp')
  })

  it('rejects external, protocol-relative, backslash, or malformed redirect targets, defaulting to /dashboard', () => {
    // Absolute URLs
    expect(sanitizeRedirectPath('https://malicious-site.com')).toBe('/dashboard')
    expect(sanitizeRedirectPath('http://malicious-site.com')).toBe('/dashboard')
    expect(sanitizeRedirectPath('ftp://malicious-site.com')).toBe('/dashboard')

    // Protocol-relative URLs
    expect(sanitizeRedirectPath('//malicious-site.com')).toBe('/dashboard')
    expect(sanitizeRedirectPath('///malicious-site.com')).toBe('/dashboard')
    expect(sanitizeRedirectPath('////malicious-site.com')).toBe('/dashboard')

    // Backslash escapes
    expect(sanitizeRedirectPath('\\\\malicious-site.com')).toBe('/dashboard')
    expect(sanitizeRedirectPath('/\\malicious-site.com')).toBe('/dashboard')
    expect(sanitizeRedirectPath('/\\/\\malicious-site.com')).toBe('/dashboard')
    expect(sanitizeRedirectPath('/%5cmalicious-site.com')).toBe('/dashboard')
    expect(sanitizeRedirectPath('/%2f%2fmalicious-site.com')).toBe('/dashboard')

    // Dangerous pseudo-protocols
    expect(sanitizeRedirectPath('javascript:alert(1)')).toBe('/dashboard')
    expect(sanitizeRedirectPath('data:text/html,<script>alert(1)</script>')).toBe('/dashboard')
  })
})
