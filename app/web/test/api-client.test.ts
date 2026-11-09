import { describe, expect, it, vi } from 'vitest'
import { useApiClient } from '../services/api.client'

describe('API client errors', () => {
  it('maps a message-less forbidden response to a permission error', async () => {
    vi.stubGlobal('$fetch', {
      raw: vi.fn().mockRejectedValue({
        response: {
          status: 403,
          _data: { message: 'Proxy request failed' },
        },
      }),
    })

    await expect(useApiClient().request('/workspaces/example')).rejects.toMatchObject({
      name: 'ApiError',
      message: 'You do not have permission to access this resource.',
      statusCode: 403,
    })
  })
})
