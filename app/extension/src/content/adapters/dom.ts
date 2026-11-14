import { MAX_FORM_FIELDS } from '../../shared/constants.js';
import { clampText, normalizeText, nowIso } from '../../shared/util.js';
import type {
  AttachmentResult,
  ConfirmationResult,
  DiscoveredJob,
  DiscoveredJobInput,
  DiscoveryEvidence,
  FieldValue,
  FillReport,
  JobCompleteness,
  PreparedFields,
  ScanFormField,
  ScanSubmitDescriptor
} from '../../shared/types.js';
import type { DomDocument, DomElement, DomRoot } from './dom-types.js';

const SKIP_INPUT_TYPES = new Set([
  'hidden',
  'password',
  'submit',
  'button',
  'reset',
  'image',
  'search'
]);

export function cleanUrl(raw: unknown): URL | null {
  try {
    return new URL(String(raw));
  } catch {
    return null;
  }
}

export function textOf(root: DomRoot | null | undefined, selector: string): string {
  if (!root || typeof root.querySelector !== 'function') return '';
  const el = root.querySelector(selector);
  return el ? normalizeText(el.textContent || el.value || '') : '';
}

export function attrOf(root: DomRoot | null | undefined, selector: string, attribute: string): string {
  if (!root || typeof root.querySelector !== 'function') return '';
  const el = root.querySelector(selector);
  return el ? String(el.getAttribute?.(attribute) ?? '') : '';
}

export function isVisible(el: DomElement | null | undefined): boolean {
  if (!el) return false;
  if (el.hidden === true) return false;
  if (el.getAttribute?.('hidden') !== null && el.getAttribute?.('hidden') !== undefined) return false;
  const style = el.ownerDocument?.defaultView?.getComputedStyle?.(el);
  if (style && (style.display === 'none' || style.visibility === 'hidden' || style.opacity === '0')) return false;
  return true;
}

export function hasVisiblePasswordField(doc: DomDocument | null | undefined): boolean {
  const inputs = doc?.querySelectorAll?.('input[type="password"]') || [];
  return Array.from(inputs).some((input) => isVisible(input));
}

export function detectLoginWall(doc: DomDocument | null | undefined, url: string): boolean {
  const parsed = cleanUrl(url);
  const path = (parsed?.pathname || '').toLowerCase();
  if (/(^|\/)(login|signin|sign-in|authwall|uas\/login)(\/|$)/.test(path)) return true;
  if (hasVisiblePasswordField(doc)) return true;
  const bodyText = normalizeText(doc?.body?.textContent || '').toLowerCase();
  return /sign in to apply|log in to apply|đăng nhập để ứng tuyển|đăng nhập hoặc đăng ký để ứng tuyển/.test(bodyText);
}

export function detectVisibleLoginWall(doc: DomDocument | null | undefined, url: string): boolean {
  const parsed = cleanUrl(url);
  const path = (parsed?.pathname || '').toLowerCase();
  if (/(^|\/)(login|signin|sign-in|authwall|uas\/login)(\/|$)/.test(path)) return true;
  return hasVisiblePasswordField(doc);
}

export function detectCaptchaChallenge(doc: DomDocument | null | undefined): boolean {
  if (!doc) return false;
  const known = [
    'iframe[src*="recaptcha"]',
    'iframe[src*="hcaptcha"]',
    'iframe[src*="challenges.cloudflare.com"]',
    '[data-sitekey]'
  ];
  for (const selector of known) {
    const nodes = doc.querySelectorAll?.(selector) || [];
    if (Array.from(nodes).some((node) => isVisible(node))) return true;
  }
  const bodyText = normalizeText(doc.body?.textContent || '').toLowerCase();
  return /verify you are human|unusual traffic|complete the captcha|xác minh bạn là người/.test(bodyText);
}

export function labelFor(el: DomElement | null | undefined): string {
  if (!el) return '';
  const owner = el.ownerDocument;
  const id = el.getAttribute?.('id');
  if (id && owner?.querySelectorAll) {
    const label = Array.from(owner.querySelectorAll('label[for]'))
      .find((candidate) => candidate.getAttribute?.('for') === id);
    if (label) return normalizeText(label.textContent || '');
  }
  const aria = el.getAttribute?.('aria-label');
  if (aria) return normalizeText(aria);
  const labelledBy = el.getAttribute?.('aria-labelledby')?.trim();
  if (labelledBy && owner?.querySelectorAll) {
    const nodes = Array.from(owner.querySelectorAll('[id]'));
    const labelText = labelledBy.split(/\s+/)
      .map((labelId) => nodes.find((node) => node.getAttribute?.('id') === labelId)?.textContent || '')
      .filter(Boolean)
      .join(' ');
    if (labelText) return normalizeText(labelText);
  }
  return '';
}

