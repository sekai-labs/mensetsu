export const getBackendApiBase = (): string => {
  return process.env.NUXT_API_BASE_SERVER || process.env.INTERNAL_API_BASE || 'http://localhost:3000'
}

export const SESSION_COOKIE_NAME = 'mensetsu_session_id'
