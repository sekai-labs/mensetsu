import { MESSAGE, SITE } from '../shared/constants.js';
import { normalizeApiOrigin } from '../shared/api-origin.js';
import { deriveViewState, VIEW } from '../features/connection/onboarding.js';
import { queryActiveTab, requestHostPermission, sendRuntimeMessage } from '../shared/browser-api.js';
import { matchDiscoveredJob } from '../features/jobs/matching.js';
import { createApplicationEmailDraft, isValidApplicationEmail } from '../features/applications/email-draft.js';
import { mergeSavedJobsPage, SAVED_JOBS_PAGE_LIMIT, savedJobView } from '../features/jobs/saved-jobs.js';
import { safeHttpsUrl } from '../shared/url-safety.js';
import type { ViewName } from '../features/connection/onboarding.js';
import type { ExtensionTab } from '../shared/webextension.js';
import type { ApplicationEmailDraft } from '../features/applications/email-draft.js';
import type {
  BeginSubmitResult,
  BlockResult,
  PrepareResult,
  SubmitCompletionResult
} from '../features/workflows/workflow.js';
import { submitOutcomeNotice } from '../features/workflows/workflow.js';
import type {
  BackgroundState,
  ConnectApiOriginResult,
  CreateSavedJobRequestDto,
  DiscoveredJob,
  DiscoveryResult,
  ExtensionMessage,
  ExtensionResponse,
  FillReport,
  JobApplicationResponseDto,
  LoginResult,
  ResumeOption,
  ReviewSummary,
  SavedJob,
  SavedJobsPage,
  Scan,
  Settings
} from '../shared/types.js';

import { appendText, type PopupElement } from './popup-dom.js';
type UiElement = PopupElement;

let activeTab: ExtensionTab | null = null;
let currentScan: Scan | null = null;
let currentReview: ReviewSummary | null = null;
let currentResumes: ResumeOption[] = [];
let state: BackgroundState | null = null;
let currentDiscovery: DiscoveryResult | null = null;
let savedJobs: SavedJob[] = [];
let savedJobsCursor: string | null = null;
let savedJobsLoading = false;
let currentEmailApplication: JobApplicationResponseDto | null = null;
let currentEmailRecipient = '';
let currentWebsiteApplication: JobApplicationResponseDto | null = null;

function $(selector: string): UiElement {
  return document.querySelector(selector) as unknown as UiElement;
}

function showAlert(message: string | null, type: 'error' | 'info' = 'error'): void {
  const box = $('#alert-box');
  if (!message) {
    box.classList.add('hidden');
    box.textContent = '';
    return;
  }
  box.className = `alert-box alert-${type}`;
  box.textContent = message;
  box.classList.remove('hidden');
}

async function sendBackground<T = unknown>(msg: ExtensionMessage): Promise<T> {
  const res = await sendRuntimeMessage<ExtensionResponse<T>>(msg);
  if (!res?.ok) {
    throw new Error(res?.error || 'Operation failed');
  }
  return res.data as T;
}

function preferredMode(): 'apply' | 'discover' {
  try {
    const host = new URL(activeTab?.url || '').hostname.toLowerCase();
    return ['linkedin.com', 'www.linkedin.com', 'topcv.vn', 'www.topcv.vn'].includes(host)
      ? 'discover'
      : 'apply';
  } catch {
    return 'apply';
  }
}

async function setMode(mode: 'apply' | 'discover'): Promise<void> {
  const discovering = mode === 'discover';
  $('#discover-mode').classList.toggle('hidden', !discovering);
  $('#apply-mode').classList.toggle('hidden', discovering);
  $('#discover-mode-btn').setAttribute('aria-pressed', String(discovering));
  $('#apply-mode-btn').setAttribute('aria-pressed', String(!discovering));
  if (!discovering) {
    $('#resume-card').classList.remove('hidden');
    if (state?.auth?.authenticated) await loadSavedJobs();
  }
}

async function loadActiveTab(): Promise<void> {
  try {
    const tabs = await queryActiveTab();
    activeTab = tabs?.[0] || null;
  } catch {
    activeTab = null;
  }
  if (activeTab?.url) {
    $('#tab-url').textContent = activeTab.url;
  }
}

function setBadge(text: string, kind: string): void {
  const badge = $('#connection-badge');
  badge.className = `badge badge-${kind}`;
  badge.textContent = text;
}

function applySettings(settings: Settings | null | undefined): void {
  const origin = settings?.apiOrigin || '';
  const allowLoopback = settings?.allowInsecureLoopback === true;
  $('#setup-api-origin').value = origin;
  $('#setup-allow-loopback-check').checked = allowLoopback;
  $('#api-origin-input').value = origin;
  $('#allow-loopback-check').checked = allowLoopback;
  $('#linkedin-optin-check').checked = settings?.linkedinOptIn === true;

  const originState = $('#settings-origin-state');
  if (!origin) {
    originState.textContent = 'No API origin is saved yet.';
  } else if (state?.connection?.state === 'healthy') {
    originState.textContent = `Connected to ${origin}.`;
  } else {
    originState.textContent = `Saved origin ${origin} is not currently reachable.`;
  }
}

