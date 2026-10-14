import { describe, it, expect } from 'vitest'
import {
  DEFAULT_PIPELINE_STEPS,
  calculateStepState,
  filterPipelineLogs,
  formatLogTimestamp,
  formatLogsForExport,
  getLogLevelBadgeStyle,
  getPipelineStatusStyle,
  getStepCardStyle,
} from '../features/admin/utils/admin.util'
import type { AdminPipelineLog } from '../features/admin/models/admin.model'

describe('Admin Feature Utilities', () => {
  it('correctly maps pipeline status to styles', () => {
    expect(getPipelineStatusStyle('ready')).toContain('emerald')
    expect(getPipelineStatusStyle('processing')).toContain('indigo')
    expect(getPipelineStatusStyle('failed')).toContain('rose')
    expect(getPipelineStatusStyle('pending')).toContain('slate')
  })

  it('correctly maps log level badge styles', () => {
    expect(getLogLevelBadgeStyle('error')).toContain('rose')
    expect(getLogLevelBadgeStyle('warn')).toContain('amber')
    expect(getLogLevelBadgeStyle('info')).toContain('blue')
    expect(getLogLevelBadgeStyle('debug')).toContain('slate')
  })

  it('calculates step state accurately based on progress', () => {

    expect(calculateStepState('preparing_input', 'ready', 'ready')).toBe('completed')
    expect(calculateStepState('generating_lessons', 'ready', 'ready')).toBe('completed')


    expect(
      calculateStepState(
        'preparing_input',
        'reading_job_description',
        'processing',
        DEFAULT_PIPELINE_STEPS
      )
    ).toBe('completed')

    expect(
      calculateStepState(
        'reading_job_description',
        'reading_job_description',
        'processing',
        DEFAULT_PIPELINE_STEPS
      )
    ).toBe('active')

    expect(
      calculateStepState(
        'generating_lessons',
        'reading_job_description',
        'processing',
        DEFAULT_PIPELINE_STEPS
      )
    ).toBe('pending')


    expect(
      calculateStepState(
        'researching_company',
        'researching_company',
        'failed',
        DEFAULT_PIPELINE_STEPS
      )
    ).toBe('failed')
  })

  it('formats log timestamps properly', () => {
    expect(formatLogTimestamp('2026-10-05T14:32:05.123Z')).toBe('14:32:05.123')
    expect(formatLogTimestamp('14:32:05')).toBe('14:32:05')
  })

  it('filters pipeline logs by keyword and level', () => {
    const sampleLogs: AdminPipelineLog[] = [
      {
        timestamp: '2026-10-05T14:00:00Z',
        level: 'info',
        step: 'analyzing_resume',
        message: 'Parsed 12 work experiences',
      },
      {
        timestamp: '2026-10-05T14:01:00Z',
        level: 'warn',
        step: 'researching_company',
        message: 'Rate limit approaching on search API',
      },
      {
        timestamp: '2026-10-05T14:02:00Z',
        level: 'error',
        step: 'researching_company',
        message: 'Connection timeout after 30s',
      },
    ]

    const errorLogs = filterPipelineLogs(sampleLogs, '', 'error')
    expect(errorLogs).toHaveLength(1)
    expect(errorLogs[0]?.step).toBe('researching_company')

    const searchLogs = filterPipelineLogs(sampleLogs, 'Parsed', 'all')
    expect(searchLogs).toHaveLength(1)
    expect(searchLogs[0]?.level).toBe('info')
  })

  it('formats log export text cleanly', () => {
    const sampleLogs: AdminPipelineLog[] = [
      {
        timestamp: '2026-10-05T14:00:00Z',
        level: 'info',
        step: 'analyzing_resume',
        message: 'Resume parsing completed',
      },
    ]

    const text = formatLogsForExport(sampleLogs, 'ws_123')
    expect(text).toContain('Workspace ID: ws_123')
    expect(text).toContain('[analyzing_resume]: Resume parsing completed')
  })

  it('provides step card styling classes', () => {
    expect(getStepCardStyle('completed')).toContain('emerald')
    expect(getStepCardStyle('active')).toContain('indigo')
    expect(getStepCardStyle('failed')).toContain('rose')
    expect(getStepCardStyle('pending')).toContain('slate')
  })
})
