import type { ApplicationStatus, ReasonCode, SiteId } from './constants.js';
import type {
  ApproveApplicationRequestDto,
  BlockApplicationRequestDto,
  CandidateProfileDto,
  CreateSavedJobRequestDto,
  JobApplicationResponseDto,
  PrepareApplicationRequestDto,
  PrepareSavedJobRequestDto,
  ReconcileApplicationRequestDto,
  ResultApplicationRequestDto,
  SavedJobResponseDto,
  SavedJobsPageResponseDto,
  UserResponseDto
} from './contracts.js';
import type { DomDocument } from '../content/adapters/dom-types.js';

export type {
  ApplicationSite,
  ApplicationOutcome,
  ApproveApplicationRequestDto,
  BlockApplicationRequestDto,
  CandidateProfileDto,
  CreateSavedJobRequestDto,
  JobApplicationResponseDto,
  PrepareApplicationRequestDto,
  PrepareSavedJobRequestDto,
  ReconcileApplicationRequestDto,
  ResultApplicationRequestDto,
  ResumeUploadItemDto,
  SavedJobResponseDto,
  SavedJobsPageResponseDto,
  UserResponseDto,
  WorkspaceStatus
} from './contracts.js';

export type StoreAreaName = 'local' | 'session';


export interface Settings {
  apiOrigin: string;
  linkedinOptIn: boolean;
  allowInsecureLoopback: boolean;
}

export interface StoredTokens {
  access_token?: string;
  refresh_token?: string;
  [key: string]: unknown;
}

export interface LoginResult {
  requires2fa: boolean;
  user?: UserResponseDto | null;
}

export interface AuthStatus {
  authenticated: boolean;
  user?: UserResponseDto | null;
}

export interface ResumeOption {
  uploadId: string;
  filename: string;
  createdAt: string;
}

export interface ResumeDownload {
  bytes: Uint8Array;
  filename: string;
  contentType: string;
}

export interface ResumePayload {
  bytes: number[] | Uint8Array;
  filename: string;
  mime: string;
}

export type SavedJob = SavedJobResponseDto;
export type SavedJobsPage = SavedJobsPageResponseDto;

export type PreparedFields = Record<string, unknown>;

export type PrepareApplicationBody = PrepareApplicationRequestDto & {
  application_method?: string;
  contact_email?: string | null;
};

export interface MatchResult {
  eligible: boolean;
  matchedCriteria: string[];
  missingCriteria: string[];
  excludedMatches: string[];
}

export interface ScanBlocked {
  reason: string;
  detail?: string;
}

export interface ScanSubmitDescriptor {
  selector?: string;
  text?: string;
  found?: boolean;
  disabled?: boolean;
}

export interface ScanFormField {
  id?: string;
  name?: string;
  type?: string;
  required?: boolean;
  label?: string;
  autocomplete?: string;
  tag?: string;
  tagName?: string;
  disabled?: boolean;
}

export interface ScanForm {
  selector?: string;
  action?: string;
  method?: string;
  fields?: ScanFormField[];
  submit?: ScanSubmitDescriptor;
}

export interface ScanJob {
  title?: string;
  companyName?: string;
  description?: string;
  location?: string;
  sourceTitle?: string;
}

export interface ScanCapabilities {
  scan?: boolean;
  fill?: boolean;
  submit?: boolean;
  manualSubmitRequired?: boolean;
  verifiedConfirmation?: boolean;
}

export interface ScanConfirmationRules {
  pathSuffix?: string;
  expectedTitlePrefix?: string;
}

export interface Scan {
  supported?: boolean;
  site?: string;
  canonicalUrl?: string;
  sourceUrl?: string;
  job?: ScanJob | null;
  form?: ScanForm | null;
  blocked?: ScanBlocked | null;
  capabilities?: ScanCapabilities;
  evidence?: string;
  confirmation?: ScanConfirmationRules;
  fingerprint?: string;
  applicationRevision?: string;
  resumeUploadId?: string;
}

