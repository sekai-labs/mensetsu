export class ApiError extends Error {
  statusCode: number
  data?: unknown

  constructor(message: string, statusCode = 500, data?: unknown) {
    super(message)
    this.name = 'ApiError'
    this.statusCode = statusCode
    this.data = data
  }
}

export const useApiClient = () => {
  const request = async <T>(
    path: string,
    options: {
      method?: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE'
      body?: unknown
      query?: Record<string, unknown>
      headers?: Record<string, string>
    } = {}
  ): Promise<T> => {

    const backendPath = path.startsWith('/api/v1/') ? path.replace('/api/v1/', '') : path.replace(/^\//, '')
    const url = `/api/backend/${backendPath}`

    try {
      const fetchHeaders: Record<string, string> = {
        ...(options.headers || {}),
      }
      if (import.meta.server) {
        const reqHeaders = useRequestHeaders(['cookie', 'authorization'])
        Object.assign(fetchHeaders, reqHeaders)
      }

      const res = await $fetch.raw(url, {
        method: options.method || 'GET',
        body: options.body as Record<string, unknown> | undefined,
        query: options.query,
        headers: fetchHeaders,
      })
      return res._data as unknown as T
    } catch (err: unknown) {
      if (err && typeof err === 'object') {
        const statusCode = (('statusCode' in err ? err.statusCode : undefined) ||
          ('status' in err ? err.status : undefined) ||
          500) as number
        let message = 'An unexpected API error occurred'
        let data: unknown = undefined

        if ('data' in err && err.data) {
          data = err.data
          if (typeof err.data === 'object' && 'error' in err.data && err.data.error) {
            const errData = err.data.error
            if (typeof errData === 'object' && 'message' in errData && typeof errData.message === 'string') {
              message = errData.message
            }
          }
        } else if ('message' in err && typeof err.message === 'string') {
          message = err.message
        }

        throw new ApiError(message, statusCode, data)
      }

      throw new ApiError('Network connection failed', 500)
    }
  }

  return { request }
}
