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
      message: 'Session expired or invalidated',
    })
  }

  const apiBase = getBackendApiBase()

  try {
    const res = await $fetch<{
      user: {
        id: string
        email: string
        name: string
        role: string
        status?: string
        totp_enabled: boolean
        created_at: string
        updated_at: string
      }
    }>(`${apiBase}/api/v1/auth/me`, {
      headers: {
        Authorization: `Bearer ${session.accessToken}`,
      },
    })

    return res
  } catch (err: unknown) {
    const errorObj = err as { statusCode?: number }
    if (errorObj.statusCode === 401) {

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

        const retryRes = await $fetch<{
          user: {
            id: string
            email: string
            name: string
            role: string
            status?: string
            totp_enabled: boolean
            created_at: string
            updated_at: string
          }
        }>(`${apiBase}/api/v1/auth/me`, {
          headers: {
            Authorization: `Bearer ${refreshRes.access_token}`,
          },
        })

        return retryRes
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
      statusCode: 500,
      message: 'Failed to fetch current user',
    })
  }
})
