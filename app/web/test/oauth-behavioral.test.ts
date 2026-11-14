import { describe, it, expect } from 'vitest'
import { sanitizeRedirectPath } from '../features/auth/utils/auth.util'

describe('OAuth2 / OIDC Behavioral Verification Suite', () => {

  describe('Post-auth Redirect Safety (Open Redirect Defense)', () => {
    it('accepts safe same-origin application destinations', () => {
      expect(sanitizeRedirectPath('/dashboard')).toBe('/dashboard')
      expect(sanitizeRedirectPath('/interviews/new')).toBe('/interviews/new')
      expect(sanitizeRedirectPath('/settings?tab=security')).toBe('/settings?tab=security')
      expect(sanitizeRedirectPath('/settings?tab=usage#daily')).toBe(
        '/settings?tab=usage#daily'
      )
    })

    it('rejects external authority targets and protocol escapes', () => {
      expect(sanitizeRedirectPath('https://attacker.example.com')).toBe('/dashboard')
      expect(sanitizeRedirectPath('http://attacker.example.com/oauth')).toBe('/dashboard')
      expect(sanitizeRedirectPath('//attacker.example.com')).toBe('/dashboard')
      expect(sanitizeRedirectPath('///attacker.example.com')).toBe('/dashboard')
      expect(sanitizeRedirectPath('////attacker.example.com')).toBe('/dashboard')
    })

    it('rejects backslash obfuscation and encoded traversal attempts', () => {
      expect(sanitizeRedirectPath('\\\\attacker.example.com')).toBe('/dashboard')
      expect(sanitizeRedirectPath('/\\attacker.example.com')).toBe('/dashboard')
      expect(sanitizeRedirectPath('/\\/\\attacker.example.com')).toBe('/dashboard')
      expect(sanitizeRedirectPath('/%5cattacker.example.com')).toBe('/dashboard')
      expect(sanitizeRedirectPath('/%2f%2fattacker.example.com')).toBe('/dashboard')
    })

    it('rejects javascript, data, and malformed non-string inputs', () => {
      expect(sanitizeRedirectPath('javascript:alert(document.cookie)')).toBe('/dashboard')
      expect(sanitizeRedirectPath('data:text/html,<script>alert(1)</script>')).toBe('/dashboard')
      expect(sanitizeRedirectPath(null)).toBe('/dashboard')
      expect(sanitizeRedirectPath(undefined)).toBe('/dashboard')
      expect(sanitizeRedirectPath('')).toBe('/dashboard')
      expect(sanitizeRedirectPath('   ')).toBe('/dashboard')
    })
  })

})
