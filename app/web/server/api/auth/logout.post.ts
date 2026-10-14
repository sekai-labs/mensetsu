import { sessionManager } from '~/server/utils/session'
import { getBackendApiBase, SESSION_COOKIE_NAME } from '~/server/utils/config'

export default defineEventHandler(async (event) => {
  const tokenId = getCookie(event, SESSION_COOKIE_NAME)
  const apiBase = getBackendApiBase()

  if (tokenId) {
    const session = await sessionManager.getSession(tokenId)
    if (session?.accessToken) {
      try {
        await $fetch(`${apiBase}/api/v1/auth/logout`, {
          method: 'POST',
          headers: {
            Authorization: `Bearer ${session.accessToken}`,
          },
        })
      } catch {

      }
    }
    await sessionManager.deleteSession(tokenId)
  }

  deleteCookie(event, SESSION_COOKIE_NAME, {
    path: '/',
  })

  return { ok: true }
})
