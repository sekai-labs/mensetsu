/**
 * Core API Client handling base URL resolution and authenticated requests.
 */


export interface RequestOptions {
  method?: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE'
  body?: unknown
  query?: Record<string, unknown>
  headers?: Record<string, string>
}

export const useApiClient = () => {
  const fetchClient = useRequestFetch()

  const request = async <T>(
    endpoint: string,
    options: RequestOptions = {}
  ): Promise<T> => {
    // Convert direct backend calls (/api/v1/...) to Nuxt server proxy (/api/backend/...)
    let url = endpoint
    if (url.startsWith('/api/v1/')) {
      url = url.replace(/^\/api\/v1\//, '/api/backend/')
    }

    return await fetchClient<T>(url, {
      method: options.method || 'GET',
      headers: options.headers,
      body: options.body as Record<string, unknown> | undefined,
      query: options.query,
    })
  }

  return {
    baseUrl: '',
    request,
  }
}
