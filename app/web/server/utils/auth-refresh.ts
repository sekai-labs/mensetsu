import { sessionManager, type SessionData } from '~/server/utils/session'

const inFlightRefresh = new Map<string, Promise<string>>()

export async function refreshSessionToken(
  apiBase: string,
  tokenId: string,
  session: SessionData
): Promise<string> {
  const existingPromise = inFlightRefresh.get(tokenId)
  if (existingPromise) {
    return await existingPromise
  }

  const refreshPromise = (async () => {
    const latestSession = await sessionManager.getSession(tokenId)
    if (latestSession && latestSession.refreshToken !== session.refreshToken && latestSession.accessToken) {
      return latestSession.accessToken
    }

    const currentRefreshToken = latestSession?.refreshToken || session.refreshToken

    try {
      const refreshRes = await $fetch<{
        access_token: string
        refresh_token: string
      }>(`${apiBase}/api/v1/auth/refresh`, {
        method: 'POST',
        body: {
          refresh_token: currentRefreshToken,
        },
      })

      await sessionManager.updateAccessToken(tokenId, refreshRes.access_token, refreshRes.refresh_token)
      return refreshRes.access_token
    } catch (err: unknown) {
      const recheckedSession = await sessionManager.getSession(tokenId)
      if (recheckedSession && recheckedSession.refreshToken !== currentRefreshToken && recheckedSession.accessToken) {
        return recheckedSession.accessToken
      }
      throw err
    }
  })()

  inFlightRefresh.set(tokenId, refreshPromise)
  try {
    return await refreshPromise
  } finally {
    inFlightRefresh.delete(tokenId)
  }
}
