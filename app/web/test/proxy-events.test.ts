import { describe, it, expect } from 'vitest'
import http from 'node:http'
import { isEventsPath, buildProxyHeaders, buildTargetUrl } from '../server/utils/routes'

describe('Backend Proxy Route Matching & Header Handling', () => {
  it('identifies exact SSE events endpoints and rejects history endpoints', () => {
    expect(isEventsPath('/api/v1/workflows/runs/run_123/events')).toBe(true)
    expect(isEventsPath('/api/v1/workspaces/ws_456/events')).toBe(true)
    expect(isEventsPath('/api/v1/workflows/runs/run_123/events/history')).toBe(false)
    expect(isEventsPath('/api/v1/workflows/runs')).toBe(false)
    expect(isEventsPath('/api/v1/workflows/runs/run_123')).toBe(false)
    expect(isEventsPath('/api/v1/workflows/runs/run_123/graph')).toBe(false)
    expect(isEventsPath('/api/v1/workflows/runs/run_123/logs')).toBe(false)
  })

  it('builds targetUrl without embedded query string to avoid duplicate params', () => {
    const targetUrl = buildTargetUrl('http://127.0.0.1:8000', '/api/v1/workflows/runs')
    expect(targetUrl).toBe('http://127.0.0.1:8000/api/v1/workflows/runs')
  })

  it('builds upstream headers without browser cookie header', () => {
    const headers = buildProxyHeaders({
      accessToken: 'test-token',
      contentType: 'application/json',
      accept: 'text/event-stream',
      lastEventId: '42',
    })
    expect(headers.Authorization).toBe('Bearer test-token')
    expect(headers['content-type']).toBe('application/json')
    expect(headers.accept).toBe('text/event-stream')
    expect(headers['last-event-id']).toBe('42')
    expect((headers as Record<string, string>).cookie).toBeUndefined()
    expect((headers as Record<string, string>).Cookie).toBeUndefined()
  })

  it('verifies upstream HTTP server receives Authorization and Last-Event-ID but no Cookie', async () => {
    let receivedHeaders: http.IncomingHttpHeaders | null = null

    const server = http.createServer((req, res) => {
      receivedHeaders = req.headers
      res.writeHead(200, { 'Content-Type': 'text/plain' })
      res.end('ok')
    })
    const { promise: listenPromise, resolve: listenResolve } = Promise.withResolvers<void>()
    server.listen(0, '127.0.0.1', () => listenResolve())
    await listenPromise
    const address = server.address() as { port: number }
    const url = `http://127.0.0.1:${address.port}/api/v1/workflows/runs/run_test/events`
    const proxyHeaders = buildProxyHeaders({
      accessToken: 'test-upstream-token',
      accept: 'text/event-stream',
      lastEventId: '108',
    })

    await fetch(url, {
      method: 'GET',
      headers: proxyHeaders,
    })

    server.close()

    expect(receivedHeaders).not.toBeNull()
    expect(receivedHeaders!.authorization).toBe('Bearer test-upstream-token')
    expect(receivedHeaders!['last-event-id']).toBe('108')
    expect(receivedHeaders!['accept']).toBe('text/event-stream')
    expect(receivedHeaders!.cookie).toBeUndefined()
  })
})
