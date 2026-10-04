/**
 * Pure domain utilities for authentication.
 */

const PUBLIC_AUTH_ROUTES = ['/', '/login', '/register'] as const

/**
 * Checks whether a given route path is considered a public route that does not require authentication.
 */
export const isPublicAuthRoute = (path: string): boolean => {
  if (!path) return false
  const normalized = path.split('?')[0].split('#')[0]
  return (PUBLIC_AUTH_ROUTES as readonly string[]).includes(normalized)
}

/**
 * Validates email format according to RFC 5322 regex.
 */
export const isValidEmail = (email: string): boolean => {
  if (!email || typeof email !== 'string') return false
  const trimmed = email.trim()
  const emailRegex = /^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)+$/
  return emailRegex.test(trimmed)
}

export interface PasswordValidationResult {
  valid: boolean
  hasMinLength: boolean
  hasLetter: boolean
  hasNumberOrSpecial: boolean
}

/**
 * Validates password strength (minimum length default 8, or minLength param).
 */
export const isValidPassword = (password: string, minLength = 8): boolean => {
  if (!password || typeof password !== 'string') return false
  return password.length >= minLength
}

/**
 * Detailed password strength breakdown.
 */
export const validatePasswordStrength = (password: string, minLength = 8): PasswordValidationResult => {
  const hasMinLength = typeof password === 'string' && password.length >= minLength
  const hasLetter = /[a-zA-Z]/.test(password || '')
  const hasNumberOrSpecial = /[^a-zA-Z]/.test(password || '')

  return {
    valid: hasMinLength && hasLetter && hasNumberOrSpecial,
    hasMinLength,
    hasLetter,
    hasNumberOrSpecial,
  }
}

/**
 * Resolves post-auth redirect based on authentication state, destination, and fallback.
 */
export const resolvePostAuthRedirect = (
  isAuthenticated: boolean,
  currentPath: string,
  targetRedirect?: string | null
): string | null => {
  const isAuthPage = currentPath === '/login' || currentPath === '/register'

  if (isAuthenticated && isAuthPage) {
    if (targetRedirect && targetRedirect.startsWith('/') && !targetRedirect.startsWith('//')) {
      return targetRedirect
    }
    return '/dashboard'
  }

  if (!isAuthenticated && !isPublicAuthRoute(currentPath)) {
    return `/login?redirect=${encodeURIComponent(currentPath)}`
  }

  return null
}
