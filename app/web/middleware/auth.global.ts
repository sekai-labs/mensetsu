import { resolvePostAuthRedirect } from '~/features/auth/utils/auth.util'
import { useAuthService } from '~/features/auth/services/auth.service'

export default defineNuxtRouteMiddleware(async (to) => {
  const { user, isAuthenticated, isInitialized, fetchCurrentUser } = useAuthService()

  if (!isInitialized.value) {
    await fetchCurrentUser()
  }

  const redirectTarget = resolvePostAuthRedirect(isAuthenticated.value, to.path, to.query.redirect as string | undefined)

  if (redirectTarget) {
    return navigateTo(redirectTarget)
  }

  if ((to.path === '/admin' || to.path.startsWith('/admin/')) && user.value?.role !== 'admin') {
    return navigateTo('/dashboard')
  }
})