function renderView(view: ViewName): void {
  const setupVisible = view === VIEW.SETUP || view === VIEW.CONNECTION_ERROR;
  $('#setup-panel').classList.toggle('hidden', !setupVisible);
  $('#auth-panel').classList.toggle('hidden', view !== VIEW.LOGIN);
  $('#main-panel').classList.toggle('hidden', view !== VIEW.WORKSPACE);
  $('#toggle-settings-btn').classList.toggle('hidden', setupVisible);

  if (setupVisible) {
    $('#settings-panel').classList.add('hidden');
    const feedback = $('#setup-feedback');
    if (view === VIEW.CONNECTION_ERROR) {
      feedback.textContent = state?.connection?.message || 'The configured API could not be verified.';
      feedback.className = 'hint-error';
      setBadge('Connection failed', 'warning');
    } else {
      feedback.textContent = 'Enter the Mensetsu API base URL to continue.';
      feedback.className = 'hint';
      setBadge('API URL required', 'warning');
    }
  } else if (view === VIEW.LOGIN) {
    setBadge('Connected', 'success');
  } else {
    setBadge('Authenticated', 'success');
  }
}

async function refreshState(): Promise<void> {
  showAlert(null);
  try {
    state = await sendBackground<BackgroundState>({
      type: MESSAGE.GET_STATE,
      tabId: activeTab?.id
    });

    applySettings(state.settings);

    const view = deriveViewState({
      apiOrigin: state.settings?.apiOrigin,
      connection: state.connection,
      authenticated: state.auth?.authenticated === true
    });
    renderView(view);

    if (view === VIEW.WORKSPACE) {
      const user = state.auth?.user || state.candidateProfile;
      $('#user-name').textContent = (user && 'name' in user ? String(user.name) : '') || 'Candidate';
      $('#user-email').textContent = (user && 'email' in user ? String(user.email) : '') || '';
      await loadResumes();
      if (state.review) {
        renderReview(state.review);
      }
      await setMode(preferredMode());
    }
  } catch (error) {
    const message = String((error as Error)?.message || error);
    setBadge('Connection failed', 'warning');
    $('#auth-panel').classList.add('hidden');
    $('#main-panel').classList.add('hidden');
    $('#setup-panel').classList.remove('hidden');
    $('#toggle-settings-btn').classList.add('hidden');
    $('#setup-feedback').textContent = message;
    $('#setup-feedback').className = 'hint-error';
    showAlert(message, 'error');
  }
}

interface RunConnectionInput {
  originInput: UiElement;
  loopbackInput: UiElement;
  button: UiElement;
  successMessage?: string;
  refreshAfterConnect?: boolean;
}

async function runConnection({
  originInput,
  loopbackInput,
  button,
  successMessage,
  refreshAfterConnect = true
}: RunConnectionInput): Promise<boolean> {
  let origin: string;
  try {
    origin = normalizeApiOrigin(originInput.value, {
      allowInsecureLoopback: loopbackInput.checked
    });
  } catch (error) {
    showAlert(String((error as Error)?.message || error), 'error');
    return false;
  }

  const label = button.textContent || '';
  button.disabled = true;
  button.textContent = 'Testing connection...';

  try {
    const permission = requestHostPermission(origin);
    const granted = await permission;
    if (granted !== true) {
      throw new Error(`Mensetsu needs permission to contact ${origin}. Allow access and try again.`);
    }

    const result = await sendBackground<ConnectApiOriginResult>({
      type: MESSAGE.CONNECT,
      origin,
      allowInsecureLoopback: loopbackInput.checked,
      timeoutMs: 8000
    });

    if (!result?.connected) {
      showAlert(result?.connection?.message || `Could not connect to ${origin}.`, 'error');
      return false;
    }

    if (refreshAfterConnect) await refreshState();
    if (successMessage) showAlert(successMessage, 'info');
    return true;
  } catch (error) {
    showAlert(String((error as Error)?.message || error), 'error');
    return false;
  } finally {
    button.disabled = false;
    button.textContent = label;
  }
}

async function handleSetupSubmit(event: Event): Promise<void> {
  event.preventDefault();
  await runConnection({
    originInput: $('#setup-api-origin'),
    loopbackInput: $('#setup-allow-loopback-check'),
    button: $('#setup-connect-btn'),
    successMessage: 'Connected to Mensetsu. Sign in to continue.'
  });
}

