import {
  OUTCOME,
  REASON,
  REVIEW_TTL_MS,
  toServerBlockReason
} from '../../shared/constants.js';
import {
  computeStructureFingerprint,
  createReview,
  markApproved,
  markFilled,
  markNeedsReview,
  markSubmitted,
  markSubmitting,
  normalizeApplicationEnvelope,
  reviewSummary,
  validateReview
} from '../applications/review.js';
import type {
  ApiClient,
  ApplicationOutcome,
  ApplicationSite,
  ExtensionStore,
  FillReport,
  PendingSubmission,
  PrepareApplicationBody,
  Review,
  ReviewSummary,
  Scan,
  SubmitState,
  ValidateReviewOptions,
  ValidationResult
} from '../../shared/types.js';
import type {
  ApproveApplicationRequestDto,
  BlockApplicationRequestDto,
  ReconcileApplicationRequestDto,
  ResultApplicationRequestDto
} from '../../shared/contracts.js';

export class WorkflowError extends Error {
  readonly code: string;
  readonly details: Record<string, unknown>;

  constructor(code: string, message: string, details: Record<string, unknown> = {}) {
    super(message);
    this.name = 'WorkflowError';
    this.code = code;
    this.details = details;
  }
}

export interface CreateWorkflowOptions {
  api?: ApiClient | null;
  store?: ExtensionStore | null;
  now?: () => number;
  ttlMs?: number;
}

export interface SubmitEvidence {
  confirmed?: boolean;
  outcome?: string;
  confirmationUrl?: string | null;
  reasonCode?: string;
  [key: string]: unknown;
}

export interface PrepareResult {
  review: ReviewSummary | null;
  alreadySubmitted: boolean;
  blocked: boolean;
}

export interface BeginSubmitResult {
  alreadySubmitted: boolean;
  applicationId?: string;
  revision?: string;
  review: ReviewSummary | null;
}

export interface SubmitCompletionResult {
  reported: boolean;
  pending?: boolean;
  error?: string;
  review: ReviewSummary | null;
}

export interface BlockResult {
  reported: boolean;
  reasonCode?: string;
  error?: string;
  review?: ReviewSummary | null;
}

export interface ReviewState {
  review: ReviewSummary | null;
  pending: PendingSubmission | null;
}

export interface Workflow {
  prepare(tabId: number | string, scan: Scan, resumeUploadId?: string): Promise<PrepareResult>;
  approve(tabId: number | string, scan: Scan): Promise<{ review: ReviewSummary | null }>;
  beginFill(tabId: number | string, scan: Scan): Promise<{ applicationId: string; revision: string; fields: Record<string, unknown>; resumeUploadId: string }>;
  recordFill(tabId: number | string, scan: Scan, report: Partial<FillReport> | null | undefined): Promise<{ review: ReviewSummary | null; fill: Partial<FillReport> }>;
  beginSubmit(tabId: number | string, scan: Scan, valuesFingerprint?: string | null): Promise<BeginSubmitResult>;
  completeSubmit(tabId: number | string, evidence?: SubmitEvidence): Promise<SubmitCompletionResult>;
  block(tabId: number | string, reasonCode: string, detail?: string): Promise<BlockResult>;
  reconcile(tabId: number | string, input: { submitted?: boolean }): Promise<{ review: ReviewSummary | null }>;
  recoverPending(): Promise<Array<Record<string, unknown>>>;
  getState(tabId: number | string): Promise<ReviewState>;
  reset(tabId: number | string): Promise<void>;
}

function isHex64(value: unknown): boolean {
  return typeof value === 'string' && /^[0-9a-f]{64}$/.test(value);
}

function asRecord(value: unknown): Record<string, unknown> | null {
  return value && typeof value === 'object' ? value as Record<string, unknown> : null;
}

function assertScan(scan: Scan | null | undefined): void {
  if (!scan || scan.supported !== true) {
    throw new WorkflowError(scan?.blocked?.reason || REASON.UNSUPPORTED_SITE, scan?.blocked?.detail || 'This page is not supported.');
  }
  if (!scan.blocked && (!scan.canonicalUrl || !scan.fingerprint)) {
    throw new WorkflowError(REASON.FORM_NOT_FOUND, 'The job page did not expose a reviewable application form.');
  }
}

