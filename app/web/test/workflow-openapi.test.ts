import fs from 'node:fs'
import path from 'node:path'
import { describe, it, expect, vi } from 'vitest'
import { useWorkflowRunStore } from '../composables/useWorkflowStore'

describe('Workflow Service & Store Error Boundaries', () => {
  it('asserts every workflow service API path and method exists in openapi.json', () => {
    const openapiPath = path.resolve(__dirname, '../../../server/api/contracts/openapi.json')
    const openapi = JSON.parse(fs.readFileSync(openapiPath, 'utf8'))
    const openapiPaths = openapi.paths || {}

    const serviceCalls: Array<{ path: string; method: string }> = [
      { path: '/api/v1/workflows/runs', method: 'get' },
      { path: '/api/v1/workflows/runs', method: 'post' },
      { path: '/api/v1/workflows/runs/{run_id}', method: 'get' },
      { path: '/api/v1/workflows/runs/{run_id}/graph', method: 'get' },
      { path: '/api/v1/workflows/runs/{run_id}/delegations/graph', method: 'get' },
      { path: '/api/v1/workflows/runs/{run_id}/logs', method: 'get' },
      { path: '/api/v1/workflows/runs/{run_id}/events', method: 'get' },
      { path: '/api/v1/workflows/runs/{run_id}/delegations', method: 'get' },
      { path: '/api/v1/workflows/runs/{run_id}/artifacts', method: 'get' },
      { path: '/api/v1/workflows/runs/{run_id}/cancel', method: 'post' },
    ]

    for (const call of serviceCalls) {
      const endpoint = openapiPaths[call.path]
      expect(endpoint, `OpenAPI should declare path: ${call.path}`).toBeDefined()
      expect(endpoint[call.method], `OpenAPI path ${call.path} should support ${call.method.toUpperCase()}`).toBeDefined()
    }
  })

  it('treats 404 in eventSource.onerror as terminal and stops reconnecting', async () => {
    const { store } = useWorkflowRunStore('run_foreign_or_missing_404')
    const instance = store.value!

    const svc = instance.service as Record<string, any>
    vi.spyOn(svc, 'getWorkflowRun').mockRejectedValue({
      statusCode: 404,
      message: 'Workflow run not found',
    })

    let registeredOnError: (() => void) | null = null
    instance.connectSse((url) => {
      const fake = {
        url,
        addEventListener: () => {},
        removeEventListener: () => {},
        close: () => {},
        set onerror(fn: () => void) {
          registeredOnError = fn
        },
      }
      return fake as unknown as EventSource
    })

    expect(registeredOnError).not.toBeNull()
    await registeredOnError!()

    expect(instance.connectionError.value).toBe('Run not found or not accessible.')
    expect(instance.sectionErrors.value.run).toBe('Run not found or not accessible.')
    expect(instance.isConnected.value).toBe(false)
  })
})