async function loadResumes(): Promise<void> {
  try {
    currentResumes = await sendBackground<ResumeOption[]>({ type: MESSAGE.LIST_RESUMES });
    const select = $('#resume-select');
    select.replaceChildren();

    if (!currentResumes.length) {
      const opt = document.createElement('option') as unknown as UiElement;
      opt.value = '';
      opt.textContent = 'No completed resumes found in library';
      select.appendChild(opt);
      return;
    }

    const placeholder = document.createElement('option') as unknown as UiElement;
    placeholder.value = '';
    placeholder.textContent = '-- Select a resume --';
    select.appendChild(placeholder);

    for (const resume of currentResumes) {
      const opt = document.createElement('option') as unknown as UiElement;
      opt.value = resume.uploadId;
      const created = resume.createdAt ? new Date(resume.createdAt).toLocaleDateString() : '';
      opt.textContent = created ? `${resume.filename} (${created})` : resume.filename;
      select.appendChild(opt);
    }

    if (currentReview?.resumeUploadId) {
      select.value = currentReview.resumeUploadId;
    } else if (currentResumes.length > 0) {
      select.value = currentResumes[0].uploadId;
    }
    $('#resume-card').classList.remove('hidden');
  } catch {
    currentResumes = [];
    const select = $('#resume-select');
    select.replaceChildren();
    const option = document.createElement('option') as unknown as UiElement;
    option.value = '';
    option.textContent = 'Could not load resumes. Reopen the extension after reconnecting.';
    select.appendChild(option);
    $('#resume-card').classList.remove('hidden');
  }
}

function renderReview(review: ReviewSummary | null): void {
  currentReview = review;
  if (!review) return;

  $('#job-company').textContent = review.companyName || '-';
  $('#job-role').textContent = review.roleTitle || '-';
  $('#job-card').classList.remove('hidden');

  const fieldsContainer = $('#fields-list');
  fieldsContainer.replaceChildren();
  if (review.preparedFields && Object.keys(review.preparedFields).length > 0) {
    for (const [key, value] of Object.entries(review.preparedFields)) {
      if (!value) continue;
      const row = document.createElement('div') as unknown as UiElement;
      row.className = 'field-item';
      appendText(row, 'span', 'data-label', key);
      appendText(row, 'span', 'data-value', value);
      fieldsContainer.appendChild(row);
    }
    $('#fields-card').classList.remove('hidden');
  }

  const phaseBadge = $('#review-phase-badge');
  phaseBadge.textContent = review.phase || review.status;
  $('#actions-card').classList.remove('hidden');

  const prepareBtn = $('#prepare-btn');
  const approveBtn = $('#approve-btn');
  const fillBtn = $('#fill-btn');
  const submitBtn = $('#submit-btn');
  const submitHint = $('#submit-hint');
  const confirmedCard = $('#confirmed-card');
  const reconcileCard = $('#reconcile-card');

  prepareBtn.classList.add('hidden');
  approveBtn.classList.add('hidden');
  fillBtn.classList.add('hidden');
  submitBtn.classList.add('hidden');
  submitHint.classList.add('hidden');
  confirmedCard.classList.add('hidden');
  reconcileCard.classList.add('hidden');
  prepareBtn.textContent = 'Prepare Application';
  submitBtn.textContent = 'Submit';
  submitBtn.disabled = false;

  switch (review.phase) {
    case 'prepared':
      approveBtn.classList.remove('hidden');
      break;
    case 'approved':
      fillBtn.classList.remove('hidden');
      break;
    case 'filled':
      submitBtn.classList.remove('hidden');
      submitHint.classList.remove('hidden');
      submitBtn.disabled = review.submitUsed === true;
      break;
    case 'submitting':
      submitBtn.classList.remove('hidden');
      submitBtn.disabled = true;
      submitBtn.textContent = 'Submitting...';
      break;
    case 'submitted':
      confirmedCard.classList.remove('hidden');
      if (review.confirmationUrl) {
        $('#confirmation-url-text').textContent = review.confirmationUrl;
      }
      break;
    case 'unknown':
      reconcileCard.classList.remove('hidden');
      break;
    case 'needs_review':
      prepareBtn.classList.remove('hidden');
      prepareBtn.textContent = 'Re-prepare Application';
      if (review.submitUsed === true) reconcileCard.classList.remove('hidden');
      break;
    default:
      prepareBtn.classList.remove('hidden');
      break;
  }
}

async function handleScan(): Promise<void> {
  if (!activeTab?.id) return;
  showAlert(null);
  try {
    currentScan = await sendBackground<Scan>({
      type: MESSAGE.SCAN,
      tabId: activeTab.id,
      url: activeTab.url
    });

    const siteBadge = $('#site-badge');
    siteBadge.textContent = currentScan.site || 'unsupported';

    if (currentScan.site === SITE.GREENHOUSE) {
      siteBadge.className = 'badge badge-success';
    } else if (currentScan.site === SITE.TOPCV || currentScan.site === SITE.LINKEDIN) {
      siteBadge.className = 'badge badge-warning';
    } else {
      siteBadge.className = 'badge badge-neutral';
    }

    if (currentScan.blocked) {
      const job = currentScan.job || {};
      const canRecordBlock = Boolean(job.companyName?.trim() && job.title?.trim() && job.description?.trim());
      $('#blocked-card').classList.remove('hidden');
      $('#blocked-title').textContent = `${String(currentScan.site || 'unsupported').toUpperCase()} Blocked`;
      $('#blocked-reason-badge').textContent = currentScan.blocked.reason;
      $('#blocked-detail').textContent = canRecordBlock
        ? currentScan.blocked.detail || ''
        : `${currentScan.blocked.detail || ''} No application record can be created without verified company, role, and job-description text.`;
      $('#report-block-btn').classList.toggle('hidden', !canRecordBlock);
      $('#job-company').textContent = job.companyName || '-';
      $('#job-role').textContent = job.title || '-';
      $('#job-description-stats').textContent = `${job.description?.length || 0} chars`;
      $('#job-card').classList.remove('hidden');
      return;
    }

    $('#blocked-card').classList.add('hidden');

    if (currentScan.job) {
      $('#job-company').textContent = currentScan.job.companyName || '-';
      $('#job-role').textContent = currentScan.job.title || '-';
      $('#job-description-stats').textContent = `${currentScan.job.description?.length || 0} chars`;
      $('#job-card').classList.remove('hidden');
      $('#actions-card').classList.remove('hidden');
      $('#prepare-btn').classList.remove('hidden');
      $('#resume-card').classList.remove('hidden');
    }
  } catch (error) {
    showAlert(String((error as Error)?.message || error), 'error');
  }
}

