import type { UserRole } from '~/models/common.model'
import type { components } from '@mensetsu/contracts'

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
  temp_token?: string
  code: string
}

export interface AuthResponse {
  user?: User
  access_token?: string
  refresh_token?: string
  requires_2fa?: boolean
  temp_token?: string
}

export type TotpSetupResponse = components['schemas']['TotpSetupResponseDto']

export interface TotpVerifyRequest {
  code: string
}

export interface TotpDisableRequest {
  code: string
}

// OAuth DTOs use the Rust-generated canonical OpenAPI schemas.
export type OAuthProviderSummaryDto = components['schemas']['OAuthProviderSummaryDto']
export type OAuthIdentityDto = components['schemas']['OAuthIdentityDto']
export type OAuthProviderAdminDto = components['schemas']['OAuthProviderAdminDto']
export type OAuthProviderUpsertRequestDto = components['schemas']['OAuthProviderUpsertRequestDto']
export type OAuthLinkStartResponseDto = components['schemas']['OAuthLinkStartResponseDto']
export type OAuthExchangeRequestDto = components['schemas']['OAuthExchangeRequestDto']
export type OAuthExchangeResponseDto = components['schemas']['OAuthExchangeResponseDto']