export function collectFields(form: DomRoot | null | undefined): ScanFormField[] {
  const nodes = form?.querySelectorAll?.('input, select, textarea') || [];
  const fields: ScanFormField[] = [];
  for (const el of Array.from(nodes)) {
    if (fields.length >= MAX_FORM_FIELDS) break;
    const tag = String(el.tagName || el.tag || '').toLowerCase();
    const type = String(el.getAttribute?.('type') || (tag === 'textarea' ? 'textarea' : tag === 'select' ? 'select' : 'text')).toLowerCase();
    if (SKIP_INPUT_TYPES.has(type)) continue;
    if (el.disabled === true || el.getAttribute?.('disabled') !== null && el.getAttribute?.('disabled') !== undefined) continue;
    if (el.getAttribute?.('aria-hidden') === 'true') continue;
    fields.push({
      id: String(el.getAttribute?.('id') || ''),
      name: String(el.getAttribute?.('name') || ''),
      type,
      required: el.required === true || el.getAttribute?.('aria-required') === 'true' || el.getAttribute?.('required') !== null && el.getAttribute?.('required') !== undefined,
      label: labelFor(el),
      autocomplete: String(el.getAttribute?.('autocomplete') || ''),
      tag,
      disabled: false
    });
  }
  return fields;
}

export function submitControl(form: DomRoot | null | undefined): DomElement | null {
  if (!form || typeof form.querySelector !== 'function') return null;
  return form.querySelector('button[type="submit"], input[type="submit"]');
}

export function submitDescriptor(form: DomRoot | null | undefined): ScanSubmitDescriptor {
  const button = submitControl(form);
  if (!button) return { selector: '', text: '', found: false, disabled: true };
  return {
    selector: button.tagName?.toLowerCase() === 'input' ? 'input[type="submit"]' : 'button[type="submit"]',
    text: normalizeText(button.textContent || button.value || ''),
    found: true,
    disabled: button.disabled === true || button.getAttribute?.('aria-disabled') === 'true'
  };
}

function findField(form: DomRoot | null | undefined, key: unknown): DomElement | null {
  if (key === null || key === undefined) return null;
  const fieldKey = String(key);
  const fields = form?.querySelectorAll?.('input, select, textarea') || [];
  return Array.from(fields).find((field) =>
    field.getAttribute?.('id') === fieldKey || field.getAttribute?.('name') === fieldKey
  ) || null;
}

const AUTOCOMPLETE_FALLBACKS: Readonly<Record<string, string>> = Object.freeze({
  first_name: 'input[autocomplete="given-name"]',
  last_name: 'input[autocomplete="family-name"]',
  email: 'input[autocomplete="email"]',
  phone: 'input[autocomplete="tel"]',
  location: 'input[autocomplete="address-level2"]'
});

function nativeSet(el: DomElement, value: string): void {
  const prototype = el.tagName?.toLowerCase() === 'textarea'
    ? globalThis.HTMLTextAreaElement?.prototype
    : globalThis.HTMLInputElement?.prototype;
  const descriptor = prototype ? Object.getOwnPropertyDescriptor(prototype, 'value') : null;
  if (descriptor?.set) descriptor.set.call(el, value);
  else el.value = value;
}

function dispatch(el: DomElement, type: string): void {
  if (typeof el.dispatchEvent !== 'function') return;
  const event = typeof Event === 'function'
    ? new Event(type, { bubbles: true, composed: true })
    : { type, bubbles: true };
  el.dispatchEvent(event);
}

const SAFE_INPUT_TYPES = new Set(['text', 'email', 'tel', 'url', 'textarea']);

const SENSITIVE_PATTERN = /password|gender|sex\b|race|ethnic|veteran|military|disability|handicap|consent|agree|terms|privacy|acknowledg|ssn|social.*sec|salary|compensation|citizen|visa/i;

