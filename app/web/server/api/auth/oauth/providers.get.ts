import { getBackendApiBase } from '~/server/utils/config'
import type { OAuthProviderSummaryDto } from '~/features/auth/models/auth.model'

export default defineEventHandler(async () => {
  const apiBase = getBackendApiBase()
  try {
    const providers = await $fetch<OAuthProviderSummaryDto[]>(
      `${apiBase}/api/v1/auth/oauth/providers`
    )
    return providers || []
  } catch (err: unknown) {
    // If backend is unavailable or has no providers configured, return empty list gracefully
    return []
  }
})