function criteriaDescription(items: unknown): string {
  if (!Array.isArray(items) || !items.length) return 'None reported';
  return items
    .map((item) => (typeof item === 'string'
      ? item
      : `${(item as { type?: string; criterion?: string }).type || (item as { criterion?: string }).criterion}: ${(item as { value?: unknown }).value}`))
    .join(', ');
}

function saveJobPayload(job: DiscoveredJob): CreateSavedJobRequestDto {
  return {
    source_url: job.sourceUrl || job.canonicalUrl,
    role_title: job.roleTitle || null,
    company_name: job.companyName || null,
    job_description: job.jobDescription || null,
    location: job.location || null,
    application_url: job.applicationUrl || null,
    contact_email: job.contactEmail || null
  };
}

function renderDiscoveryResults(result: DiscoveryResult | null): void {
  const container = $('#discovery-results');
  container.replaceChildren();
  const jobs = result?.jobs || [];
  if (!result?.supported) {
    appendText(container, 'p', 'hint', result?.blocked?.detail || 'This page does not expose supported job listings.');
    return;
  }
  if (!jobs.length) {
    appendText(container, 'p', 'hint', 'No visible job cards were found. Open a job-search results page and try again.');
    return;
  }

  for (const job of jobs) {
    const card = document.createElement('article') as unknown as UiElement;
    card.className = 'discovery-result';
    const role = job.roleTitle || 'Job title not captured';
    const company = job.companyName || 'Company not captured';
    const description = job.jobDescription;
    const sourceUrl = safeHttpsUrl(job.sourceUrl || job.canonicalUrl);
    const match = matchDiscoveredJob(job, state?.candidateProfile || {});

    appendText(card, 'h5', '', `${role} — ${company}`);
    if (job.location) appendText(card, 'div', 'result-meta', `Location: ${job.location}`);
    appendText(card, 'div', 'criteria-list', match.eligible ? (description ? 'Profile criteria: match' : 'Preliminary match; full description not captured') : 'Profile criteria: review manually');
    appendText(card, 'div', 'criteria-list', `Matched: ${criteriaDescription(match.matchedCriteria)}`);
    if (match.missingCriteria.length) {
      appendText(card, 'div', 'criteria-list', `Missing: ${criteriaDescription(match.missingCriteria)}`);
    }
    if (match.excludedMatches.length) {
      appendText(card, 'div', 'criteria-list', `Excluded keywords: ${criteriaDescription(match.excludedMatches)}`);
    }
    if (description) appendText(card, 'div', 'result-description', description.slice(0, 900));
    else appendText(card, 'div', 'hint', 'Listing data only. Open the job detail page and scan again for full matching and application preparation.');
    if (job.contactEmail) {
      appendText(card, 'div', 'result-meta', `Email captured from job details: ${job.contactEmail}`);
    }
    if (sourceUrl) {
      const link = appendText(card, 'a', 'mono-text truncate', sourceUrl);
      link.href = sourceUrl;
      link.target = '_blank';
      link.rel = 'noopener';
    }

    const saveButton = appendText(card, 'button', 'btn btn-primary btn-block', match.excludedMatches.length ? 'Save anyway for review' : 'Save for approval');
    saveButton.type = 'button';
    saveButton.disabled = !sourceUrl;
    saveButton.addEventListener('click', async () => {
      saveButton.disabled = true;
      try {
        await sendBackground({
          type: MESSAGE.SAVE_JOB,
          job: saveJobPayload(job)
        });
        saveButton.textContent = 'Saved for approval';
        await loadSavedJobs();
      } catch (error) {
        saveButton.disabled = false;
        showAlert(String((error as Error)?.message || error), 'error');
      }
    });
    container.appendChild(card);
  }
}