export function isSensitiveField(el: DomElement | null | undefined, label = ''): boolean {
  if (!el) return false;
  const name = String(el.getAttribute?.('name') || '');
  const id = String(el.getAttribute?.('id') || '');
  const aria = String(el.getAttribute?.('aria-label') || '');
  const type = String(el.getAttribute?.('type') || '').toLowerCase();
  if (type === 'password' || type === 'checkbox' || type === 'radio') return true;
  const text = `${name} ${id} ${aria} ${label}`;
  return SENSITIVE_PATTERN.test(text);
}

export function fillPreparedFields(form: DomRoot | null | undefined, preparedFields: PreparedFields | null | undefined): FillReport {
  const filled: FillReport['filled'] = [];
  const skipped: FillReport['skipped'] = [];

  for (const [rawKey, rawValue] of Object.entries(preparedFields || {})) {
    if (rawValue === null || rawValue === undefined || rawValue === '') {
      skipped.push({ key: rawKey, reason: 'empty' });
      continue;
    }
    const value = String(rawValue);
    let el = findField(form, rawKey);
    if (!el && AUTOCOMPLETE_FALLBACKS[rawKey]) {
      el = form?.querySelector?.(AUTOCOMPLETE_FALLBACKS[rawKey]) || null;
    }
    if (!el) {
      skipped.push({ key: rawKey, reason: 'not_found' });
      continue;
    }
    const tag = String(el.tagName || '').toLowerCase();
    const type = String(el.getAttribute?.('type') || (tag === 'textarea' ? 'textarea' : 'text')).toLowerCase();

    if (type === 'file') {
      skipped.push({ key: rawKey, reason: 'file_handled_separately' });
      continue;
    }

    if (!SAFE_INPUT_TYPES.has(type)) {
      skipped.push({ key: rawKey, reason: 'unsafe_type', type });
      continue;
    }

    const label = labelFor(el);
    if (isSensitiveField(el, label)) {
      skipped.push({ key: rawKey, reason: 'sensitive_field_skipped' });
      continue;
    }

    nativeSet(el, value);
    dispatch(el, 'input');
    dispatch(el, 'change');
    filled.push({ key: rawKey, id: String(el.getAttribute?.('id') || ''), name: String(el.getAttribute?.('name') || '') });
  }

  return { filled, skipped, fields: valuesFromForm(form) };
}

export function attachResume(
  form: DomRoot | null | undefined,
  bytes: Uint8Array | number[] | null | undefined,
  filename: string | undefined,
  mime = 'application/pdf'
): AttachmentResult {
  if (!form || !bytes || bytes.length === 0) return { attached: false, reason: 'resume_missing' };
  const input = form.querySelector?.('input[type="file"][id*="resume"], input[type="file"][name*="resume"], input[type="file"]');
  if (!input) return { attached: false, reason: 'file_input_not_found' };
  if (typeof DataTransfer !== 'function' || typeof File !== 'function') {
    return { attached: false, reason: 'file_api_unavailable' };
  }
  try {
    const file = new File([bytes as unknown as BlobPart], filename || 'resume.pdf', { type: mime || 'application/pdf' });
    const transfer = new DataTransfer();
    transfer.items.add(file);
    (input as unknown as { files: FileList }).files = transfer.files;
    dispatch(input, 'input');
    dispatch(input, 'change');
    return { attached: true, filename: file.name, size: file.size };
  } catch (error) {
    return { attached: false, reason: String((error as Error)?.message || error) };
  }
}

export function valuesFromForm(form: DomRoot | null | undefined): FieldValue[] {
  const fields = collectFields(form);
  return fields.map((descriptor) => {
    const el = findField(form, descriptor.id || descriptor.name);
    const value = el?.value != null ? String(el.value) : '';
    const file = descriptor.type === 'file'
      ? { name: String(el?.files?.[0]?.name || ''), size: Number(el?.files?.[0]?.size || 0) }
      : null;
    return { id: String(descriptor.id || ''), name: String(descriptor.name || ''), type: String(descriptor.type || ''), value, checked: el?.checked === true, file };
  });
}

export function resumeRequired(form: DomRoot | null | undefined): boolean {
  const inputs = form?.querySelectorAll?.('input[type="file"]') || [];
  return Array.from(inputs).some((input) => input.required === true || input.getAttribute?.('aria-required') === 'true');
}

export function resumeAttached(form: DomRoot | null | undefined): boolean {
  const inputs = form?.querySelectorAll?.('input[type="file"]') || [];
  return Array.from(inputs).some((input) => Number(input.files?.length || 0) > 0);
}

