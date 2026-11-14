import { sessionManager } from '~/server/utils/session'
import { getBackendApiBase, SESSION_COOKIE_NAME } from '~/server/utils/config'
import type { OAuthLinkStartResponseDto } from '~/features/auth/models/auth.model'

export default defineEventHandler(async (event) => {
  const provider = getRouterParam(event, 'provider')
  if (!provider) {
    throw createError({
      statusCode: 400,
      message: 'Provider is required',
    })
  }

  const tokenId = getCookie(event, SESSION_COOKIE_NAME)
  if (!tokenId) {
    throw createError({
      statusCode: 401,
      message: 'Authentication required to link account',
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

  const apiBase = getBackendApiBase()
  try {
    const res = await $fetch<OAuthLinkStartResponseDto>(
      `${apiBase}/api/v1/auth/oauth/link/${encodeURIComponent(provider)}`,
      {
        method: 'POST',
        headers: {
          Authorization: `Bearer ${session.accessToken}`,
        },
      }
    )

    return res
  } catch (err: unknown) {
    const errorObj = err as { statusCode?: number; data?: { error?: { message?: string } } }
    throw createError({
      statusCode: errorObj.statusCode || 400,
      message: errorObj.data?.error?.message || 'Failed to initiate account linking',
    })
  }
})