async function handleDiscover(): Promise<void> {
  if (!activeTab?.id) return;
  try {
    const host = new URL(activeTab.url || '').hostname.toLowerCase();
    if (['linkedin.com', 'www.linkedin.com'].includes(host) && state?.settings?.linkedinOptIn !== true) {
      showAlert('Enable LinkedIn page detection in extension settings before scanning LinkedIn.', 'error');
      return;
    }
  } catch {
    showAlert('The active tab URL is invalid.', 'error');
    return;
  }
  showAlert(null);
  const button = $('#discover-scan-btn');
  button.disabled = true;
  try {
    currentDiscovery = await sendBackground<DiscoveryResult>({
      type: MESSAGE.DISCOVER,
      tabId: activeTab.id,
      url: activeTab.url
    });
    renderDiscoveryResults(currentDiscovery);
  } catch (error) {
    showAlert(String((error as Error)?.message || error), 'error');
  } finally {
    button.disabled = false;
  }
}

function appendSafeLink(parent: UiElement, rawUrl: unknown, label: string, className = 'mono-text truncate'): UiElement | null {
  const url = safeHttpsUrl(rawUrl);
  if (!url) return null;
  const link = appendText(parent, 'a', className, label || url);
  link.href = url;
  link.target = '_blank';
  link.rel = 'noopener';
  return link;
}

function jobComplete(job: SavedJob): boolean {
  return Boolean(
    job.role_title?.trim()
      && job.company_name?.trim()
      && job.job_description?.trim()
  );
}

function hasEmployerEmail(job: SavedJob): boolean {
  return isValidApplicationEmail(job.contact_email);
}

function renderSavedJobs(): void {
  const container = $('#saved-jobs-list');
  container.replaceChildren();
  const visibleJobs = savedJobs.filter((job) => String(job.status) !== 'dismissed');
  if (!visibleJobs.length) {
    appendText(
      container,
      'p',
      'hint',
      savedJobsCursor === null
        ? 'No saved jobs yet. Scan a job-search page in Discover mode.'
        : 'No more saved jobs await your decision.'
    );
    return;
  }

  for (const job of visibleJobs) {
    const card = document.createElement('article') as unknown as UiElement;
    card.className = 'saved-job-item';
    const role = job.role_title || 'Job title not captured';
    const company = job.company_name || 'Company not captured';
    const view = savedJobView(job);
    appendText(card, 'h5', '', `${role} — ${company}`);
    appendText(card, 'div', 'result-meta', `Status: ${view.status}`);
    appendSafeLink(card, job.source_url, 'Open source job');
    if (job.location) appendText(card, 'div', 'result-meta', `Location: ${job.location}`);
    const contactEmail = job.contact_email || '';
    const hasEmail = hasEmployerEmail(job);
    if (contactEmail) appendText(card, 'div', 'result-meta', `Email captured from job details: ${contactEmail}`);
    if (contactEmail && !hasEmail) appendText(card, 'div', 'hint', 'The captured employer email is invalid; use the website fallback.');
    if (!jobComplete(job)) {
      appendText(card, 'div', 'hint', 'Full job details are missing. Open the job detail page, scan it, and save again before preparing an application.');
    }
    if (view.hint) appendText(card, 'div', view.phase === 'failed' ? 'hint-error' : 'hint', view.hint);

    if (view.phase === 'continue') {
      const continueButton = appendText(card, 'button', 'btn btn-secondary btn-block', 'Continue application');
      continueButton.type = 'button';
      continueButton.addEventListener('click', () => handleContinueSavedJob(job));
    } else if (view.phase === 'needs_review' || view.phase === 'unlinked_awaiting' || view.phase === 'unlinked_approved') {
      const actions = document.createElement('div') as unknown as UiElement;
      actions.className = 'application-action-row';
      if (hasEmail) {
        const emailButton = appendText(actions, 'button', 'btn btn-primary', 'Approve email application');
        emailButton.type = 'button';
        emailButton.disabled = !jobComplete(job) || !currentResumes.length;
        emailButton.addEventListener('click', () => handlePrepareSavedJob(job, 'email', emailButton));
      } else {
        const webButton = appendText(actions, 'button', 'btn btn-secondary', 'Approve website fallback');
        webButton.type = 'button';
        webButton.disabled = !jobComplete(job) || !currentResumes.length;
        webButton.addEventListener('click', () => handlePrepareSavedJob(job, 'web', webButton));
      }
      card.appendChild(actions);
      if (!job.job_application_id && String(job.status) === 'awaiting_approval') {
        const dismissButton = appendText(card, 'button', 'btn btn-text', 'Dismiss');
        dismissButton.type = 'button';
        dismissButton.addEventListener('click', () => handleDismissSavedJob(job, dismissButton));
      }
      if (!currentResumes.length) appendText(card, 'div', 'hint-error', 'Upload a completed PDF resume in Mensetsu before preparing an application.');
    }
    container.appendChild(card);
  }
}

function updateLoadMoreButton(): void {
  const button = $('#saved-jobs-more-btn');
  const hidden = savedJobsCursor === null;
  button.classList.toggle('hidden', hidden);
  button.hidden = hidden;
  button.disabled = savedJobsLoading;
  button.setAttribute('aria-busy', String(savedJobsLoading));
}

