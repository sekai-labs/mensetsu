import { describe, it, expect } from 'vitest'
import {
  filterWorkspaces,
  calculateWorkspaceStatistics,
  groupQuestionsByCategory,
  filterQuestions,
  calculateLearningProgress,
} from '../features/interviews/utils/interview.util'
import type {
  WorkspaceResponse,
  InterviewQuestionResponse,
  LearningUnitResponse,
} from '../features/interviews/models/interview.model'

describe('Interview Feature Utilities', () => {
  const sampleWorkspaces: WorkspaceResponse[] = [
    {
      id: 'ws_1',
      user_id: 'user_1',
      company_name: 'Stripe',
      role_title: 'Staff Infrastructure Engineer',
      status: 'ready',
      current_phase: 'ready',
      progress_percent: 100,
      created_at: '2026-01-01T00:00:00Z',
      updated_at: '2026-01-01T00:00:00Z',
    },
    {
      id: 'ws_2',
      user_id: 'user_1',
      company_name: 'Anthropic',
      role_title: 'Systems Research Engineer',
      status: 'processing',
      current_phase: 'researching_company',
      progress_percent: 45,
      created_at: '2026-01-02T00:00:00Z',
      updated_at: '2026-01-02T00:00:00Z',
    },
    {
      id: 'ws_3',
      user_id: 'user_1',
      company_name: 'OpenAI',
      role_title: 'Distributed Systems Tech Lead',
      status: 'failed',
      current_phase: 'failed',
      progress_percent: 30,
      created_at: '2026-01-03T00:00:00Z',
      updated_at: '2026-01-03T00:00:00Z',
    },
  ]

  it('filters workspaces by query and status correctly', () => {
    expect(filterWorkspaces(sampleWorkspaces, 'Stripe', 'all')).toHaveLength(1)
    expect(filterWorkspaces(sampleWorkspaces, 'systems', 'all')).toHaveLength(2)
    expect(filterWorkspaces(sampleWorkspaces, '', 'ready')).toHaveLength(1)
    expect(filterWorkspaces(sampleWorkspaces, '', 'processing')).toHaveLength(1)
    expect(filterWorkspaces(sampleWorkspaces, 'NonExistent', 'all')).toHaveLength(0)
  })

  it('calculates workspace summary statistics accurately', () => {
    const stats = calculateWorkspaceStatistics(sampleWorkspaces)
    expect(stats.total).toBe(3)
    expect(stats.ready).toBe(1)
    expect(stats.active).toBe(1)
    expect(stats.failed).toBe(1)
  })

  const sampleQuestions: InterviewQuestionResponse[] = [
    {
      id: 'q_1',
      workspace_id: 'ws_1',
      category: 'system_design',
      question: 'Design a distributed rate limiter for payment gateways.',
      difficulty: 'hard',
      skills_tested: ['Distributed Systems', 'Redis'],
      interviewer_intent: 'Check token bucket and race condition mitigation.',
      answer_outline: ['Explain sliding log algorithm', 'Use Redis lua script'],
      strong_answer_rubric: ['Addresses split-brain scenarios'],
      common_weaknesses: ['Assuming network calls never fail'],
      follow_up_questions: ['How to handle global cluster synchronization?'],
      created_at: '2026-01-01T00:00:00Z',
    },
    {
      id: 'q_2',
      workspace_id: 'ws_1',
      category: 'debugging',
      question: 'Diagnose memory leaks in asynchronous Tokio worker pool.',
      difficulty: 'medium',
      skills_tested: ['Rust', 'Async I/O'],
      interviewer_intent: 'Examine heap profiling ability.',
      answer_outline: ['Inspect Arc cyclic references'],
      strong_answer_rubric: ['Uses heap profiler graphs'],
      common_weaknesses: ['Restarting pods instead of root cause isolation'],
      follow_up_questions: ['How do you reproduce under synthetic load?'],
      created_at: '2026-01-01T00:00:00Z',
    },
  ]

  it('groups questions by category', () => {
    const grouped = groupQuestionsByCategory(sampleQuestions)
    expect(Object.keys(grouped)).toContain('system_design')
    expect(Object.keys(grouped)).toContain('debugging')
    expect(grouped.system_design).toHaveLength(1)
    expect(grouped.debugging).toHaveLength(1)
  })

  it('filters questions by keyword, difficulty, and category', () => {
    const rateLimiter = filterQuestions(sampleQuestions, {
      searchQuery: 'rate limiter',
      difficulty: 'all',
      category: 'all',
    })
    expect(rateLimiter).toHaveLength(1)
    expect(rateLimiter[0]!.id).toBe('q_1')
    const hardQuestions = filterQuestions(sampleQuestions, {
      difficulty: 'hard',
    })
    expect(hardQuestions).toHaveLength(1)

    const skillSearch = filterQuestions(sampleQuestions, {
      searchQuery: 'Rust',
    })
    expect(skillSearch).toHaveLength(1)
    expect(skillSearch[0]!.id).toBe('q_2')
  })

  it('calculates learning unit progress correctly', () => {
    const units: LearningUnitResponse[] = [
      {
        id: 'u_1',
        workspace_id: 'ws_1',
        skill: 'Temporal',
        title: 'Temporal Invariants',
        prerequisites: [],
        estimated_minutes: 30,
        summary: 'Intro to workflow event histories',
        content_markdown: 'Content here',
        completed: true,
        created_at: '2026-01-01T00:00:00Z',
      },
      {
        id: 'u_2',
        workspace_id: 'ws_1',
        skill: 'SurrealDB',
        title: 'Graph Traversal',
        prerequisites: ['Temporal'],
        estimated_minutes: 45,
        summary: 'Multi-hop graph queries',
        content_markdown: 'Content here',
        completed: false,
        created_at: '2026-01-01T00:00:00Z',
      },
    ]

    const progress = calculateLearningProgress(units)
    expect(progress.completedCount).toBe(1)
    expect(progress.totalCount).toBe(2)
    expect(progress.percentage).toBe(50)
    expect(progress.totalMinutes).toBe(75)
  })
})
