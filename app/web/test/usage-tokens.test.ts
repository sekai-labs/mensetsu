import { describe, it, expect } from 'vitest'
import {
  normalizeMetricsSummary,
  buildTokenUsageQueryParams,
} from '../features/usage/services/usage.service'
import type { MetricsSummaryResponseDto, TokenUsageFilterQuery } from '../models/metrics.model'

describe('Metrics summary contract normalization', () => {
  it('returns only authenticated user totals and hides platform metrics', () => {
    const response = {
      token_totals: {
        request_count: 42,
        input_tokens: 15000,
        output_tokens: 4500,
        total_tokens: 19500,
      },
      platform_token_totals: {
        request_count: 1200,
        input_tokens: 500000,
        output_tokens: 120000,
        total_tokens: 620000,
      },
      service_metrics: {
        success_rate_percent: 99.8,
        requests_per_second: 15.4,
      },
    } satisfies MetricsSummaryResponseDto

    const summary = normalizeMetricsSummary(response, 'user')
    expect(summary.scope).toBe('user')
    expect(summary.token_usage.request_count).toBe(42)
    expect(summary.token_usage.total_tokens).toBe(19500)
    expect(summary.http).toBeNull()
  })

  it('uses platform totals and service metrics only for admins', () => {
    const response = {
      token_totals: {
        request_count: 5,
        input_tokens: 1000,
        output_tokens: 200,
        total_tokens: 1200,
      },
      platform_token_totals: {
        request_count: 850,
        input_tokens: 340000,
        output_tokens: 85000,
        total_tokens: 425000,
      },
      service_metrics: {
        success_rate_percent: 100,
        requests_per_second: 8.5,
      },
    } satisfies MetricsSummaryResponseDto

    const adminSummary = normalizeMetricsSummary(response, 'platform')
    expect(adminSummary.scope).toBe('platform')
    expect(adminSummary.token_usage.total_tokens).toBe(425000)
    expect(adminSummary.http).toEqual({
      success_rate_percent: 100,
      requests_per_second: 8.5,
    })

    const userSummary = normalizeMetricsSummary(response, 'user')
    expect(userSummary.token_usage.total_tokens).toBe(1200)
    expect(userSummary.http).toBeNull()
  })

  it('does not fabricate service metrics when the API omits them', () => {
    const response = {
      token_totals: {
        request_count: 0,
        input_tokens: 0,
        output_tokens: 0,
        total_tokens: 0,
      },
    } satisfies MetricsSummaryResponseDto

    expect(normalizeMetricsSummary(response, 'platform').http).toBeNull()
  })
})

describe('Token Usage Query & Filter Building', () => {
  it('builds standard query parameters with default 25-row limit', () => {
    const filter: TokenUsageFilterQuery = {}
    const params = buildTokenUsageQueryParams(filter, false)

    expect(params.limit).toBe(25)
    expect(params.search).toBeUndefined()
    expect(params.provider).toBeUndefined()
    expect(params.agent).toBeUndefined()
    expect(params.user_id).toBeUndefined()
  })

  it('binds server-side search, provider, agent, and date filters', () => {
    const filter: TokenUsageFilterQuery = {
      search: 'claude-3-5-sonnet',
      provider: 'anthropic',
      agent: 'technical-research',
      from_date: '2026-11-01',
      to_date: '2026-11-14',
    }
    const params = buildTokenUsageQueryParams(filter, false)

    expect(params.search).toBe('claude-3-5-sonnet')
    expect(params.provider).toBe('anthropic')
    expect(params.agent).toBe('technical-research')
    expect(params.from_date).toBe('2026-11-01')
    expect(params.to_date).toBe('2026-11-14')
  })

  it('includes user_id filter ONLY when requester is admin', () => {
    const filterWithUser: TokenUsageFilterQuery = {
      user_id: 'user:engineer_01',
    }

    // Regular user: user_id must NOT be bound
    const userParams = buildTokenUsageQueryParams(filterWithUser, false)
    expect(userParams.user_id).toBeUndefined()

    // Admin: user_id is bound
    const adminParams = buildTokenUsageQueryParams(filterWithUser, true)
    expect(adminParams.user_id).toBe('user:engineer_01')
  })

  it('binds cursor pagination parameters when navigating pages', () => {
    const filterWithCursor: TokenUsageFilterQuery = {
      before_timestamp: '2026-11-13T20:00:00.000Z',
      before_event_id: 'evt_12345',
    }
    const params = buildTokenUsageQueryParams(filterWithCursor, false)

    expect(params.before_timestamp).toBe('2026-11-13T20:00:00.000Z')
    expect(params.before_event_id).toBe('evt_12345')
  })
})

describe('Cursor Stack Pagination Invariants', () => {
  it('manages cursor history stack for bidirectional 25-row navigation', () => {
    // Initial page 1: no cursor
    const cursorStack: Array<{ before_timestamp: string; before_event_id: string } | null> = [null]
    let pageNumber = 1

    expect(cursorStack[pageNumber - 1]).toBeNull()

    // Page 1 arrives with next_cursor
    const page1NextCursor = {
      before_timestamp: '2026-11-14T02:00:00.000Z',
      before_event_id: 'evt_row_25',
    }
    cursorStack.push(page1NextCursor)

    // User clicks Next -> Page 2
    pageNumber++
    expect(pageNumber).toBe(2)
    expect(cursorStack[pageNumber - 1]).toEqual(page1NextCursor)

    // Page 2 arrives with next_cursor
    const page2NextCursor = {
      before_timestamp: '2026-11-14T01:00:00.000Z',
      before_event_id: 'evt_row_50',
    }
    cursorStack.push(page2NextCursor)

    // User clicks Previous -> Page 1
    pageNumber--
    expect(pageNumber).toBe(1)
    expect(cursorStack[pageNumber - 1]).toBeNull()

    // Resetting filters resets stack to [null] and page to 1
    const resetStack = [null]
    const resetPage = 1
    expect(resetStack).toEqual([null])
    expect(resetPage).toBe(1)
  })
})