async function loadSavedJobs(options: { append?: boolean } = {}): Promise<void> {
  const container = $('#saved-jobs-list');
  if (savedJobsLoading) return;
  const append = options.append === true && savedJobsCursor !== null;
  savedJobsLoading = true;
  updateLoadMoreButton();
  try {
    const page = await sendBackground<SavedJobsPage>({
      type: MESSAGE.LIST_SAVED_JOBS,
      cursor: append ? savedJobsCursor ?? undefined : undefined,
      limit: SAVED_JOBS_PAGE_LIMIT
    });
    savedJobs = append ? mergeSavedJobsPage(savedJobs, page) : page.items;
    savedJobsCursor = typeof page.next_cursor === 'string' && page.next_cursor.length > 0
      ? page.next_cursor
      : null;
    renderSavedJobs();
  } catch (error) {
    const message = `Saved jobs could not be loaded: ${String((error as Error)?.message || error)}`;
    if (append) {
      showAlert(message, 'error');
    } else {
      savedJobs = [];
      savedJobsCursor = null;
      container.replaceChildren();
      appendText(container, 'p', 'hint-error', message);
    }
  } finally {
    savedJobsLoading = false;
    updateLoadMoreButton();
  }
}

async function handleDismissSavedJob(job: SavedJob, button: UiElement): Promise<void> {
  button.disabled = true;
  try {
    await sendBackground({ type: MESSAGE.DISMISS_SAVED_JOB, savedJobId: job.id });
    await loadSavedJobs();
  } catch (error) {
    button.disabled = false;
    showAlert(String((error as Error)?.message || error), 'error');
  }
}

function updateEmailDraftLink(): void {
  if (!currentEmailApplication) return;
  const subject = $('#email-subject').value;
  const body = $('#email-body').value;
  $('#open-email-draft').href = `mailto:${encodeURIComponent(currentEmailRecipient)}?subject=${encodeURIComponent(subject)}&body=${encodeURIComponent(body)}`;
}

function emailRecipient(job: SavedJob, application: JobApplicationResponseDto): string {
  const record = application as unknown as { recipient_email?: string | null; recipientEmail?: string | null };
  return record.recipient_email || record.recipientEmail || job.contact_email || '';
}

function showEmailDraft(job: SavedJob, application: JobApplicationResponseDto): ApplicationEmailDraft {
  const draft = createApplicationEmailDraft({
    recipientEmail: emailRecipient(job, application),
    roleTitle: job.role_title || '',
    companyName: job.company_name || '',
    candidateName: state?.auth?.user?.name || '',
    candidateEmail: state?.auth?.user?.email || ''
  });
  currentEmailApplication = application;
  currentEmailRecipient = draft.recipientEmail;
  currentWebsiteApplication = null;
  $('#email-recipient').value = draft.recipientEmail;
  $('#email-subject').value = draft.subject;
  $('#email-body').value = draft.body;
  $('#email-draft-card').classList.remove('hidden');
  $('#web-application-card').classList.add('hidden');
  updateEmailDraftLink();
  return draft;
}

function showWebsiteApplication(job: SavedJob, application: JobApplicationResponseDto): void {
  const safeUrl = safeHttpsUrl(job.application_url)
    || safeHttpsUrl(job.source_url);
  if (!safeUrl) throw new Error('No safe HTTPS application URL is available for this saved job.');
  currentWebsiteApplication = application;
  currentEmailApplication = null;
  $('#open-web-application').href = safeUrl;
  $('#web-application-card').classList.remove('hidden');
  $('#email-draft-card').classList.add('hidden');
}

async function handlePrepareSavedJob(job: SavedJob, method: 'email' | 'web', button: UiElement): Promise<void> {
  const resumeUploadId = $('#resume-select').value;
  if (!resumeUploadId) {
    showAlert('Select a completed resume before approving an application.', 'error');
    return;
  }
  if (method === 'email' && !hasEmployerEmail(job)) {
    showAlert('No valid employer email was captured for this job.', 'error');
    return;
  }
  if (!jobComplete(job)) {
    showAlert('Open the job detail page, scan it, and save the complete description before applying.', 'error');
    return;
  }
  if (method === 'email') {
    try {
      createApplicationEmailDraft({
        recipientEmail: job.contact_email,
        roleTitle: job.role_title,
        companyName: job.company_name
      });
    } catch (error) {
      showAlert(String((error as Error)?.message || error), 'error');
      return;
    }
  }
  showAlert(null);
  button.disabled = true;
  try {
    if (String(job.status) !== 'approved') {
      await sendBackground({ type: MESSAGE.APPROVE_SAVED_JOB, savedJobId: job.id });
      job.status = 'approved';
    }
    const response = await sendBackground<JobApplicationResponseDto>({
      type: MESSAGE.PREPARE_SAVED_JOB,
      savedJobId: job.id,
      resumeUploadId,
      applicationMethod: method
    });
    if (!response?.id) throw new Error('The server did not return the prepared application.');
    await loadSavedJobs();
    if (method === 'email') showEmailDraft(job, response);
    else showWebsiteApplication(job, response);
  } catch (error) {
    showAlert(String((error as Error)?.message || error), 'error');
  } finally {
    button.disabled = false;
  }
}

