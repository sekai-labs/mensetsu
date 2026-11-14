import type { SavedJob, SavedJobsPage } from '../../shared/types.js';

export const SAVED_JOBS_PAGE_LIMIT = 25;

export type SavedJobPhase =
  | 'unlinked_awaiting'
  | 'unlinked_approved'
  | 'needs_review'
  | 'submitted'
  | 'queued'
  | 'preparing'
  | 'ready'
  | 'failed'
  | 'continue';

export interface SavedJobView {
  phase: SavedJobPhase;
  status: string;
  hint: string;
}

export function mergeSavedJobsPage(current: SavedJob[], page: SavedJobsPage): SavedJob[] {
  const merged = current.slice();
  const positions = new Map(merged.map((job, index) => [job.id, index] as const));
  for (const job of page.items) {
    const position = positions.get(job.id);
    if (position === undefined) {
      positions.set(job.id, merged.length);
      merged.push(job);
    } else {
      merged[position] = job;
    }
  }
  return merged;
}

export function savedJobView(job: SavedJob): SavedJobView {
  if (!job.job_application_id) {
    if (job.status === 'approved') {
      return { phase: 'unlinked_approved', status: 'Approved for preparation', hint: '' };
    }
    return { phase: 'unlinked_awaiting', status: 'Awaiting your approval', hint: '' };
  }

  const applicationStatus = job.application_status ?? null;
  if (applicationStatus === 'needs_review') {
    return {
      phase: 'needs_review',
      status: 'Job details changed — review and approve again',
      hint: 'The saved job facts changed after approval. Review the updated application and approve it again before preparing.'
    };
  }

  if (applicationStatus === 'submitted') {
    switch (job.workspace_status ?? null) {
      case 'pending':
        return {
          phase: 'queued',
          status: 'Submitted — interview preparation queued',
          hint: 'Interview preparation starts automatically. Refresh to see progress.'
        };
      case 'processing':
        return {
          phase: 'preparing',
          status: 'Submitted — preparing interview materials',
          hint: 'Mensetsu is preparing interview materials in the background.'
        };
      case 'ready':
        return {
          phase: 'ready',
          status: 'Submitted — interview preparation ready',
          hint: 'Open Mensetsu to review the prepared interview materials.'
        };
      case 'failed':
        return {
          phase: 'failed',
          status: 'Submitted — interview preparation failed',
          hint: 'Interview preparation failed. Open Mensetsu to retry.'
        };
      default:
        return {
          phase: 'submitted',
          status: 'Submitted',
          hint: 'Interview preparation status is not available yet.'
        };
    }
  }

  return { phase: 'continue', status: 'Application in progress — continue', hint: '' };
}
