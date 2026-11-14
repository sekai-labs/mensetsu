import { describe, it } from 'node:test';
import assert from 'node:assert/strict';

import { mergeSavedJobsPage, savedJobView } from '../src/features/jobs/saved-jobs.js';
import type { SavedJob, SavedJobsPage } from '../src/shared/types.js';

function savedJob(overrides: Partial<SavedJob> = {}): SavedJob {
  return {
    id: 'saved_job:1',
    source_url: 'https://www.linkedin.com/jobs/view/1',
    canonical_url: 'https://www.linkedin.com/jobs/view/1',
    source_site: 'linkedin',
    status: 'awaiting_approval',
    role_title: 'Backend Engineer',
    company_name: 'Example Company',
    job_description: 'Build and operate services.',
    location: 'Remote',
    application_url: null,
    contact_email: null,
    job_application_id: null,
    created_at: '2026-11-12T00:00:00Z',
    updated_at: '2026-11-12T00:00:00Z',
    application_status: null,
    workspace_status: null,
    ...overrides
  };
}

function page(items: SavedJob[], next_cursor: string | null): SavedJobsPage {
  return { items, next_cursor };
}

describe('Saved jobs page state projection', () => {
  it('separates unlinked awaiting and approved jobs', () => {
    assert.deepStrictEqual(savedJobView(savedJob()).phase, 'unlinked_awaiting');
    assert.strictEqual(savedJobView(savedJob({ status: 'approved' })).phase, 'unlinked_approved');
  });

  it('projects submitted applications from workspace status', () => {
    const submitted = (workspaceStatus: SavedJob['workspace_status']): SavedJob => savedJob({
      status: 'approved',
      job_application_id: 'job_application:1',
      application_status: 'submitted',
      workspace_status: workspaceStatus
    });

    assert.strictEqual(savedJobView(submitted('pending')).phase, 'queued');
    assert.match(savedJobView(submitted('pending')).status, /queued/i);
    assert.strictEqual(savedJobView(submitted('processing')).phase, 'preparing');
    assert.strictEqual(savedJobView(submitted('ready')).phase, 'ready');
    assert.strictEqual(savedJobView(submitted('failed')).phase, 'failed');
    assert.strictEqual(savedJobView(submitted(null)).phase, 'submitted');
  });

  it('keeps the actionable review state for changed job facts', () => {
    const view = savedJobView(savedJob({
      job_application_id: 'job_application:1',
      application_status: 'needs_review',
      workspace_status: null
    }));
    assert.strictEqual(view.phase, 'needs_review');
    assert.match(view.hint, /approve it again/i);
  });

  it('offers continue for every other linked application state', () => {
    for (const applicationStatus of ['prepared', 'approved', 'submitting', 'unknown'] as const) {
      const view = savedJobView(savedJob({
        job_application_id: 'job_application:1',
        application_status: applicationStatus,
        workspace_status: 'processing'
      }));
      assert.strictEqual(view.phase, 'continue');
    }
  });

});

describe('Saved jobs page merging', () => {
  it('appends one page without duplicating ids', () => {
    const first = savedJob({ id: 'saved_job:1' });
    const second = savedJob({ id: 'saved_job:2' });
    const merged = mergeSavedJobsPage([first], page([second, first], 'cursor_2'));
    assert.deepStrictEqual(merged.map((job) => job.id), ['saved_job:1', 'saved_job:2']);
  });

  it('replaces stale copies of an already listed job', () => {
    const first = savedJob({ id: 'saved_job:1', status: 'awaiting_approval' });
    const refreshed = savedJob({ id: 'saved_job:1', status: 'approved' });
    const merged = mergeSavedJobsPage([first], page([refreshed], null));
    assert.strictEqual(merged.length, 1);
    assert.strictEqual(merged[0].status, 'approved');
  });
});