export function jobDescriptionText(doc: DomRoot | null | undefined, selector: string): string {
  return normalizeText(doc?.querySelector?.(selector)?.textContent || '');
}

export async function waitForConfirmation(
  check: () => Promise<ConfirmationResult> | ConfirmationResult,
  { timeoutMs = 20_000, intervalMs = 400 }: { timeoutMs?: number; intervalMs?: number } = {}
): Promise<ConfirmationResult> {
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    const result = await check();
    if (result?.confirmed) return result;
    await new Promise((resolve) => setTimeout(resolve, intervalMs));
  }
  return { confirmed: false, reason: 'confirmation_timeout' };
}

const MAX_DISCOVERY_JOBS = 40;
const MAX_DISCOVERY_DESCRIPTION_CHARS = 30_000;
const DISCOVERY_HIDDEN_TOKENS = new Set(['hidden', 'd-none', 'ng-hide', 'is-hidden', 'sr-only', 'visually-hidden']);
const CONTACT_EMAIL_PATTERN = /[a-z0-9._%+-]+@[a-z0-9-]+(?:\.[a-z0-9-]+)*\.[a-z]{2,}/gi;
const NON_CONTACT_EMAIL_PATTERN = /\.(?:png|jpe?g|gif|svg|webp|css|js|json|html?)$/i;
const DISCOVERY_COMPLETENESS_FIELDS = Object.freeze([
  'roleTitle',
  'companyName',
  'jobDescription',
  'location',
  'applicationUrl',
  'contactEmail'
] as const);

type CompletenessField = (typeof DISCOVERY_COMPLETENESS_FIELDS)[number];

export const DISCOVERY_LIMITS = Object.freeze({
  maxJobs: MAX_DISCOVERY_JOBS,
  maxDescriptionChars: MAX_DISCOVERY_DESCRIPTION_CHARS
});

export function isRendered(el: DomElement | string | null | undefined): boolean {
  if (!el || typeof el === 'string') return false;
  if (el.nodeType === 3) return true;
  if (el.nodeType && el.nodeType !== 1) return true;
  for (let node: DomElement | null | undefined = el; node && node.nodeType !== 9; node = node.parentNode) {
    const hiddenAttr = node.getAttribute?.('hidden');
    if (hiddenAttr !== null && hiddenAttr !== undefined) return false;
    const tokens = String(node.getAttribute?.('class') || '').split(/\s+/);
    if (tokens.some((token) => DISCOVERY_HIDDEN_TOKENS.has(token))) return false;
    const style = node.ownerDocument?.defaultView?.getComputedStyle?.(node);
    if (style && (style.display === 'none' || style.visibility === 'hidden' || String(style.opacity) === '0')) return false;
  }
  if (typeof el.getBoundingClientRect === 'function') {
    const rect = el.getBoundingClientRect();
    const hasElementChildren = Boolean(el.children?.length);
    if (!hasElementChildren && Number(rect?.width) === 0 && Number(rect?.height) === 0) return false;
  }
  return true;
}

export function visibleElements(root: DomRoot | null | undefined, selector: string): DomElement[] {
  const nodes = root?.querySelectorAll?.(selector);
  if (!nodes) return [];
  return Array.from(nodes).filter((node) => isRendered(node));
}

export function visibleText(el: DomElement | string | null | undefined): string {
  const parts: string[] = [];
  const walk = (node: DomElement | string | null | undefined): void => {
    if (node === null || node === undefined) return;
    if (typeof node === 'string') {
      parts.push(node);
      return;
    }
    if (node.nodeType === 3) {
      parts.push(node.textContent || '');
      return;
    }
    if (node.nodeType === 8) return;
    if (typeof node.getAttribute === 'function' && !isRendered(node)) return;
    const childNodes = node.childNodes;
    if (childNodes && childNodes.length) {
      for (const child of Array.from(childNodes)) walk(child);
      return;
    }
    const children = node.children;
    if (children && children.length) {
      for (const child of Array.from(children)) walk(child);
      return;
    }
    parts.push(typeof node.textContent === 'string' ? node.textContent : '');
  };
  walk(el);
  return normalizeText(parts.join(' '));
}

export interface VisibleMatch {
  value: string;
  selector: string;
}

