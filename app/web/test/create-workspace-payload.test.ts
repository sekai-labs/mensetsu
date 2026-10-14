import { describe, it, expect, vi } from 'vitest'
import type { CreateWorkspaceRequest } from '../features/interviews/models/interview.model'
import { useInterviewService } from '../features/interviews/services/interview.service'

describe('CreateWorkspaceRequest Payload Keys Verification', () => {
  it('strictly adheres to OpenAPI CreateWorkspaceRequestDto property names', async () => {
    const rawMock = vi.fn().mockResolvedValue({
      _data: {
        id: 'workspace:123',
        user_id: 'usr_1',
        company_name: 'Acme Corp',
        role_title: 'Platform Architect',
        status: 'pending',
        current_phase: 'preparing_input',
        progress_percent: 0,
        created_at: '2026-10-13T00:00:00Z',
        updated_at: '2026-10-13T00:00:00Z',
      },
    })

    const fetchContainer: { raw: typeof rawMock } = { raw: rawMock }
    Reflect.set(globalThis, '$fetch', fetchContainer)

    const payload: CreateWorkspaceRequest = {
      company_name: 'Acme Corp',
      role_title: 'Platform Architect',
      seniority: 'staff',
      target_date: '2026-11-01T00:00:00Z',
      prep_time_weeks: 3,
      company_urls: ['https://example.com/eng'],
      resume_text: 'Experienced distributed systems engineer.',
      job_description: 'Looking for a Rust/Python platform architect.',
    }

    const { createWorkspace } = useInterviewService()
    await createWorkspace(payload)

    expect(rawMock).toHaveBeenCalled()
    const calls = rawMock.mock.calls
    expect(calls.length).toBeGreaterThan(0)
    const callArgs = calls[0]
    expect(callArgs[0]).toBe('/api/backend/workspaces')
    const options = callArgs[1]
    expect(options).toBeDefined()
    const body = options?.body
    expect(body).toEqual(payload)
    expect(body).toHaveProperty('company_name', 'Acme Corp')
    expect(body).toHaveProperty('role_title', 'Platform Architect')
    expect(body).toHaveProperty('seniority', 'staff')
    expect(body).toHaveProperty('target_date', '2026-11-01T00:00:00Z')
    expect(body).toHaveProperty('prep_time_weeks', 3)
    expect(body).toHaveProperty('company_urls')
    expect(body).toHaveProperty('resume_text', 'Experienced distributed systems engineer.')
    expect(body).toHaveProperty('job_description', 'Looking for a Rust/Python platform architect.')
    expect(body).not.toHaveProperty('raw_resume_text')
    expect(body).not.toHaveProperty('raw_job_description_text')
    expect(body).not.toHaveProperty('preparation_time_weeks')
    expect(body).not.toHaveProperty('target_interview_date')
  })
})
