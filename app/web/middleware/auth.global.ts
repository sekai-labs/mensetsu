import { isPublicAuthRoute, resolvePostAuthRedirect } from '~/features/auth/utils/auth.util'
import { useAuthService } from '~/features/auth/services/auth.service'

export default defineNuxtRouteMiddleware(async (to) => {
  const { user, fetchMe } = useAuthService()

  if (!user.value) {
    try {
      await fetchMe()
    } catch {
      // Session cookie check failed or expired
    }
  }

  const isAuthenticated = !!user.value
  const redirectPath = resolvePostAuthRedirect(isAuthenticated, to.path, to.query.redirect as string | null)
  if (redirectPath) {
    return navigateTo(redirectPath)
  }
})
