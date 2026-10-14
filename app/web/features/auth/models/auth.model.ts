import type { UserRole } from '~/models/common.model'

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

export interface LoginRequest {
  email: string
  password: string
}

export interface RegisterRequest {
  email: string
  password: string
  name: string
}

export interface Login2faRequest {
  temp_token: string
  code: string
}

export interface AuthResponse {
  user?: User
  access_token?: string
  refresh_token?: string
  requires_2fa?: boolean
  temp_token?: string
}

export interface TotpSetupResponse {
  secret: string
  qr_code_svg: string
  otpauth_url: string
}

export interface TotpVerifyRequest {
  code: string
}

export interface TotpDisableRequest {
  code: string
}
