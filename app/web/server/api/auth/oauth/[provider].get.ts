import { getBackendApiBase, OAUTH_REDIRECT_COOKIE_NAME } from '~/server/utils/config'
import { sanitizeRedirectPath } from '~/features/auth/utils/auth.util'

export default defineEventHandler((event) => {
  const provider = getRouterParam(event, 'provider')
  if (!provider) {
    throw createError({
      statusCode: 400,
      message: 'Provider is required',
    })
  }

  const query = getQuery(event)
  const redirectTarget = sanitizeRedirectPath(query.redirect as string | undefined)
  setCookie(event, OAUTH_REDIRECT_COOKIE_NAME, redirectTarget, {
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'lax',
    path: '/',
    maxAge: 600,
  })

  setResponseHeader(event, 'cache-control', 'no-store')
  return sendRedirect(
    event,
    `${getBackendApiBase()}/api/v1/auth/oauth/authorize/${encodeURIComponent(provider)}`,
    302
  )
})
