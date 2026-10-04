import { sessionManager } from '~/server/utils/session'
import { getBackendApiBase, SESSION_COOKIE_NAME } from '~/server/utils/config'

export default defineEventHandler(async (event) => {
  const body = await readBody(event)
  const apiBase = getBackendApiBase()

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
      body,
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
