export const isPublicAuthRoute = (path: string): boolean => {
  return path === '/' || path === '/login' || path === '/register'
}

export const isValidEmail = (email: string): boolean => {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)
}

export const isValidPassword = (password: string, minLength = 8): boolean => {
  return password.length >= minLength
}

export const validatePasswordStrength = (
  password: string,
  minLength = 8
): {
  valid: boolean
  hasMinLength: boolean
  hasLetter: boolean
  hasNumberOrSpecial: boolean
} => {
  const hasMinLength = password.length >= minLength
  const hasLetter = /[A-Za-z]/.test(password)
  const hasNumberOrSpecial = /[^A-Za-z]/.test(password)
  return {
    valid: hasMinLength && hasLetter && hasNumberOrSpecial,
    hasMinLength,
    hasLetter,
    hasNumberOrSpecial,
  }
}

export const resolvePostAuthRedirect = (
  isAuthenticated: boolean,
  currentPath: string,
  targetRedirect?: string | null
): string | null => {
  if (isAuthenticated) {
    if (currentPath === '/login' || currentPath === '/register') {
      return targetRedirect || '/dashboard'
    }
    return null
  }

  if (isPublicAuthRoute(currentPath)) {
    return null
  }

  return `/login?redirect=${encodeURIComponent(currentPath)}`
}
