export function isEventsPath(pathname: string): boolean {
  return (
    /^\/api\/v1\/workflows\/runs\/[^/]+\/events$/.test(pathname) ||
    /^\/api\/v1\/workspaces\/[^/]+\/events$/.test(pathname)
  )
}

export function buildProxyHeaders(options: {
  accessToken?: string | null
  contentType?: string
  accept?: string
  lastEventId?: string
}): Record<string, string> {
  const headers: Record<string, string> = {}
  if (options.accessToken) {
    headers.Authorization = `Bearer ${options.accessToken}`
  }
  if (options.contentType) {
    headers['content-type'] = options.contentType
  }
  if (options.accept) {
    headers.accept = options.accept
  }
  if (options.lastEventId) {
    headers['last-event-id'] = options.lastEventId
  }
  return headers
}

export function buildTargetUrl(apiBase: string, cleanPathname: string): string {
  return `${apiBase}${cleanPathname}`
}