export function visibleTextOf(root: DomRoot | null | undefined, selectors: readonly string[] | null | undefined): VisibleMatch {
  for (const selector of selectors || []) {
    for (const el of visibleElements(root, selector)) {
      const value = visibleText(el);
      if (value) return { value, selector };
    }
  }
  return { value: '', selector: '' };
}

export function visibleAttributeOf(root: DomRoot | null | undefined, selectors: readonly string[] | null | undefined, attribute: string): VisibleMatch {
  for (const selector of selectors || []) {
    for (const el of visibleElements(root, selector)) {
      const value = normalizeText(String(el.getAttribute?.(attribute) ?? ''));
      if (value) return { value, selector };
    }
  }
  return { value: '', selector: '' };
}

export function absoluteHttpsUrl(raw: unknown, base?: unknown): string {
  if (raw === null || raw === undefined) return '';
  const value = String(raw).replace(/[\u0000-\u001f\u007f]/g, '').trim();
  if (!value) return '';
  try {
    const url = new URL(value, base ? String(base) : undefined);
    if (url.protocol !== 'https:' || url.username || url.password) return '';
    return url.toString();
  } catch {
    return '';
  }
}

export function extractEmailsFromText(text: unknown): string[] {
  const matches = String(text || '').match(CONTACT_EMAIL_PATTERN) || [];
  const seen = new Set<string>();
  const emails: string[] = [];
  for (const match of matches) {
    const email = match.toLowerCase().replace(/[.,;:]+$/, '');
    if (seen.has(email)) continue;
    seen.add(email);
    emails.push(email);
  }
  return emails;
}

export function pickContactEmail(text: unknown): string {
  const emails = extractEmailsFromText(text);
  for (const email of emails) {
    if (email.length > 254) continue;
    if (NON_CONTACT_EMAIL_PATTERN.test(email)) continue;
    return email;
  }
  return '';
}

export function dedupeJobsByCanonicalUrl(jobs: ReadonlyArray<DiscoveredJob | null | undefined> | null | undefined): DiscoveredJob[] {
  const seen = new Set<string>();
  const result: DiscoveredJob[] = [];
  for (const job of jobs || []) {
    if (!job) continue;
    const key = String(job.canonicalUrl || job.sourceUrl || '');
    if (!key || seen.has(key)) continue;
    seen.add(key);
    result.push(job);
  }
  return result;
}

function discoveredCompleteness(job: Partial<Record<CompletenessField, string | boolean>>): JobCompleteness {
  const fields = {} as Record<CompletenessField, boolean>;
  for (const field of DISCOVERY_COMPLETENESS_FIELDS) fields[field] = Boolean(job[field]);
  return {
    ...fields,
    present: DISCOVERY_COMPLETENESS_FIELDS.filter((field) => fields[field]).length,
    total: DISCOVERY_COMPLETENESS_FIELDS.length,
    complete: fields.roleTitle && fields.companyName && fields.jobDescription
  };
}

export function createDiscoveredJob(input: DiscoveredJobInput = {}): DiscoveredJob {
  const sourceUrl = absoluteHttpsUrl(input.sourceUrl, input.baseUrl);
  const evidence: DiscoveryEvidence = input.evidence && typeof input.evidence === 'object'
    ? { ...(input.evidence as DiscoveryEvidence) }
    : { source: 'visible_dom', verified: false, scope: 'job_card', selectors: {} };
  const base = {
    site: String(input.site || ''),
    sourceUrl,
    canonicalUrl: absoluteHttpsUrl(input.canonicalUrl, sourceUrl || input.baseUrl) || sourceUrl,
    roleTitle: normalizeText(input.roleTitle),
    companyName: normalizeText(input.companyName),
    jobDescription: clampText(normalizeText(input.jobDescription), MAX_DISCOVERY_DESCRIPTION_CHARS),
    location: normalizeText(input.location),
    applicationUrl: absoluteHttpsUrl(input.applicationUrl, input.baseUrl),
    contactEmail: pickContactEmail(input.contactEmail),
    observedAt: normalizeText(input.observedAt) || nowIso(),
    evidence
  };
  return {
    ...base,
    completeness: discoveredCompleteness({
      roleTitle: base.roleTitle,
      companyName: base.companyName,
      jobDescription: base.jobDescription,
      location: base.location,
      applicationUrl: base.applicationUrl,
      contactEmail: base.contactEmail
    })
  };
}
