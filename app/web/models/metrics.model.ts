import type {
  TokenUsageTotals,
  TokenUsageRow,
  TokenUsageCursor,
  components,
  operations,
} from '@mensetsu/contracts'

export type DailyTokenUsageDto = components['schemas']['DailyTokenUsageDto']
export type DailyTokenUsageItem = DailyTokenUsageDto
export type DailyTokenUsageResponseDto = components['schemas']['DailyTokenUsageResponseDto']
export type MetricsSummaryResponseDto = components['schemas']['MetricsSummaryResponseDto']
export type ServiceMetricsDto = components['schemas']['ServiceMetricsDto']
export type TokenUsageFilterQuery = NonNullable<operations['user_token_usage']['parameters']['query']>

export interface HttpServiceMetrics {
  success_rate_percent: number
  requests_per_second: number
}

export interface MetricsSummary {
  scope: 'user' | 'platform'
  token_usage: TokenUsageTotals
  http: HttpServiceMetrics | null
}

export interface DailyTokenUsageResponse {
  items: DailyTokenUsageItem[]
  totals: TokenUsageTotals
}

export interface TokenUsagePageState {
  items: TokenUsageRow[]
  totals: TokenUsageTotals
  next_cursor: TokenUsageCursor | null
}