export interface SubmitNotice {
  level: 'info' | 'error';
  message: string;
}

export function submitOutcomeNotice(review: ReviewSummary | null): SubmitNotice | null {
  if (!review) return null;
  switch (review.phase) {
    case 'submitted':
      return { level: 'info', message: 'Application submitted and verified at the confirmation route.' };
    case 'unknown':
      return {
        level: 'error',
        message: 'Confirmation could not be automatically verified. Choose Yes below if you submitted the application, or No if you did not.'
      };
    case 'needs_review':
      return {
        level: 'error',
        message: `Submission is unverified (${review.reasonCode || 'confirmation_not_found'}). Choose Yes below if you submitted the application, or No if you did not. Mensetsu will not mark it submitted without your confirmation.`
      };
    default:
      return { level: 'info', message: `Application state: ${review.phase}.` };
  }
}

export function createWorkflow({
  api,
  store,
  now = Date.now,
  ttlMs = REVIEW_TTL_MS
}: CreateWorkflowOptions = {}): Workflow {
  if (!api || !store) throw new Error('createWorkflow requires api and store');
  const activeApi = api;
  const activeStore = store;
  const locks = new Set<string>();

  async function requireReview(tabId: number | string): Promise<Review> {
    const review = await activeStore.getReview(tabId);
    if (!review) throw new WorkflowError(REASON.REVIEW_MISSING, 'Prepare this job before continuing.');
    return review;
  }

  function check(review: Review, scan: Scan, options: ValidateReviewOptions = {}): void {
    const result: ValidationResult = validateReview(review, scan, { now: now(), ttlMs, ...options });
    if (!result.ok) throw new WorkflowError(String(result.code), `Review guard rejected the action: ${result.code}`);
  }

  async function markStale(tabId: number | string, review: Review, code: string): Promise<Review> {
    const stale = markNeedsReview(review, code, now());
    await activeStore.setReview(tabId, stale);
    return stale;
  }

  async function findDuplicate(error: unknown, scan: Scan) {
    const body = asRecord(asRecord(error)?.body);
    const directId = body?.application_id ?? body?.id ?? body?.applicationId;
    if (directId) {
      try {
        return normalizeApplicationEnvelope(await activeApi.getApplication(String(directId)));
      } catch {
        return null;
      }
    }
    try {
      const applications = await activeApi.listApplications();
      const match = applications.find((item) => {
        const canonical = String(item.canonical_url ?? '');
        return canonical === String(scan.canonicalUrl ?? '');
      });
      return match ? normalizeApplicationEnvelope(match) : null;
    } catch {
      return null;
    }
  }

  async function reconcile(tabId: number | string, { submitted }: { submitted?: boolean }): Promise<{ review: ReviewSummary | null }> {
    const review = await activeStore.getReview(tabId);
    if (!review) throw new WorkflowError(REASON.REVIEW_MISSING, 'There is no prepared application for this tab.');
    const response = await activeApi.reconcileApplication(review.applicationId, {
      submitted: Boolean(submitted)
    });
    const application = normalizeApplicationEnvelope(response);
    const next = application.status === 'submitted'
      ? markSubmitted(review, { confirmationUrl: application.confirmationUrl, now: now() })
      : markNeedsReview(review, application.reasonCode || REASON.CONFIRMATION_NOT_FOUND, now());
    await activeStore.setReview(tabId, next);
    await activeStore.deletePending(review.applicationId);
    return { review: reviewSummary(next) };
  }

  return {
    async prepare(tabId: number | string, scan: Scan, resumeUploadId?: string): Promise<PrepareResult> {
      assertScan(scan);

      const resolvedResumeId = resumeUploadId || scan.resumeUploadId || '';
      if (!resolvedResumeId) {
        throw new WorkflowError(REASON.RESUME_MISSING, 'Select a completed resume upload before preparing.');
      }

      let fingerprint = scan.fingerprint;
      if (!scan.blocked) {
        fingerprint = await computeStructureFingerprint(scan);
        if (fingerprint !== scan.fingerprint) {
          throw new WorkflowError(REASON.FORM_CHANGED, 'The form changed while it was being prepared.');
        }
      }

      const companyName = String(scan.job?.companyName || '').trim();
      const roleTitle = String(scan.job?.title || '').trim();
      const jobDescription = String(scan.job?.description || '').trim();
      if (!companyName || !roleTitle || !jobDescription) {
        throw new WorkflowError(REASON.JOB_DETAILS_MISSING, 'Could not verify the company, role, and job description on this page.');
      }
      const payload: PrepareApplicationBody = {
        site: String(scan.site ?? '') as ApplicationSite,
        source_url: String(scan.sourceUrl || scan.canonicalUrl || ''),
        company_name: companyName,
        role_title: roleTitle,
        job_description: jobDescription,
        resume_upload_id: resolvedResumeId,
        application_method: 'web'
      };

      let response: unknown;
      try {
        response = await activeApi.prepareApplication(payload);
      } catch (error) {
        if ((error as { status?: number })?.status === 409) {
          const duplicate = await findDuplicate(error, scan);
          if (duplicate) {
            response = duplicate;
          } else {
            throw new WorkflowError(REASON.DUPLICATE_APPLICATION, 'This job was already prepared.', { cause: error });
          }
        } else {
          throw error;
        }
      }

      const application = normalizeApplicationEnvelope(response);
      if (application.canonicalUrl && application.canonicalUrl !== scan.canonicalUrl) {
        const expected = new URL(String(scan.canonicalUrl));
        const actual = new URL(String(application.canonicalUrl));
        if (expected.pathname !== actual.pathname && !expected.pathname.includes(actual.pathname) && !actual.pathname.includes(expected.pathname)) {
          throw new WorkflowError(REASON.JOB_CHANGED, 'The server returned a different canonical job URL.');
        }
      }

      const review = await createReview({ tabId, scan: { ...scan, fingerprint: fingerprint || 'unverified' }, application, now: now() });
      if (application.status === 'submitted') {
        review.phase = 'submitted';
        review.status = 'submitted';
        review.submitUsed = true;
        review.confirmationUrl = (application.confirmationUrl as string | null) ?? null;
      }

      if (scan.blocked) {
        const serverCode = toServerBlockReason(scan.blocked.reason);
      const blockPayload: BlockApplicationRequestDto = { reason_code: serverCode };
        try {
          await activeApi.blockApplication(application.applicationId, blockPayload);
        } catch {}
        review.phase = 'needs_review';
        review.status = 'needs_review';
        review.reasonCode = scan.blocked.reason;
      }

      await activeStore.setReview(tabId, review);
      return {
        review: reviewSummary(review),
        alreadySubmitted: application.status === 'submitted',
        blocked: Boolean(scan.blocked)
      };
    },

    async approve(tabId: number | string, scan: Scan): Promise<{ review: ReviewSummary | null }> {
      const review = await requireReview(tabId);
      check(review, scan);
      if (review.phase === 'submitted') throw new WorkflowError(REASON.ALREADY_SUBMITTED, 'This application is already submitted.');
      if (review.phase === 'submitting') throw new WorkflowError(REASON.SUBMIT_IN_PROGRESS, 'This application is already being submitted.');

      const response = await activeApi.approveApplication(review.applicationId, {
        revision: review.revision
      } satisfies ApproveApplicationRequestDto);
      const application = normalizeApplicationEnvelope(response);
      if (application.revision && review.revision && application.revision !== review.revision) {
        const stale = await markStale(tabId, review, REASON.JOB_CHANGED);
        throw new WorkflowError(REASON.JOB_CHANGED, 'The server revision changed; review the job again.', { review: stale });
      }
      const approved = markApproved({ ...review, revision: application.revision || review.revision }, now());
      await activeStore.setReview(tabId, approved);
      return { review: reviewSummary(approved) };
    },

    async beginFill(tabId: number | string, scan: Scan) {
      const review = await requireReview(tabId);
      check(review, scan, { requireApproved: true, requireUnsubmitted: true });
      if (!review.preparedFields || Object.keys(review.preparedFields).length === 0) {
        throw new WorkflowError(REASON.PROFILE_INCOMPLETE, 'The server returned no prepared profile fields.');
      }
      return {
        applicationId: review.applicationId,
        revision: review.revision,
        fields: review.preparedFields as Record<string, unknown>,
        resumeUploadId: review.resumeUploadId
      };
    },

    async recordFill(tabId: number | string, scan: Scan, report: Partial<FillReport> | null | undefined): Promise<{ review: ReviewSummary | null; fill: Partial<FillReport> }> {
      const review = await requireReview(tabId);
      check(review, scan, { requireApproved: true, requireUnsubmitted: true });
      if (!report || !isHex64(report.valuesFingerprint)) {
        throw new WorkflowError(REASON.FORM_CHANGED, 'The page did not return a valid filled-form fingerprint.');
      }
      if (report.resumeRequired === true && report.resumeAttached !== true) {
        throw new WorkflowError(REASON.RESUME_MISSING, 'The site requires a resume attachment and it was not attached.');
      }
      const filled = markFilled(review, String(report.valuesFingerprint), now());
      await activeStore.setReview(tabId, filled);
      return { review: reviewSummary(filled), fill: report };
    },

    async beginSubmit(tabId: number | string, scan: Scan, valuesFingerprint?: string | null): Promise<BeginSubmitResult> {
      const review = await requireReview(tabId);
      check(review, scan, {
        requireApproved: true,
        requireFilled: true,
        requireUnsubmitted: true,
        valuesFingerprint
      });
      if (locks.has(review.applicationId)) {
        throw new WorkflowError(REASON.SUBMIT_IN_PROGRESS, 'A submit is already in progress for this application.');
      }
      locks.add(review.applicationId);
      try {
        let response: unknown;
        try {
          response = await activeApi.startApplication(review.applicationId);
        } catch (error) {
          if ((error as { status?: number })?.status === 409) {
            const stale = await markStale(tabId, review, REASON.ALREADY_SUBMITTED);
            throw new WorkflowError(REASON.ALREADY_SUBMITTED, 'The server refused a second submit for this application.', { review: stale });
          }
          throw error;
        }
        const application = normalizeApplicationEnvelope(response);
        if (application.status === 'submitted') {
          const submitted = markSubmitted(review, { confirmationUrl: application.confirmationUrl, now: now() });
          await activeStore.setReview(tabId, submitted);
          return { alreadySubmitted: true, review: reviewSummary(submitted) };
        }

        const submitting = markSubmitting(review, now());
        await activeStore.setReview(tabId, submitting);
        const submitState: SubmitState = {
          applicationId: review.applicationId,
          revision: review.revision,
          startedAt: now(),
          phase: 'clicked_pending'
        };
        await activeStore.setSubmitState(tabId, submitState);
        await activeStore.queuePending(review.applicationId, {
          tabId,
          applicationId: review.applicationId,
          revision: review.revision,
          canonicalUrl: review.canonicalUrl,
          startedAt: now(),
          attempts: 0
        });
        return {
          alreadySubmitted: false,
          applicationId: review.applicationId,
          revision: review.revision,
          review: reviewSummary(submitting)
        };
      } catch (error) {
        locks.delete(review.applicationId);
        throw error;
      }
    },

    async completeSubmit(tabId: number | string, evidence: SubmitEvidence = {}): Promise<SubmitCompletionResult> {
      const review = await requireReview(tabId);
      const outcome: ApplicationOutcome = evidence.confirmed
        ? OUTCOME.CONFIRMED
        : evidence.outcome === OUTCOME.NOT_SUBMITTED
          ? OUTCOME.NOT_SUBMITTED
          : evidence.outcome === OUTCOME.CONFIRMED
            ? OUTCOME.CONFIRMED
            : OUTCOME.UNCERTAIN;
      const pending = await activeStore.getPending(review.applicationId);
      try {
        const resultPayload: ResultApplicationRequestDto = {
          outcome,
          confirmation_url: evidence.confirmationUrl ?? null
        };
        await activeApi.resultApplication(review.applicationId, resultPayload);
        await activeStore.deletePending(review.applicationId);

        let next: Review;
        if (outcome === OUTCOME.CONFIRMED) {
          next = markSubmitted(review, { confirmationUrl: evidence.confirmationUrl, now: now() });
        } else if (outcome === OUTCOME.UNCERTAIN) {
          next = {
            ...review,
            phase: 'unknown',
            status: 'unknown',
            confirmationUrl: evidence.confirmationUrl || null,
            updatedAt: new Date(now()).toISOString()
          };
        } else {
          next = markNeedsReview(review, evidence.reasonCode || REASON.CONFIRMATION_NOT_FOUND, now());
        }
        await activeStore.setReview(tabId, next);
        return { reported: true, review: reviewSummary(next) };
      } catch (error) {
        await activeStore.queuePending(review.applicationId, {
          ...(pending || {}),
          tabId,
          applicationId: review.applicationId,
          revision: review.revision,
          startedAt: pending?.startedAt ?? now(),
          lastOutcome: outcome,
          lastEvidence: evidence,
          attempts: Number(pending?.attempts || 0) + 1
        });
        const next = markNeedsReview(review, evidence.reasonCode || REASON.API_ERROR, now());
        await activeStore.setReview(tabId, next);
        return { reported: false, pending: true, error: String((error as Error)?.message || error), review: reviewSummary(next) };
      } finally {
        locks.delete(review.applicationId);
        await activeStore.deleteSubmitState(tabId);
      }
    },

    async block(tabId: number | string, reasonCode: string, _detail = ''): Promise<BlockResult> {
      const review = await activeStore.getReview(tabId);
      if (!review) return { reported: false, reasonCode };
      const serverCode = toServerBlockReason(reasonCode);
      const blockPayload: BlockApplicationRequestDto = { reason_code: serverCode };
      try {
        await activeApi.blockApplication(review.applicationId, blockPayload);
      } catch (error) {
        const next = markNeedsReview(review, reasonCode, now());
        await activeStore.setReview(tabId, next);
        return { reported: false, error: String((error as Error)?.message || error), review: reviewSummary(next) };
      }
      const next = markNeedsReview(review, reasonCode, now());
      await activeStore.setReview(tabId, next);
      return { reported: true, review: reviewSummary(next) };
    },

    async reconcile(tabId: number | string, input: { submitted?: boolean }) {
      return reconcile(tabId, input);
    },

    async recoverPending(): Promise<Array<Record<string, unknown>>> {
      const pending = await activeStore.listPending();
      const results: Array<Record<string, unknown>> = [];
      for (const item of pending) {
        const review = await activeStore.getReview(item.pending?.tabId);
        if (!review) {
          await activeStore.deletePending(item.applicationId);
          continue;
        }
        try {
          const result = await reconcile(item.pending.tabId, {
            submitted: false
          } satisfies ReconcileApplicationRequestDto);
          results.push({ applicationId: item.applicationId, recovered: true, review: result.review });
        } catch (error) {
          results.push({ applicationId: item.applicationId, recovered: false, error: String((error as Error)?.message || error) });
        }
      }
      return results;
    },

    async getState(tabId: number | string): Promise<ReviewState> {
      const review = await activeStore.getReview(tabId);
      return {
        review: reviewSummary(review),
        pending: review ? await activeStore.getPending(review.applicationId) : null
      };
    },

    async reset(tabId: number | string): Promise<void> {
      const review = await activeStore.getReview(tabId);
      if (review) {
        locks.delete(review.applicationId);
        await activeStore.deletePending(review.applicationId);
      }
      await activeStore.deleteReview(tabId);
      await activeStore.deleteSubmitState(tabId);
    }
  };
}
