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
        const errorResponse = 'response' in err && err.response && typeof err.response === 'object'
          ? err.response as { status?: number; _data?: unknown }
          : undefined
        const statusCode = (('statusCode' in err ? err.statusCode : undefined) ||
          ('status' in err ? err.status : undefined) ||
          errorResponse?.status ||
          500) as number
        let message = 'An unexpected API error occurred'
        let data: unknown = undefined

        const findUsefulMessage = (value: unknown): string | undefined => {
          if (!value || typeof value !== 'object') {
            return undefined
          }

          const errorData = value as Record<string, unknown>
          for (const key of ['error', 'data']) {
            const nestedMessage = findUsefulMessage(errorData[key])
            if (nestedMessage) {
              return nestedMessage
            }
          }

          if (typeof errorData.message === 'string' &&
            !['An unexpected API error occurred', 'Proxy request failed', 'Forbidden'].includes(errorData.message)) {
            return errorData.message
          }

          return undefined
        }

        const errorData = ('data' in err && err.data) || errorResponse?._data
        if (errorData) {
          data = errorData
          const dataMessage = findUsefulMessage(errorData)
          if (dataMessage) {
            message = dataMessage
          }
        } else if ('message' in err && typeof err.message === 'string' &&
          !['An unexpected API error occurred', 'Proxy request failed', 'Forbidden'].includes(err.message)) {
          message = err.message
        }

        if (statusCode === 403 && message === 'An unexpected API error occurred') {
          message = 'You do not have permission to access this resource.'
        }

        throw new ApiError(message, statusCode, data)
      }

      throw new ApiError('Network connection failed', 500)
    }
  }

  return { request }
}