export interface ScanStructure {
  site: string;
  canonicalUrl: string;
  title: string;
  company: string;
  form: {
    selector: string;
    action: string;
    method: string;
    submit: { selector: string; text: string };
    fields: Array<{
      id: string;
      name: string;
      type: string;
      required: boolean;
      label: string;
      autocomplete: string;
      tag: string;
    }>;
  };
}

export type ReviewPhase =
  | 'prepared'
  | 'approved'
  | 'filled'
  | 'submitting'
  | 'submitted'
  | 'unknown'
  | 'needs_review';

export interface Review {
  tabId: number | string;
  applicationId: string;
  site: string;
  canonicalUrl: string;
  sourceUrl: string;
  companyName: string;
  roleTitle: string;
  revision: string;
  status: ApplicationStatus | ReviewPhase | string;
  preparedFields: PreparedFields;
  resumeUploadId: string;
  fingerprint: string;
  phase: ReviewPhase;
  preparedAt: number;
  approvedAt: number | null;
  filledAt: number | null;
  filledFingerprint: string | null;
  submitUsed: boolean;
  submittedAt: number | null;
  confirmationUrl: string | null;
  reasonCode: string | null;
  updatedAt: string;
}

export interface ReviewSummary {
  applicationId: string;
  site: string;
  canonicalUrl: string;
  companyName: string;
  roleTitle: string;
  phase: ReviewPhase;
  status: string;
  revision: string;
  preparedFields: PreparedFields;
  resumeUploadId: string;
  preparedAt: number;
  approvedAt: number | null;
  filledAt: number | null;
  filledFingerprint: string | null;
  submitUsed: boolean;
  confirmationUrl: string | null;
  reasonCode: string | null;
}

export interface ValidationResult {
  ok: boolean;
  code?: ReasonCode | string;
}

export interface ValidateReviewOptions {
  now?: number;
  ttlMs?: number;
  requireApproved?: boolean;
  requireFilled?: boolean;
  requireUnsubmitted?: boolean;
  valuesFingerprint?: string | null;
}

export interface SubmitState {
  applicationId: string;
  revision: string;
  startedAt: number;
  phase: string;
}

export interface PendingSubmission {
  tabId: number | string;
  applicationId: string;
  revision: string;
  canonicalUrl?: string;
  startedAt: number;
  attempts: number;
  lastOutcome?: string;
  lastEvidence?: Record<string, unknown>;
}

export interface PendingEntry {
  applicationId: string;
  pending: PendingSubmission;
}

export type ApplicationRecord = Partial<JobApplicationResponseDto> & {
  application_method?: string | null;
  recipient_email?: string | null;
  [key: string]: unknown;
};

export interface NormalizedApplication {
  applicationId: string;
  status: string;
  revision: string;
  site: string;
  canonicalUrl: string;
  sourceUrl: string;
  companyName: string;
  roleTitle: string;
  preparedFields: PreparedFields;
  resumeUploadId: string;
  workspaceId: string | null;
  reasonCode: string | null;
  confirmationUrl: string | null;
  duplicate: boolean;
}

export interface ApiClient {
  login(email: string, password: string): Promise<LoginResult>;
  login2fa(code: string): Promise<LoginResult>;
  logout(): Promise<void>;
  authStatus(): Promise<AuthStatus>;
  listResumes(): Promise<ResumeOption[]>;
  downloadResume(uploadId: string): Promise<ResumeDownload>;
  candidateProfile(): Promise<CandidateProfileDto | null>;
  listSavedJobs(cursor?: string, limit?: number): Promise<SavedJobsPage>;
  saveJob(payload: CreateSavedJobRequestDto): Promise<SavedJobResponseDto>;
  approveSavedJob(savedJobId: string): Promise<SavedJobResponseDto>;
  dismissSavedJob(savedJobId: string): Promise<SavedJobResponseDto>;
  prepareSavedJob(savedJobId: string, payload: PrepareSavedJobRequestDto): Promise<JobApplicationResponseDto>;
  prepareApplication(payload: PrepareApplicationBody): Promise<JobApplicationResponseDto>;
  approveApplication(applicationId: string, payload: ApproveApplicationRequestDto): Promise<JobApplicationResponseDto>;
  startApplication(applicationId: string): Promise<JobApplicationResponseDto>;
  blockApplication(applicationId: string, payload: BlockApplicationRequestDto): Promise<JobApplicationResponseDto>;
  resultApplication(applicationId: string, payload: ResultApplicationRequestDto): Promise<JobApplicationResponseDto>;
  reconcileApplication(applicationId: string, payload: ReconcileApplicationRequestDto): Promise<JobApplicationResponseDto>;
  getApplication(applicationId: string): Promise<JobApplicationResponseDto>;
  listApplications(): Promise<JobApplicationResponseDto[]>;
  refreshTokens(): Promise<StoredTokens>;
}

