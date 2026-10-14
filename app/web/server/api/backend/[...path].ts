import { sessionManager } from '~/server/utils/session'
import { getBackendApiBase, SESSION_COOKIE_NAME } from '~/server/utils/config'
import { isEventsPath, buildProxyHeaders, buildTargetUrl } from '~/server/utils/routes'
import { refreshSessionToken } from '~/server/utils/auth-refresh'

export default defineEventHandler(async (event) => {
  const [pathnameRaw = ''] = event.path.split('?')
  const cleanPathname = pathnameRaw.replace(/^\/api\/backend\//, '/api/v1/')
  const apiBase = getBackendApiBase()
  const targetUrl = buildTargetUrl(apiBase, cleanPathname)

  const tokenId = getCookie(event, SESSION_COOKIE_NAME)
  const session = tokenId ? await sessionManager.getSession(tokenId) : null

  const method = event.method
  const headers = buildProxyHeaders({
    accessToken: session?.accessToken,
    contentType: getRequestHeader(event, 'content-type'),
    accept: getRequestHeader(event, 'accept'),
    lastEventId: getRequestHeader(event, 'last-event-id'),
  })

  let body: unknown = undefined
  if (method !== 'GET' && method !== 'HEAD') {
    try {
      body = await readBody(event)
    } catch {
      throw createError({
        statusCode: 400,
        message: 'Invalid JSON request payload',
      })
    }
  }

  if (isEventsPath(cleanPathname)) {
    const sseHeaders: Record<string, string> = {
      ...headers,
      accept: 'text/event-stream',
    }

    const abortController = new AbortController()
    event.node.req.on('close', () => {
      abortController.abort()
    })

    const upstreamUrl = `${targetUrl}?${event.path.split('?')[1] || ''}`.replace(/\?$/, '')

    let upstreamRes: Response
    try {
      upstreamRes = await fetch(upstreamUrl, {
        method: 'GET',
        headers: sseHeaders,
        signal: abortController.signal,
      })
    } catch (err: unknown) {
      if (abortController.signal.aborted) {
        return
      }
      throw createError({
        statusCode: 502,
        message: 'Failed to connect to upstream SSE stream',
      })
    }

    if (upstreamRes.status === 401 && session?.refreshToken && tokenId) {
      try {
        const newAccessToken = await refreshSessionToken(apiBase, tokenId, session)
        sseHeaders.Authorization = `Bearer ${newAccessToken}`
        upstreamRes = await fetch(upstreamUrl, {
          method: 'GET',
          headers: sseHeaders,
          signal: abortController.signal,
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

    if (!upstreamRes.ok) {
      throw createError({
        statusCode: upstreamRes.status,
        message: `Upstream SSE failed with status ${upstreamRes.status}`,
      })
    }

    setResponseStatus(event, 200)
    setResponseHeaders(event, {
      'content-type': 'text/event-stream',
      'cache-control': 'no-cache, no-transform',
      connection: 'keep-alive',
      'x-accel-buffering': 'no',
    })

    if (!upstreamRes.body) {
      return
    }
    try {
      return await sendStream(event, upstreamRes.body)
    } catch (err: unknown) {
      abortController.abort()
      if (
        (err && typeof err === 'object' && 'name' in err && err.name === 'AbortError') ||
        (err && typeof err === 'object' && 'code' in err && err.code === 'ERR_STREAM_PREMATURE_CLOSE')
      ) {
        return
      }
      throw err
    }
  }

  try {
    const upstreamRaw = await $fetch.raw(targetUrl, {
      method,
      headers,
      body: body as Record<string, unknown> | undefined,
      query: getQuery(event),
    })
    if (upstreamRaw.status) {
      setResponseStatus(event, upstreamRaw.status)
    }
    return upstreamRaw._data
  } catch (err: unknown) {
    const errorObj = err as {
      statusCode?: number
      data?: unknown
      status?: number
    }

    if ((errorObj.statusCode === 401 || errorObj.status === 401) && session?.refreshToken && tokenId) {
      try {
        const newAccessToken = await refreshSessionToken(apiBase, tokenId, session)
        headers.Authorization = `Bearer ${newAccessToken}`
        const retriedRaw = await $fetch.raw(targetUrl, {
          method,
          headers,
          body: body as Record<string, unknown> | undefined,
          query: getQuery(event),
        })
        if (retriedRaw.status) {
          setResponseStatus(event, retriedRaw.status)
        }
        return retriedRaw._data
      } catch {
        const latest = await sessionManager.getSession(tokenId)
        if (latest && latest.refreshToken !== session.refreshToken && latest.accessToken) {
          headers.Authorization = `Bearer ${latest.accessToken}`
          const retriedRaw = await $fetch.raw(targetUrl, {
            method,
            headers,
            body: body as Record<string, unknown> | undefined,
            query: getQuery(event),
          })
          if (retriedRaw.status) {
            setResponseStatus(event, retriedRaw.status)
          }
          return retriedRaw._data
        }
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