async function handleContinueSavedJob(job: SavedJob): Promise<void> {
  try {
    const application = await sendBackground<JobApplicationResponseDto>({
      type: MESSAGE.GET_APPLICATION,
      applicationId: job.job_application_id || ''
    });
    const record = application as unknown as { application_method?: string | null };
    if (record.application_method === 'email' || (record.application_method !== 'web' && hasEmployerEmail(job))) {
      showEmailDraft(job, application);
    } else {
      showWebsiteApplication(job, application);
    }
  } catch (error) {
    showAlert(String((error as Error)?.message || error), 'error');
  }
}

async function handleEmailSent(): Promise<void> {
  const id = currentEmailApplication?.id;
  if (!id) return;
  const button = $('#email-sent-btn');
  button.disabled = true;
  try {
    await sendBackground({ type: MESSAGE.CONFIRM_EMAIL_SENT, applicationId: id });
    currentEmailApplication = null;
    currentEmailRecipient = '';
    $('#email-draft-card').classList.add('hidden');
    showAlert('Email submission recorded. Interview preparation is queued to start.', 'info');
    await loadSavedJobs();
  } catch (error) {
    showAlert(String((error as Error)?.message || error), 'error');
  } finally {
    button.disabled = false;
  }
}

async function handleWebsiteSubmitted(): Promise<void> {
  const id = currentWebsiteApplication?.id;
  if (!id) return;
  const button = $('#web-submitted-btn');
  button.disabled = true;
  try {
    await sendBackground({ type: MESSAGE.CONFIRM_WEB_SUBMITTED, applicationId: id });
    currentWebsiteApplication = null;
    $('#web-application-card').classList.add('hidden');
    showAlert('Website submission recorded. Interview preparation is queued to start.', 'info');
    await loadSavedJobs();
  } catch (error) {
    showAlert(String((error as Error)?.message || error), 'error');
  } finally {
    button.disabled = false;
  }
}

async function handlePrepare(): Promise<void> {
  if (!activeTab?.id || !currentScan) return;
  const resumeUploadId = $('#resume-select').value;
  if (!resumeUploadId) {
    showAlert('Please select a resume before preparing the application.', 'error');
    return;
  }

  showAlert(null);
  try {
    const res = await sendBackground<PrepareResult>({
      type: MESSAGE.PREPARE,
      tabId: activeTab.id,
      scan: currentScan,
      resumeUploadId
    });
    renderReview(res.review);
    showAlert('Application prepared and fields loaded from candidate profile.', 'info');
  } catch (error) {
    showAlert(String((error as Error)?.message || error), 'error');
  }
}

async function handleApprove(): Promise<void> {
  if (!activeTab?.id || !currentScan) return;
  showAlert(null);
  try {
    const res = await sendBackground<{ review: ReviewSummary | null }>({
      type: MESSAGE.APPROVE,
      tabId: activeTab.id,
      scan: currentScan
    });
    renderReview(res.review);
    showAlert('Application approved. You can now fill the form.', 'info');
  } catch (error) {
    showAlert(String((error as Error)?.message || error), 'error');
  }
}

async function handleFill(): Promise<void> {
  if (!activeTab?.id || !currentScan) return;
  showAlert(null);
  try {
    const res = await sendBackground<{ review: ReviewSummary | null; fill: FillReport }>({
      type: MESSAGE.BEGIN_FILL,
      tabId: activeTab.id,
      scan: currentScan
    });
    renderReview(res.review);
    const filledCount = res.fill?.filled?.length || 0;
    const attached = res.fill?.resumeAttached === true;
    const reason = res.fill?.resume?.reason ? ` (${res.fill.resume.reason})` : '';
    if (attached) {
      showAlert(`Form filled: ${filledCount} fields filled, resume attached. Form was NOT submitted.`, 'info');
    } else {
      showAlert(`Form filled: ${filledCount} fields filled, resume NOT attached${reason}. Form was NOT submitted.`, 'error');
    }
  } catch (error) {
    showAlert(String((error as Error)?.message || error), 'error');
  }
}

async function handleSubmit(): Promise<void> {
  if (!activeTab?.id || !currentScan) return;
  showAlert(null);
  const btn = $('#submit-btn');
  btn.disabled = true;
  btn.textContent = 'Watching for Confirmation...';
  showAlert('Handoff approved: Please submit on the job page. Watching for confirmation...', 'info');

  try {
    const res = await sendBackground<BeginSubmitResult & SubmitCompletionResult>({
      type: MESSAGE.BEGIN_SUBMIT,
      tabId: activeTab.id,
      scan: currentScan,
      valuesFingerprint: currentReview?.filledFingerprint ?? undefined
    });
    renderReview(res.review);
    const notice = submitOutcomeNotice(res.review);
    if (notice) showAlert(notice.message, notice.level);
  } catch (error) {
    showAlert(String((error as Error)?.message || error), 'error');
    await refreshState();
  }
}

