import type {
  TokenUsageResponse,
  TokenUsageTotals,
} from '@mensetsu/contracts'
import { useApiClient } from '~/services/api.client'
import type {
  MetricsSummary,
  MetricsSummaryResponseDto,
  DailyTokenUsageResponse,
  TokenUsageFilterQuery,
} from '~/models/metrics.model'

const asRecord = (value: unknown): Record<string, unknown> =>
  value !== null && typeof value === 'object' ? (value as Record<string, unknown>) : {}

const finiteNumber = (value: unknown): number => {
  const number = typeof value === 'number' ? value : Number(value)
  return Number.isFinite(number) ? number : 0
}

export const normalizeMetricsSummary = (
  response: MetricsSummaryResponseDto,
  scope: 'user' | 'platform' = 'user'
): MetricsSummary => {
  const payload = response as unknown as Record<string, unknown>
  const totals = asRecord(
    scope === 'platform'
      ? payload.platform_token_totals ?? payload.token_totals
      : payload.token_totals
  )

  const token_usage: TokenUsageTotals = {
    request_count: finiteNumber(totals.request_count),
    input_tokens: finiteNumber(totals.input_tokens),
    output_tokens: finiteNumber(totals.output_tokens),
    total_tokens: finiteNumber(totals.total_tokens),
  }

  let http: MetricsSummary['http'] = null
  const serviceMetrics = asRecord(payload.service_metrics)
  if (scope === 'platform' && payload.service_metrics && typeof payload.service_metrics === 'object') {
    const successRate = Number(serviceMetrics.success_rate_percent)
    const requestsPerSecond = Number(serviceMetrics.requests_per_second)
    if (Number.isFinite(successRate) && Number.isFinite(requestsPerSecond)) {
      http = {
        success_rate_percent: successRate,
        requests_per_second: requestsPerSecond,
      }
    }
  }

  return { scope, token_usage, http }
}

export const buildTokenUsageQueryParams = (
  filter: TokenUsageFilterQuery,
  isAdmin = false
): Record<string, string | number> => {
  const params: Record<string, string | number> = {
    limit: filter.limit ?? 25,
  }

  if (filter.search?.trim()) {
    params.search = filter.search.trim()
  }
  if (filter.provider?.trim()) {
    params.provider = filter.provider.trim()
  }
  if (filter.agent?.trim()) {
    params.agent = filter.agent.trim()
  }
  if (filter.from_date?.trim()) {
    params.from_date = filter.from_date.trim()
  }
  if (filter.to_date?.trim()) {
    params.to_date = filter.to_date.trim()
  }
  if (isAdmin && filter.user_id?.trim()) {
    params.user_id = filter.user_id.trim()
  }
  if (filter.before_timestamp && filter.before_event_id) {
    params.before_timestamp = filter.before_timestamp
    params.before_event_id = filter.before_event_id
  }

  return params
}

export const useUsageService = () => {
  const { request } = useApiClient()

  const getMetricsSummary = async (isAdmin = false): Promise<MetricsSummary> => {
    const response = await request<MetricsSummaryResponseDto>('/api/v1/metrics/summary')
    return normalizeMetricsSummary(response, isAdmin ? 'platform' : 'user')
  }

  const getDailyTokenUsage = async (isAdmin = false): Promise<DailyTokenUsageResponse> => {
    const primaryEndpoint = isAdmin
      ? '/api/v1/admin/analytics/tokens/daily'
      : '/api/v1/analytics/tokens/daily'
    const fallbackEndpoint = isAdmin
      ? '/api/v1/admin/usage/tokens/daily'
      : '/api/v1/usage/tokens/daily'

    try {
      return await request<DailyTokenUsageResponse>(primaryEndpoint)
    } catch {
      // Try fallback route
      return await request<DailyTokenUsageResponse>(fallbackEndpoint)
    }
  }

  const getTokenUsagePage = async (
    filter: TokenUsageFilterQuery,
    isAdmin = false
  ): Promise<TokenUsageResponse> => {
    const endpoint = isAdmin ? '/api/v1/admin/usage/tokens' : '/api/v1/usage/tokens'
    const query = buildTokenUsageQueryParams(filter, isAdmin)
    return await request<TokenUsageResponse>(endpoint, { query })
  }

  return {
    getMetricsSummary,
    getDailyTokenUsage,
    getTokenUsagePage,
  }
}
