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

  const apiBase = getBackendApiBase()
  try {
    return await $fetch<{
      secret: string
      qr_code_data_url: string
      otpauth_url: string
    }>(`${apiBase}/api/v1/auth/2fa/setup`, {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${session.accessToken}`,
      },
    })
  } catch (err: unknown) {
    const errorObj = err as { statusCode?: number; data?: { error?: { message?: string } } }
    throw createError({
      statusCode: errorObj.statusCode || 500,
      message: errorObj.data?.error?.message || 'Failed to initiate 2FA setup',
    })
  }
})
