import { APPLICATION_STATUS, REASON, REVIEW_TTL_MS } from '../../shared/constants.js';
import { isNonEmptyString, nowIso, sha256Hex, stableStringify } from '../../shared/util.js';
import type {
  ApplicationRecord,
  FieldValue,
  NormalizedApplication,
  PreparedFields,
  Review,
  ReviewPhase,
  ReviewSummary,
  Scan,
  ScanFormField,
  ScanStructure,
  ValidateReviewOptions,
  ValidationResult
} from '../../shared/types.js';

export function normalizeApplicationEnvelope(value: unknown): NormalizedApplication {
  if (!value || typeof value !== 'object') {
    throw new Error('Application response was empty.');
  }
  const record = value as ApplicationRecord;
  const applicationId = String(record.application_id ?? record.id ?? '');
  if (!applicationId) throw new Error('Application response did not include an id.');
  const status = String(record.status ?? 'prepared');
  if (!(APPLICATION_STATUS as readonly string[]).includes(status)) {
    throw new Error(`Application response used an unknown status: ${status}`);
  }
  return {
    applicationId,
    status,
    revision: String(record.revision ?? ''),
    site: String(record.site ?? ''),
    canonicalUrl: String(record.canonical_url ?? ''),
    sourceUrl: String(record.source_url ?? ''),
    companyName: String(record.company_name ?? ''),
    roleTitle: String(record.role_title ?? ''),
    preparedFields: (record.prepared_fields ?? {}) as unknown as PreparedFields,
    resumeUploadId: String(record.resume_upload_id ?? ''),
    workspaceId: String(record.workspace_id ?? '') || null,
    reasonCode: ruleText(record.reason_code),
    confirmationUrl: ruleText(record.confirmation_url),
    duplicate: record.duplicate === true
  };
}

function ruleText(value: unknown): string | null {
  return value === null || value === undefined || value === '' ? null : String(value);
}

function fieldDescriptor(field: ScanFormField | null | undefined): ScanStructure['form']['fields'][number] {
  return {
    id: String(field?.id ?? ''),
    name: String(field?.name ?? ''),
    type: String(field?.type ?? ''),
    required: field?.required === true,
    label: String(field?.label ?? ''),
    autocomplete: String(field?.autocomplete ?? ''),
    tag: String(field?.tag ?? field?.tagName ?? '')
  };
}

export function scanStructure(scan: Scan | null | undefined): ScanStructure {
  return {
    site: String(scan?.site ?? ''),
    canonicalUrl: String(scan?.canonicalUrl ?? ''),
    title: String(scan?.job?.title ?? ''),
    company: String(scan?.job?.companyName ?? ''),
    form: {
      selector: String(scan?.form?.selector ?? ''),
      action: String(scan?.form?.action ?? ''),
      method: String(scan?.form?.method ?? ''),
      submit: {
        selector: String(scan?.form?.submit?.selector ?? ''),
        text: String(scan?.form?.submit?.text ?? '')
      },
      fields: (scan?.form?.fields || []).map(fieldDescriptor)
    }
  };
}

export async function computeStructureFingerprint(scan: Scan | null | undefined): Promise<string> {
  return sha256Hex(stableStringify(scanStructure(scan)));
}

export async function computeValuesFingerprint(fields: ReadonlyArray<FieldValue | null | undefined> | null | undefined): Promise<string> {
  const normalized = (fields || []).map((field) => ({
    id: String(field?.id ?? ''),
    name: String(field?.name ?? ''),
    type: String(field?.type ?? ''),
    value: field?.type === 'file'
      ? { name: String(field?.file?.name ?? ''), size: Number(field?.file?.size ?? 0) }
      : String(field?.value ?? ''),
    checked: field?.type === 'checkbox' || field?.type === 'radio' ? Boolean(field?.checked) : null
  }));
  return sha256Hex(stableStringify(normalized));
}

export interface CreateReviewInput {
  tabId: number | string;
  scan: Scan;
  application: NormalizedApplication;
  now?: number;
}

export async function createReview({ tabId, scan, application, now = Date.now() }: CreateReviewInput): Promise<Review> {
  return {
    tabId,
    applicationId: application.applicationId,
    site: String(scan.site ?? ''),
    canonicalUrl: String(scan.canonicalUrl ?? ''),
    sourceUrl: String(scan.sourceUrl || scan.canonicalUrl || ''),
    companyName: scan.job?.companyName || application.companyName || '',
    roleTitle: scan.job?.title || application.roleTitle || '',
    revision: application.revision,
    status: application.status,
    preparedFields: application.preparedFields || {},
    resumeUploadId: application.resumeUploadId || '',
    fingerprint: await computeStructureFingerprint(scan),
    phase: 'prepared',
    preparedAt: now,
    approvedAt: null,
    filledAt: null,
    filledFingerprint: null,
    submitUsed: false,
    submittedAt: null,
    confirmationUrl: null,
    reasonCode: null,
    updatedAt: nowIso(now)
  };
}

