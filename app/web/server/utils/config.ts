export const getBackendApiBase = (): string => {
  return process.env.NUXT_API_BASE_SERVER || process.env.INTERNAL_API_BASE || 'http://localhost:3000'
}

export const SESSION_COOKIE_NAME = 'mensetsu_session_id'
export const OAUTH_REDIRECT_COOKIE_NAME = 'mensetsu_oauth_redirect'
export const TOTP_TEMP_TOKEN_COOKIE_NAME = 'mensetsu_2fa_temp_token'