export interface ExtensionStore {
  getSettings(): Promise<Settings>;
  setSettings(settings: Partial<Settings>): Promise<void>;
  getTokens(): Promise<StoredTokens | null>;
  setTokens(tokens: StoredTokens): Promise<void>;
  clearTokens(): Promise<void>;
  getTempToken(): Promise<string | null>;
  setTempToken(token: string): Promise<void>;
  clearTempToken(): Promise<void>;
  getReview(tabId: number | string): Promise<Review | null>;
  setReview(tabId: number | string, review: Review): Promise<void>;
  deleteReview(tabId: number | string): Promise<void>;
  getSubmitState(tabId: number | string): Promise<SubmitState | null>;
  setSubmitState(tabId: number | string, state: SubmitState): Promise<void>;
  deleteSubmitState(tabId: number | string): Promise<void>;
  queuePending(applicationId: string, pending: PendingSubmission): Promise<void>;
  getPending(applicationId: string): Promise<PendingSubmission | null>;
  deletePending(applicationId: string): Promise<void>;
  listPending(): Promise<PendingEntry[]>;
}

export interface ConnectionResult {
  ok: boolean;
  state?: string;
  code?: string;
  message?: string;
  origin?: string;
}

export interface ConnectionOptions {
  fetchImpl?: FetchLike;
  timeoutMs?: number;
}

export interface ConnectionStatus {
  state: string;
  origin?: string;
  code?: string;
  message?: string;
}

export interface ConnectApiOriginInput {
  rawOrigin?: unknown;
  allowInsecureLoopback?: boolean;
  currentSettings?: Partial<Settings> | null;
  store?: ExtensionStore | null;
  checkConnection?: (origin: string) => Promise<ConnectionResult>;
  permissionGranted?: (origin: string) => Promise<boolean> | boolean;
}

export interface ConnectApiOriginResult {
  connected: boolean;
  origin: string;
  connection: ConnectionStatus;
  tokensCleared?: boolean;
}

export interface ViewStateInput {
  apiOrigin?: string;
  connection?: { state?: string; message?: string } | null;
  authenticated?: boolean;
}

export interface DiscoveryEvidence {
  source: string;
  adapter?: string;
  scope?: string;
  verified: boolean;
  selectors: Record<string, string>;
}

export interface JobCompleteness {
  roleTitle: boolean;
  companyName: boolean;
  jobDescription: boolean;
  location: boolean;
  applicationUrl: boolean;
  contactEmail: boolean;
  present: number;
  total: number;
  complete: boolean;
}

export interface DiscoveredJob {
  site: string;
  sourceUrl: string;
  canonicalUrl: string;
  roleTitle: string;
  companyName: string;
  jobDescription: string;
  location: string;
  applicationUrl: string;
  contactEmail: string;
  observedAt: string;
  completeness: JobCompleteness;
  evidence: DiscoveryEvidence;
}

export interface DiscoveredJobInput {
  site?: unknown;
  sourceUrl?: unknown;
  canonicalUrl?: unknown;
  roleTitle?: unknown;
  companyName?: unknown;
  jobDescription?: unknown;
  location?: unknown;
  applicationUrl?: unknown;
  contactEmail?: unknown;
  observedAt?: unknown;
  baseUrl?: unknown;
  evidence?: unknown;
}

export interface DiscoveryResult {
  supported: boolean;
  site: string;
  jobs: DiscoveredJob[];
  blocked?: ScanBlocked | null;
}

export interface ContentMessageResult {
  supported?: boolean;
  site?: string;
  jobs?: DiscoveredJob[];
  blocked?: ScanBlocked | null;
  [key: string]: unknown;
}

