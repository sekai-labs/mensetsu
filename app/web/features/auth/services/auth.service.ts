import type {
  User,
  LoginRequest,
  RegisterRequest,
  Login2faRequest,
  AuthResponse,
  TotpSetupResponse,
  TotpVerifyRequest,
  TotpDisableRequest,
} from '../models/auth.model'

export const useAuthService = () => {
  const user = useState<User | null>('auth-user', () => null)
  const isInitialized = useState<boolean>('auth-initialized', () => false)
  const isAuthenticated = computed(() => !!user.value)

  const fetchCurrentUser = async (): Promise<User | null> => {
    try {
      const headers: Record<string, string> = {}
      if (import.meta.server) {
        const reqHeaders = useRequestHeaders(['cookie', 'authorization'])
        Object.assign(headers, reqHeaders)
      }
      const res = await $fetch<{ user: User }>('/api/auth/me', { headers })
      user.value = res.user
      return res.user
    } catch {
      user.value = null
      return null
    } finally {
      isInitialized.value = true
    }
  }

  const login = async (payload: LoginRequest): Promise<AuthResponse> => {
    const res = await $fetch<AuthResponse>('/api/auth/login', {
      method: 'POST',
      body: payload,
    })
    if (!res.requires_2fa && res.user) {
      user.value = res.user
    }
    return res
  }

  const login2fa = async (payload: Login2faRequest): Promise<AuthResponse> => {
    const res = await $fetch<AuthResponse>('/api/auth/login-2fa', {
      method: 'POST',
      body: payload,
    })
    if (res.user) {
      user.value = res.user
    }
    return res
  }

  const register = async (payload: RegisterRequest): Promise<AuthResponse> => {
    const res = await $fetch<AuthResponse>('/api/auth/register', {
      method: 'POST',
      body: payload,
    })
    if (res.user) {
      user.value = res.user
    }
    return res
  }

  const logout = async (): Promise<void> => {
    try {
      await $fetch('/api/auth/logout', { method: 'POST' })
    } finally {
      user.value = null
    }
  }

  const setup2fa = async (): Promise<TotpSetupResponse> => {
    return await $fetch<TotpSetupResponse>('/api/auth/2fa/setup', {
      method: 'POST',
    })
  }

  const verify2fa = async (payload: TotpVerifyRequest): Promise<{ success: boolean }> => {
    const res = await $fetch<{ success: boolean }>('/api/auth/2fa/verify', {
      method: 'POST',
      body: payload,
    })
    if (user.value) {
      user.value.totp_enabled = true
    }
    return res
  }

  const disable2fa = async (payload: TotpDisableRequest): Promise<{ success: boolean }> => {
    const res = await $fetch<{ success: boolean }>('/api/auth/2fa/disable', {
      method: 'POST',
      body: payload,
    })
    if (user.value) {
      user.value.totp_enabled = false
    }
    return res
  }

  return {
    user,
    isAuthenticated,
    isInitialized,
    fetchCurrentUser,
    login,
    login2fa,
    register,
    logout,
    setup2fa,
    verify2fa,
    disable2fa,
  }
}
