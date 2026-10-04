import { describe, it, expect } from 'vitest'
import type { WorkspaceResponse, MatchAnalysisResponse } from '../features/interviews/models/interview.model'

describe('Web Domain Contracts and Calculations', () => {
  it('correctly models workspace response state', () => {
    const ws: WorkspaceResponse = {
      id: 'ws_test_123',
      user_id: 'user_1',
      company_name: 'Anthropic',
      role_title: 'Systems Engineer',
      status: 'ready',
      current_phase: 'ready',
      progress_percent: 100,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    }

    expect(ws.status).toBe('ready')
    expect(ws.progress_percent).toBe(100)
    expect(ws.company_name).toBe('Anthropic')
  })

  it('calculates match percentage accurately', () => {
    const match: MatchAnalysisResponse = {
      workspace_id: 'ws_test_123',
      overall_match_ratio: 0.85,
      strengths: [
        { skill: 'Rust', evidence_quote: '7 years Rust', relevance: 'Core requirement' }
      ],
      gaps: [
        { skill: 'Temporal', requirement_quote: 'Durable orchestration', importance: 'preferred', recommended_focus: 'Study DAG workflows' }
      ],
      risks: ['Probing on distributed transactions'],
      wording_improvements: [],
    }

    const percentage = Math.round(match.overall_match_ratio * 100)
    expect(percentage).toBe(85)
    expect(match.strengths.length).toBe(1)
    expect(match.gaps.length).toBe(1)
  })
})
