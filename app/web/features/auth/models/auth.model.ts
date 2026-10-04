import type { UserRole } from '~/models/common.model'

export type { UserRole }

export interface User {
  id: string
  email: string
  name: string
  role: UserRole
  status?: string
  totp_enabled: boolean
  created_at: string
  updated_at: string
}

export interface AuthResponse {
  user: User
  access_token: string
  refresh_token: string
}

export interface MeResponse {
  user: User
}

export interface LoginCredentials {
  email: string
  password: string
}

export interface RegisterCredentials {
  email: string
  password: string
  displayName: string
}

export interface LoginResult {
  user?: User
  requires_2fa: boolean
  temp_token?: string
}

export interface TotpSetupData {
  secret: string
  qr_code_data_url: string
  otpauth_url: string
}
