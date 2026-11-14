import { sessionManager } from '~/server/utils/session'
import { getBackendApiBase, SESSION_COOKIE_NAME, TOTP_TEMP_TOKEN_COOKIE_NAME } from '~/server/utils/config'

export default defineEventHandler(async (event) => {
  const body = await readBody(event)
  const apiBase = getBackendApiBase()

  const cookieTempToken = getCookie(event, TOTP_TEMP_TOKEN_COOKIE_NAME)
  const effectiveTempToken = body.temp_token || cookieTempToken

  if (!effectiveTempToken) {
    throw createError({
      statusCode: 400,
      message: '2FA session expired. Please sign in again.',
    })
  }

  try {
    const backendRes = await $fetch<{
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
      access_token: string
      refresh_token: string
    }>(`${apiBase}/api/v1/auth/login-2fa`, {
      method: 'POST',
      body: {
        temp_token: effectiveTempToken,
        code: body.code,
      },
    })

    const tokenId = sessionManager.generateTokenId()
    await sessionManager.saveSession(tokenId, {
      userId: backendRes.user.id,
      accessToken: backendRes.access_token,
      refreshToken: backendRes.refresh_token,
      createdAt: Date.now(),
    })

    setCookie(event, SESSION_COOKIE_NAME, tokenId, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      path: '/',
      maxAge: 7 * 24 * 60 * 60,
    })

    deleteCookie(event, TOTP_TEMP_TOKEN_COOKIE_NAME, { path: '/' })

    return {
      user: backendRes.user,
    }
  } catch (err: unknown) {
    const errorObj = err as { statusCode?: number; data?: { error?: { message?: string } } }
    throw createError({
      statusCode: errorObj.statusCode || 401,
      message: errorObj.data?.error?.message || 'Invalid 2FA code',
    })
  }
})
