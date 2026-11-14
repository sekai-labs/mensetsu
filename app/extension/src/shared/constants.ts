import type { Settings } from './types.js';

export const MESSAGE = Object.freeze({
  PING: 'mensetsu/ping',
  SCAN: 'mensetsu/scan',
  DISCOVER: 'mensetsu/discover',
  FILL: 'mensetsu/fill',
  SUBMIT: 'mensetsu/submit',
  CHECK_CONFIRMATION: 'mensetsu/check-confirmation',
  GET_STATE: 'mensetsu/get-state',
  CONNECT: 'mensetsu/connect-api',
  LOGIN: 'mensetsu/login',
  LOGIN_2FA: 'mensetsu/login-2fa',
  LOGOUT: 'mensetsu/logout',
  AUTH_STATUS: 'mensetsu/auth-status',
  LIST_RESUMES: 'mensetsu/list-resumes',
  LIST_SAVED_JOBS: 'mensetsu/list-saved-jobs',
  SAVE_JOB: 'mensetsu/save-job',
  APPROVE_SAVED_JOB: 'mensetsu/approve-saved-job',
  DISMISS_SAVED_JOB: 'mensetsu/dismiss-saved-job',
  PREPARE_SAVED_JOB: 'mensetsu/prepare-saved-job',
  CONFIRM_EMAIL_SENT: 'mensetsu/confirm-email-sent',
  CONFIRM_WEB_SUBMITTED: 'mensetsu/confirm-web-submitted',
  GET_APPLICATION: 'mensetsu/get-application',
  PREPARE: 'mensetsu/prepare',
  APPROVE: 'mensetsu/approve',
  BEGIN_FILL: 'mensetsu/begin-fill',
  RECORD_FILL: 'mensetsu/record-fill',
  BEGIN_SUBMIT: 'mensetsu/begin-submit',
  BLOCK: 'mensetsu/block',
  RECONCILE: 'mensetsu/reconcile',
  SET_LINKEDIN_OPT_IN: 'mensetsu/set-linkedin-opt-in'
} as const);

export type MessageType = (typeof MESSAGE)[keyof typeof MESSAGE];

export const SITE = Object.freeze({
  GREENHOUSE: 'greenhouse',
  TOPCV: 'topcv',
  LINKEDIN: 'linkedin'
} as const);

export type SiteId = (typeof SITE)[keyof typeof SITE];

export const APPLICATION_STATUS = Object.freeze([
  'prepared',
  'approved',
  'submitting',
  'submitted',
  'needs_review',
  'unknown'
] as const);

export type ApplicationStatus = (typeof APPLICATION_STATUS)[number];

export const OUTCOME = Object.freeze({
  CONFIRMED: 'confirmed',
  UNCERTAIN: 'uncertain',
  NOT_SUBMITTED: 'not_submitted'
} as const);

export type Outcome = (typeof OUTCOME)[keyof typeof OUTCOME];

export const OUTCOMES: readonly Outcome[] = Object.freeze(Object.values(OUTCOME));

export const SERVER_BLOCK_REASON = Object.freeze({
  MISSING_REQUIRED_FIELD: 'missing_required_field',
  UNSUPPORTED_FORM: 'unsupported_form',
  LOGIN_REQUIRED: 'login_required',
  CAPTCHA: 'captcha',
  FORM_CHANGED: 'form_changed'
} as const);

export type ServerBlockReason = (typeof SERVER_BLOCK_REASON)[keyof typeof SERVER_BLOCK_REASON];

export function toServerBlockReason(code: string): ServerBlockReason {
  switch (code) {
    case 'login_required':
      return SERVER_BLOCK_REASON.LOGIN_REQUIRED;
    case 'captcha':
    case 'captcha_required':
      return SERVER_BLOCK_REASON.CAPTCHA;
    case 'form_changed':
    case 'job_changed':
    case 'values_changed':
      return SERVER_BLOCK_REASON.FORM_CHANGED;
    case 'missing_required_field':
    case 'profile_incomplete':
    case 'resume_missing':
      return SERVER_BLOCK_REASON.MISSING_REQUIRED_FIELD;
    case 'unsupported_form':
    case 'unsupported_site':
    default:
      return SERVER_BLOCK_REASON.UNSUPPORTED_FORM;
  }
}

export const REASON = Object.freeze({
  UNSUPPORTED_SITE: 'unsupported_site',
  LOGIN_REQUIRED: 'login_required',
  CAPTCHA_REQUIRED: 'captcha_required',
  SITE_RESTRICTED: 'site_restricted',
  UNVERIFIED_FORM: 'unverified_form',
  LINKEDIN_OPT_IN_REQUIRED: 'linkedin_opt_in_required',
  UNSUPPORTED_FORM: 'unsupported_form',
  FORM_NOT_FOUND: 'form_not_found',
  FORM_CHANGED: 'form_changed',
  REVIEW_MISSING: 'review_missing',
  REVIEW_EXPIRED: 'review_expired',
  REVIEW_NOT_APPROVED: 'review_not_approved',
  REVIEW_NOT_FILLED: 'review_not_filled',
  VALUES_CHANGED: 'values_changed',
  JOB_CHANGED: 'job_changed',
  JOB_DETAILS_MISSING: 'job_details_missing',
  PROFILE_INCOMPLETE: 'profile_incomplete',
  RESUME_MISSING: 'resume_missing',
  RESUME_TOO_LARGE: 'resume_too_large',
  RESUME_INVALID_TYPE: 'resume_invalid_type',
  SUBMIT_UNAVAILABLE: 'submit_unavailable',
  SUBMIT_IN_PROGRESS: 'submit_in_progress',
  SUBMIT_NOT_ATTEMPTED: 'submit_not_attempted',
  SUBMIT_FAILED: 'submit_failed',
  CONFIRMATION_NOT_FOUND: 'confirmation_not_found',
  ALREADY_SUBMITTED: 'already_submitted',
  DUPLICATE_APPLICATION: 'duplicate_application',
  USER_CANCELLED: 'user_cancelled',
  API_ERROR: 'api_error',
  NETWORK_ERROR: 'network_error',
  API_ORIGIN_MISSING: 'api_origin_missing',
  API_ORIGIN_INVALID: 'api_origin_invalid',
  API_PERMISSION_MISSING: 'api_permission_missing'
} as const);

export type ReasonCode = (typeof REASON)[keyof typeof REASON];

export const REASONS: readonly ReasonCode[] = Object.freeze(Object.values(REASON));

export const REVIEW_TTL_MS = 10 * 60 * 1000;
export const CONFIRMATION_TIMEOUT_MS = 20 * 1000;
export const MAX_RESUME_BYTES = 15 * 1024 * 1024;
export const MAX_JOB_DESCRIPTION_CHARS = 60_000;
export const MAX_FORM_FIELDS = 120;

export const STORAGE_KEY = Object.freeze({
  TOKENS: 'mensetsu.session.tokens',
  TEMP_TOKEN: 'mensetsu.session.temp-token',
  SETTINGS: 'mensetsu.local.settings',
  REVIEW_PREFIX: 'mensetsu.session.review.',
  SUBMIT_PREFIX: 'mensetsu.session.submit.',
  PENDING_PREFIX: 'mensetsu.session.pending.'
} as const);

export const DEFAULT_SETTINGS: Readonly<Settings> = Object.freeze({
  apiOrigin: '',
  linkedinOptIn: false,
  allowInsecureLoopback: false
});