async function handleReconcile(submitted: boolean): Promise<void> {
  if (!activeTab?.id) return;
  showAlert(null);
  try {
    const res = await sendBackground<{ review: ReviewSummary | null }>({
      type: MESSAGE.RECONCILE,
      tabId: activeTab.id,
      submitted
    });
    renderReview(res.review);
    showAlert(`Reconciliation recorded: ${submitted ? 'Submitted' : 'Not submitted'}.`, 'info');
  } catch (error) {
    showAlert(String((error as Error)?.message || error), 'error');
  }
}

async function handleBlockReport(): Promise<void> {
  if (!activeTab?.id || !currentScan?.blocked) return;
  const job = currentScan.job || {};
  if (!job.companyName?.trim() || !job.title?.trim() || !job.description?.trim()) {
    showAlert('Cannot create an application record without verified company, role, and job-description text.', 'error');
    return;
  }
  showAlert(null);
  try {
    const resumeUploadId = $('#resume-select')?.value || currentResumes[0]?.uploadId;
    if (!currentReview) {
      await sendBackground<PrepareResult>({
        type: MESSAGE.PREPARE,
        tabId: activeTab.id,
        scan: currentScan,
        resumeUploadId
      });
    }
    const res = await sendBackground<BlockResult>({
      type: MESSAGE.BLOCK,
      tabId: activeTab.id,
      reasonCode: currentScan.blocked.reason
    });
    renderReview(res.review ?? null);
    showAlert(`Site block recorded in Mensetsu (${currentScan.blocked.reason}).`, 'info');
  } catch (error) {
    showAlert(String((error as Error)?.message || error), 'error');
  }
}

function bindEvents(): void {
  $('#setup-form').addEventListener('submit', handleSetupSubmit);

  $('#toggle-settings-btn').addEventListener('click', () => {
    $('#settings-panel').classList.toggle('hidden');
  });

  $('#cancel-settings-btn').addEventListener('click', () => {
    $('#settings-panel').classList.add('hidden');
  });

  $('#save-settings-btn').addEventListener('click', async () => {
    showAlert(null);
    const linkedinOptIn = $('#linkedin-optin-check').checked;
    const connected = await runConnection({
      originInput: $('#api-origin-input'),
      loopbackInput: $('#allow-loopback-check'),
      button: $('#save-settings-btn'),
      refreshAfterConnect: false
    });
    if (!connected) return;

    try {
      await sendBackground({
        type: MESSAGE.SET_LINKEDIN_OPT_IN,
        optIn: linkedinOptIn
      });
      $('#settings-panel').classList.add('hidden');
      showAlert('Settings saved.', 'info');
      await refreshState();
    } catch (error) {
      showAlert(String((error as Error)?.message || error), 'error');
    }
  });

  $('#sign-in-btn').addEventListener('click', async () => {
    showAlert(null);
    const email = $('#login-email').value.trim();
    const password = $('#login-password').value;
    const code = $('#login-2fa-code').value.trim();

    try {
      if ($('#2fa-fields').classList.contains('hidden')) {
        const res = await sendBackground<LoginResult>({ type: MESSAGE.LOGIN, email, password });
        if (res.requires2fa) {
          $('#2fa-fields').classList.remove('hidden');
          $('#login-fields').classList.add('hidden');
          showAlert('Please enter your 2FA TOTP code.', 'info');
          return;
        }
      } else {
        await sendBackground({ type: MESSAGE.LOGIN_2FA, code });
      }
      await refreshState();
    } catch (error) {
      showAlert(String((error as Error)?.message || error), 'error');
    }
  });

  $('#sign-out-btn').addEventListener('click', async () => {
    await sendBackground({ type: MESSAGE.LOGOUT });
    await refreshState();
  });

  $('#discover-mode-btn').addEventListener('click', () => setMode('discover'));
  $('#apply-mode-btn').addEventListener('click', () => setMode('apply'));
  $('#discover-scan-btn').addEventListener('click', handleDiscover);
  $('#refresh-saved-jobs-btn').addEventListener('click', () => {
    void loadSavedJobs();
  });
  $('#saved-jobs-more-btn').addEventListener('click', () => {
    void loadSavedJobs({ append: true });
  });
  $('#email-subject').addEventListener('input', updateEmailDraftLink);
  $('#email-body').addEventListener('input', updateEmailDraftLink);
  $('#email-sent-btn').addEventListener('click', handleEmailSent);
  $('#web-submitted-btn').addEventListener('click', handleWebsiteSubmitted);
  $('#scan-tab-btn').addEventListener('click', handleScan);
  $('#prepare-btn').addEventListener('click', handlePrepare);
  $('#approve-btn').addEventListener('click', handleApprove);
  $('#fill-btn').addEventListener('click', handleFill);
  $('#submit-btn').addEventListener('click', handleSubmit);
  $('#reconcile-yes-btn').addEventListener('click', () => handleReconcile(true));
  $('#reconcile-no-btn').addEventListener('click', () => handleReconcile(false));
  $('#report-block-btn').addEventListener('click', handleBlockReport);
}

document.addEventListener('DOMContentLoaded', () => {
  void (async () => {
    bindEvents();
    await loadActiveTab();
    await refreshState();
  })();
});
