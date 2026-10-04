import type { ComputedRef, Ref } from 'vue'
import { computed } from 'vue'
import { useApiClient } from '~/services/api.client'
import type {
  User,
  LoginCredentials,
  RegisterCredentials,
  LoginResult,
  TotpSetupData,
} from '../models/auth.model'

export interface AuthService {
  user: Ref<User | null>
  loading: Ref<boolean>
  isAuthenticated: ComputedRef<boolean>
  fetchMe: () => Promise<User | null>
  login: (credentials: LoginCredentials) => Promise<LoginResult>
  login2fa: (tempToken: string, code: string) => Promise<User>
  register: (credentials: RegisterCredentials) => Promise<User>
  logout: () => Promise<void>
  setupTotp: () => Promise<TotpSetupData>
  verifyTotp: (code: string) => Promise<User>
  disableTotp: (code: string) => Promise<User>
}

export const useAuthService = (): AuthService => {
  const { request } = useApiClient()

  const user = useState<User | null>('auth-user', () => null)
  const loading = useState<boolean>('auth-loading', () => false)
  const isAuthenticated: ComputedRef<boolean> = computed(() => !!user.value)

  const fetchMe = async (): Promise<User | null> => {
    loading.value = true
    try {
      const data = await request<{ user: User }>('/api/auth/me')
      user.value = data.user
      return data.user
    } catch {
      user.value = null
      return null
    } finally {
      loading.value = false
    }
  }

  const login = async (credentials: LoginCredentials): Promise<LoginResult> => {
    loading.value = true
    try {
      const data = await request<LoginResult>('/api/auth/login', {
        method: 'POST',
        body: credentials,
      })
      if (!data.requires_2fa && data.user) {
        user.value = data.user
      }
      return data
    } finally {
      loading.value = false
    }
  }

  const login2fa = async (tempToken: string, code: string): Promise<User> => {
    loading.value = true
    try {
      const data = await request<{ user: User }>('/api/auth/login-2fa', {
        method: 'POST',
        body: {
          temp_token: tempToken,
          code,
        },
      })
      user.value = data.user
      return data.user
    } finally {
      loading.value = false
    }
  }

  const register = async (credentials: RegisterCredentials): Promise<User> => {
    loading.value = true
    try {
      const data = await request<{ user: User }>('/api/auth/register', {
        method: 'POST',
        body: {
          email: credentials.email,
          password: credentials.password,
          name: credentials.displayName,
        },
      })
      user.value = data.user
      return data.user
    } finally {
      loading.value = false
    }
  }

  const logout = async (): Promise<void> => {
    try {
      await request('/api/auth/logout', { method: 'POST' })
    } finally {
      user.value = null
    }
  }

  const setupTotp = async (): Promise<TotpSetupData> => {
    return await request<TotpSetupData>('/api/auth/2fa/setup', {
      method: 'POST',
    })
  }

  const verifyTotp = async (code: string): Promise<User> => {
    const res = await request<User>('/api/auth/2fa/verify', {
      method: 'POST',
      body: { code },
    })
    if (user.value) {
      user.value.totp_enabled = true
    }
    return res
  }

  const disableTotp = async (code: string): Promise<User> => {
    const res = await request<User>('/api/auth/2fa/disable', {
      method: 'POST',
      body: { code },
    })
    if (user.value) {
      user.value.totp_enabled = false
    }
    return res
  }

  return {
    user,
    loading,
    isAuthenticated,
    fetchMe,
    login,
    login2fa,
    register,
    logout,
    setupTotp,
    verifyTotp,
    disableTotp,
  }
}
