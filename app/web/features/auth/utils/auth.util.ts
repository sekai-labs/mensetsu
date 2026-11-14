export const isPublicAuthRoute = (path: string): boolean => {
  return path === '/login' || path === '/register'
}


/**
 * Sanitizes post-auth redirect paths to ensure same-origin relative navigation only.
 * Strictly rejects absolute URLs, protocol-relative targets (e.g. //evil.com, ///evil.com),
 * backslashes (including URL-encoded variants), and invalid URI constructs.
 */
export const sanitizeRedirectPath = (targetRedirect?: string | null): string => {
  if (!targetRedirect || typeof targetRedirect !== 'string') {
    return '/dashboard'
  }
  const trimmed = targetRedirect.trim()
  if (!trimmed) {
    return '/dashboard'
  }

  // Reject anything that doesn't start with a single forward slash or has backslashes
  if (!trimmed.startsWith('/') || trimmed.startsWith('//') || trimmed.includes('\\')) {
    return '/dashboard'
  }

  // Check URL-decoded representation against obfuscated slashes/backslashes
  try {
    const decoded = decodeURIComponent(trimmed)
    if (!decoded.startsWith('/') || decoded.startsWith('//') || decoded.includes('\\')) {
      return '/dashboard'
    }
  } catch {
    return '/dashboard'
  }

  // Parse against a fixed synthetic origin to verify no protocol or host escapes occur
  try {
    const dummyBase = 'https://mensetsu.local'
    const parsed = new URL(trimmed, dummyBase)
    if (parsed.origin !== dummyBase || parsed.username || parsed.password) {
      return '/dashboard'
    }
    if (!parsed.pathname.startsWith('/') || parsed.pathname.startsWith('//')) {
      return '/dashboard'
    }
    return `${parsed.pathname}${parsed.search}${parsed.hash}`
  } catch {
    return '/dashboard'
  }
}

export const resolvePostAuthRedirect = (
  isAuthenticated: boolean,
  currentPath: string,
  targetRedirect?: string | null
): string | null => {
  if (isAuthenticated) {
    if (currentPath === '/') {
      return '/dashboard'
    }
    if (currentPath === '/login' || currentPath === '/register') {
      return sanitizeRedirectPath(targetRedirect)
    }
    return null
  }

  if (isPublicAuthRoute(currentPath)) {
    return null
  }

  return `/login?redirect=${encodeURIComponent(currentPath)}`
}
