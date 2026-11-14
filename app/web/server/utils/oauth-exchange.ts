import type { H3Event } from 'h3'
import { sessionManager } from '~/server/utils/session'
import {
  getBackendApiBase,
  SESSION_COOKIE_NAME,
  OAUTH_REDIRECT_COOKIE_NAME,
  TOTP_TEMP_TOKEN_COOKIE_NAME,
} from '~/server/utils/config'
import { sanitizeRedirectPath } from '~/features/auth/utils/auth.util'
import type { OAuthExchangeResponseDto } from '~/features/auth/models/auth.model'

export const handleOAuthCallback = async (event: H3Event) => {
  const query = getQuery(event)
  const rawCode = typeof query.code === 'string' ? query.code : ''
  const code = rawCode.trim() ? rawCode : ''
  const error = typeof query.error === 'string' ? query.error.trim() : ''
  const errorDescription =
    typeof query.error_description === 'string' ? query.error_description.trim() : ''

  const currentSessionId = getCookie(event, SESSION_COOKIE_NAME)
  const currentSession = currentSessionId
    ? await sessionManager.getSession(currentSessionId)
    : null
  const isAuthenticated = currentSession !== null

  const redirectTarget = sanitizeRedirectPath(getCookie(event, OAUTH_REDIRECT_COOKIE_NAME))
  deleteCookie(event, OAUTH_REDIRECT_COOKIE_NAME, { path: '/' })
  deleteCookie(event, TOTP_TEMP_TOKEN_COOKIE_NAME, { path: '/' })

  const redirectWithError = (errorCode: string) => {
    const params = new URLSearchParams()
    if (isAuthenticated) params.set('tab', 'security')
    params.set('oauth_error', errorCode)
    if (errorDescription) params.set('error_description', errorDescription)
    return sendRedirect(event, `${isAuthenticated ? '/settings' : '/login'}?${params}`, 302)
  }

  if (error || !code) {
    return redirectWithError(error || 'oauth_code_missing')
  }

  try {
    const exchange = await $fetch<OAuthExchangeResponseDto>(
      `${getBackendApiBase()}/api/v1/auth/oauth/exchange`,
      {
        method: 'POST',
        body: { code },
      }
    )

    if (exchange.linked_identity) {
      return sendRedirect(event, '/settings?tab=security&oauth_linked=true', 302)
    }

    if (exchange.requires_2fa) {
      if (!exchange.temp_token) {
        throw new Error('OAuth 2FA challenge is missing its temporary token')
      }
      setCookie(event, TOTP_TEMP_TOKEN_COOKIE_NAME, exchange.temp_token, {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax',
        path: '/',
        maxAge: 300,
      })
      return sendRedirect(
        event,
        `/login?requires_2fa=true&redirect=${encodeURIComponent(redirectTarget)}`,
        302
      )
    }

    if (!exchange.access_token || !exchange.refresh_token || !exchange.user) {
      throw new Error('OAuth exchange returned an incomplete session')
    }

    const newSessionId = sessionManager.generateTokenId()
    await sessionManager.saveSession(newSessionId, {
      userId: exchange.user.id,
      accessToken: exchange.access_token,
      refreshToken: exchange.refresh_token,
      createdAt: Date.now(),
    })

    setCookie(event, SESSION_COOKIE_NAME, newSessionId, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      path: '/',
      maxAge: 7 * 24 * 60 * 60,
    })

    return sendRedirect(event, redirectTarget, 302)
  } catch (err: unknown) {
    const errorObj = err as {
      statusCode?: number
      status?: number
      data?: { error?: { code?: string; message?: string } | string }
    }
    const status = errorObj.statusCode || errorObj.status
    let errorCode = 'oauth_failed'
    if (status === 400) {
      errorCode = 'oauth_invalid'
    } else if (status === 401) {
      errorCode = 'exchange_invalid'
    } else if (status === 403) {
      errorCode = 'account_exists'
    } else if (typeof errorObj.data === 'object' && errorObj.data && 'error' in errorObj.data) {
      const errField = errorObj.data.error
      if (typeof errField === 'object' && errField?.code) {
        errorCode = errField.code
      } else if (typeof errField === 'string' && errField.toLowerCase().includes('exists')) {
        errorCode = 'account_exists'
      }
    }

    return redirectWithError(errorCode)
  }
}
