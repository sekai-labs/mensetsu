import { sessionManager } from '~/server/utils/session'
import { getBackendApiBase, SESSION_COOKIE_NAME } from '~/server/utils/config'

export default defineEventHandler(async (event) => {
  const path = event.path.replace(/^\/api\/backend\//, '/api/v1/')
  const apiBase = getBackendApiBase()
  const targetUrl = `${apiBase}${path}`

  const tokenId = getCookie(event, SESSION_COOKIE_NAME)
  const session = tokenId ? await sessionManager.getSession(tokenId) : null

  const method = event.method
  const headers: Record<string, string> = {}

  if (session?.accessToken) {
    headers.Authorization = `Bearer ${session.accessToken}`
  }

  const contentType = getRequestHeader(event, 'content-type')
  if (contentType) {
    headers['content-type'] = contentType
  }

  let body: unknown = undefined
  if (method !== 'GET' && method !== 'HEAD') {
    try {
      body = await readBody(event)
    } catch {
      // Body may be empty
    }
  }

  if (path.endsWith('/events')) {
    return proxyRequest(event, targetUrl, {
      headers,
    })
  }

  try {
    return await $fetch(targetUrl, {
      method,
      headers,
      body: body as Record<string, unknown> | undefined,
      query: getQuery(event),
    })
  } catch (err: unknown) {
    const errorObj = err as {
      statusCode?: number
      data?: unknown
      status?: number
    }

    // Handle token refresh if 401 and session exists
    if ((errorObj.statusCode === 401 || errorObj.status === 401) && session?.refreshToken && tokenId) {
      try {
        const refreshRes = await $fetch<{
          access_token: string
          refresh_token: string
        }>(`${apiBase}/api/v1/auth/refresh`, {
          method: 'POST',
          body: {
            refresh_token: session.refreshToken,
          },
        })

        await sessionManager.updateAccessToken(tokenId, refreshRes.access_token, refreshRes.refresh_token)

        headers.Authorization = `Bearer ${refreshRes.access_token}`
        return await $fetch(targetUrl, {
          method,
          headers,
          body: body as Record<string, unknown> | undefined,
          query: getQuery(event),
        })
      } catch {
        await sessionManager.deleteSession(tokenId)
        deleteCookie(event, SESSION_COOKIE_NAME, { path: '/' })
        throw createError({
          statusCode: 401,
          message: 'Session expired',
        })
      }
    }

    throw createError({
      statusCode: errorObj.statusCode || errorObj.status || 500,
      data: errorObj.data,
      message: 'Proxy request failed',
    })
  }
})
