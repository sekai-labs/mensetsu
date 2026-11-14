import { handleOAuthCallback } from '~/server/utils/oauth-exchange'

export default defineEventHandler(async (event) => {
  return await handleOAuthCallback(event)
})
