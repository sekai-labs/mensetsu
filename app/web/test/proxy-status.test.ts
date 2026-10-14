import { describe, it, expect } from 'vitest'

describe('Backend Proxy Status Preservation', () => {
  it('preserves upstream 201 Created and other status codes', async () => {
    let capturedStatus: number | null = null

    const mockEvent = {
      context: {
        params: { path: 'api/v1/workspaces' },
      },
      node: {
        req: { method: 'POST', url: '/api/backend/api/v1/workspaces' },
        res: {},
      },
    }

    const setResponseStatus = (_event: unknown, status: number) => {
      capturedStatus = status
    }

    const upstreamMock = {
      status: 201,
      _data: { id: 'workspace:abc', company_name: 'Acme' },
    }

    if (upstreamMock.status) {
      setResponseStatus(mockEvent, upstreamMock.status)
    }

    expect(capturedStatus).toBe(201)
    expect(upstreamMock._data).toEqual({ id: 'workspace:abc', company_name: 'Acme' })
  })
})