export function validateReview(
  review: Review | null | undefined,
  scan: Scan | null | undefined,
  options: ValidateReviewOptions = {}
): ValidationResult {
  const now = options.now ?? Date.now();
  const ttlMs = options.ttlMs ?? REVIEW_TTL_MS;
  if (!review) return { ok: false, code: REASON.REVIEW_MISSING };
  if (!scan || scan.supported !== true) return { ok: false, code: scan?.blocked?.reason || REASON.UNSUPPORTED_SITE };
  if (review.site !== scan.site) return { ok: false, code: REASON.JOB_CHANGED };
  if (review.canonicalUrl !== scan.canonicalUrl) return { ok: false, code: REASON.JOB_CHANGED };
  if (review.fingerprint !== scan.fingerprint) return { ok: false, code: REASON.FORM_CHANGED };
  if (review.revision && scan.applicationRevision && review.revision !== scan.applicationRevision) {
    return { ok: false, code: REASON.JOB_CHANGED };
  }
  if (!Number.isFinite(review.preparedAt) || now - review.preparedAt > ttlMs) {
    return { ok: false, code: REASON.REVIEW_EXPIRED };
  }
  if (options.requireApproved && !['approved', 'filled', 'submitting'].includes(review.phase)) {
    return { ok: false, code: REASON.REVIEW_NOT_APPROVED };
  }
  if (options.requireFilled) {
    if (!review.filledFingerprint) return { ok: false, code: REASON.REVIEW_NOT_FILLED };
    if (options.valuesFingerprint && review.filledFingerprint !== options.valuesFingerprint) {
      return { ok: false, code: REASON.VALUES_CHANGED };
    }
  }
  if (options.requireUnsubmitted && review.submitUsed) {
    return { ok: false, code: REASON.ALREADY_SUBMITTED };
  }
  return { ok: true };
}

export function markApproved(review: Review, now = Date.now()): Review {
  return {
    ...review,
    phase: 'approved',
    approvedAt: now,
    status: 'approved',
    updatedAt: nowIso(now)
  };
}

export function markFilled(review: Review, valuesFingerprint: string, now = Date.now()): Review {
  return {
    ...review,
    phase: 'filled',
    filledAt: now,
    filledFingerprint: valuesFingerprint,
    updatedAt: nowIso(now)
  };
}

export function markSubmitting(review: Review, now = Date.now()): Review {
  return {
    ...review,
    phase: 'submitting',
    status: 'submitting',
    submitUsed: true,
    updatedAt: nowIso(now)
  };
}

export function markSubmitted(
  review: Review,
  { confirmationUrl, now = Date.now() }: { confirmationUrl?: unknown; now?: number }
): Review {
  return {
    ...review,
    phase: 'submitted',
    status: 'submitted',
    submittedAt: now,
    confirmationUrl: (confirmationUrl as string | null) || null,
    updatedAt: nowIso(now)
  };
}

export function markNeedsReview(review: Review, reasonCode: unknown, now = Date.now()): Review {
  return {
    ...review,
    phase: 'needs_review',
    status: 'needs_review',
    reasonCode: (reasonCode as string | null) || null,
    updatedAt: nowIso(now)
  };
}

export function reviewSummary(review: Review | null | undefined): ReviewSummary | null {
  if (!review) return null;
  return {
    applicationId: review.applicationId,
    site: review.site,
    canonicalUrl: review.canonicalUrl,
    companyName: review.companyName,
    roleTitle: review.roleTitle,
    phase: review.phase as ReviewPhase,
    status: String(review.status),
    revision: review.revision,
    preparedFields: review.preparedFields,
    resumeUploadId: review.resumeUploadId,
    preparedAt: review.preparedAt,
    approvedAt: review.approvedAt,
    filledAt: review.filledAt,
    filledFingerprint: review.filledFingerprint,
    submitUsed: review.submitUsed,
    confirmationUrl: review.confirmationUrl,
    reasonCode: review.reasonCode
  };
}

export function assertApplicationId(value: unknown): string {
  if (!isNonEmptyString(value)) throw new Error('applicationId is required');
  return value;
}