export interface FillReport {
  filled: Array<{ key: string; id: string; name: string }>;
  skipped: Array<{ key: string; reason: string; type?: string }>;
  fields: FieldValue[];
  resumeAttached?: boolean;
  resumeRequired?: boolean;
  resume?: AttachmentResult;
  valuesFingerprint?: string;
}

export interface FieldValue {
  id: string;
  name: string;
  type: string;
  value: string;
  checked: boolean;
  file: { name: string; size: number } | null;
}

export interface AttachmentResult {
  attached: boolean;
  reason?: string;
  filename?: string;
  size?: number;
}

export interface ConfirmationResult {
  confirmed: boolean;
  url?: string;
  text?: string;
  selector?: string;
  evidence?: string;
  reason?: string;
}

export interface SubmitResult {
  clicked: boolean;
  manualSubmitRequired?: boolean;
  reason?: string;
  detail?: string;
}

export interface ResumeAttachment {
  bytes: Uint8Array | number[];
  filename?: string;
  mime?: string;
}

export type ScanRoot = DomDocument;

export interface SiteAdapter {
  id: SiteId;
  label: string;
  verified: boolean;
  evidence: string;
  capabilities: ScanCapabilities;
  matches(rawUrl: string): boolean;
  matchesDiscovery?(rawUrl: string): boolean;
  canonicalize(rawUrl: string): string;
  scan(doc: ScanRoot, rawUrl: string): Scan;
  discover?(doc: ScanRoot | null, rawUrl: string, options?: { now?: number }): DiscoveryResult;
  fill(doc: ScanRoot, preparedFields: PreparedFields, resume: ResumeAttachment | null): FillReport;
  submit(doc: ScanRoot): SubmitResult;
  confirmation(doc: ScanRoot, rawUrl: string, expectedJobUrl?: string): ConfirmationResult;
  isConfirmationUrl?(rawUrl: string, expectedJobUrl?: string): boolean;
}

export interface DiscoveryAdapter extends SiteAdapter {
  discover(doc: ScanRoot | null, rawUrl: string, options?: { now?: number }): DiscoveryResult;
  matchesDiscovery(rawUrl: string): boolean;
}

export interface ExtensionMessage {
  type: string;
  tabId?: number;
  url?: string;
  applicationId?: string;
  savedJobId?: string;
  cursor?: string;
  limit?: number;
  resumeUploadId?: string;
  applicationMethod?: string;
  origin?: string;
  allowInsecureLoopback?: boolean;
  timeoutMs?: number;
  optIn?: boolean;
  email?: string;
  password?: string;
  code?: string;
  job?: CreateSavedJobRequestDto;
  scan?: Scan;
  preparedFields?: PreparedFields;
  resume?: ResumePayload | null;
  valuesFingerprint?: string | null;
  expectedJobUrl?: string;
  submitted?: boolean;
  reasonCode?: string;
  [key: string]: unknown;
}

export interface ExtensionResponse<T = unknown> {
  ok: boolean;
  data?: T;
  error?: string;
}

export interface BackgroundState {
  settings: Settings;
  connection: ConnectionStatus;
  auth: AuthStatus;
  profile: CandidateProfileDto | null;
  candidateProfile: CandidateProfileDto | null;
  review: ReviewSummary | null;
  pending: PendingSubmission | null;
}

export interface FetchInit {
  method?: string;
  headers?: Record<string, string>;
  body?: string;
  credentials?: string;
  redirect?: string;
  cache?: string;
  referrerPolicy?: string;
  signal?: AbortSignal;
  [key: string]: unknown;
}

export interface FetchResponse {
  ok: boolean;
  status: number;
  url?: string;
  headers?: { get?(name: string): string | null } | null;
  text?(): Promise<string>;
  json?(): Promise<unknown>;
  arrayBuffer?(): Promise<ArrayBuffer>;
}

export type FetchLike = (url: string, init?: FetchInit) => Promise<FetchResponse>;

export type { ApplicationStatus, Outcome, ReasonCode, ServerBlockReason, SiteId } from './constants.js';
