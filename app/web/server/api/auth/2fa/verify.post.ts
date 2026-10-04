import { sessionManager } from '~/server/utils/session'
import { getBackendApiBase, SESSION_COOKIE_NAME } from '~/server/utils/config'

export default defineEventHandler(async (event) => {
  const tokenId = getCookie(event, SESSION_COOKIE_NAME)
  if (!tokenId) {
    throw createError({
      statusCode: 401,
      message: 'Not authenticated',
    })
  }

  const session = await sessionManager.getSession(tokenId)
  if (!session) {
    deleteCookie(event, SESSION_COOKIE_NAME, { path: '/' })
    throw createError({
      statusCode: 401,
      message: 'Session expired',
    })
  }

  const body = await readBody(event)
  const apiBase = getBackendApiBase()

  try {
    return await $fetch(`${apiBase}/api/v1/auth/2fa/verify`, {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${session.accessToken}`,
      },
      body,
    })
  } catch (err: unknown) {
    const errorObj = err as { statusCode?: number; data?: { error?: { message?: string } } }
    throw createError({
      statusCode: errorObj.statusCode || 400,
      message: errorObj.data?.error?.message || 'Failed to verify 2FA code',
    })
  }
})
