import { MAX_JOB_DESCRIPTION_CHARS, REASON, SITE } from '../../shared/constants.js';
import { clampText, normalizeText } from '../../shared/util.js';
import {
  attachResume,
  collectFields,
  detectCaptchaChallenge,
  detectLoginWall,
  fillPreparedFields,
  jobDescriptionText,
  resumeRequired,
  submitDescriptor,
  textOf,
  valuesFromForm
} from './dom.js';
import type { DomDocument } from './dom-types.js';
import type {
  AttachmentResult,
  ConfirmationResult,
  FillReport,
  PreparedFields,
  ResumeAttachment,
  Scan,
  SiteAdapter,
  SubmitResult
} from '../../shared/types.js';

const GREENHOUSE_HOSTS = new Set(['job-boards.greenhouse.io', 'boards.greenhouse.io']);

export interface GreenhouseJobIdentity {
  board: string;
  jobId: string;
}

export function parseGreenhouseJob(rawUrl: string): GreenhouseJobIdentity | null {
  try {
    const url = new URL(rawUrl);
    if (!GREENHOUSE_HOSTS.has(url.hostname)) return null;
    const parts = url.pathname.split('/').filter(Boolean);
    if (parts.length < 2) return null;
    const board = parts[0];
    const id = parts[1] === 'jobs' ? parts[2] : parts[1];
    if (!id || !/^\d+$/.test(id)) return null;
    return { board, jobId: id };
  } catch {
    return null;
  }
}

function parseCompanyFromTitle(title: unknown): string {
  const match = /^Job Application for .+ at (.+)$/i.exec(normalizeText(title));
  return match ? match[1].trim() : '';
}

export const greenhouseAdapter: SiteAdapter = {
  id: SITE.GREENHOUSE,
  label: 'Greenhouse',
  verified: true,
  evidence: 'evidence/greenhouse-2026-11-10/',
  capabilities: { scan: true, fill: true, submit: false, manualSubmitRequired: true, verifiedConfirmation: true },

  matches(rawUrl: string): boolean {
    return Boolean(parseGreenhouseJob(rawUrl));
  },

  canonicalize(rawUrl: string): string {
    const parsed = parseGreenhouseJob(rawUrl);
    if (!parsed) throw new Error('Not a Greenhouse job post URL.');
    return `https://boards.greenhouse.io/${parsed.board}/${parsed.jobId}`;
  },

  scan(doc: DomDocument, rawUrl: string): Scan {
    const canonicalUrl = this.canonicalize(rawUrl);
    const form = doc.querySelector?.('form#application-form') ?? null;
    if (!form) {
      return {
        supported: true,
        site: SITE.GREENHOUSE,
        canonicalUrl,
        sourceUrl: canonicalUrl,
        blocked: { reason: REASON.FORM_NOT_FOUND, detail: 'Greenhouse application form was not present on the page.' },
        capabilities: this.capabilities,
        evidence: this.evidence
      };
    }
    if (detectLoginWall(doc, rawUrl)) {
      return {
        supported: true,
        site: SITE.GREENHOUSE,
        canonicalUrl,
        sourceUrl: canonicalUrl,
        blocked: { reason: REASON.LOGIN_REQUIRED, detail: 'A login wall was detected on the job page.' },
        capabilities: this.capabilities,
        evidence: this.evidence
      };
    }
    if (detectCaptchaChallenge(doc)) {
      return {
        supported: true,
        site: SITE.GREENHOUSE,
        canonicalUrl,
        sourceUrl: canonicalUrl,
        blocked: { reason: REASON.CAPTCHA_REQUIRED, detail: 'A CAPTCHA challenge was detected; the extension does not bypass it.' },
        capabilities: this.capabilities,
        evidence: this.evidence
      };
    }

    const submit = submitDescriptor(form);
    const fields = collectFields(form);
    const title = textOf(doc, '.job__title h1') || textOf(doc, 'h1.section-header') || '';
    const company = parseCompanyFromTitle(doc.title) || '';
    return {
      supported: true,
      site: SITE.GREENHOUSE,
      canonicalUrl,
      sourceUrl: canonicalUrl,
      job: {
        title,
        companyName: company,
        location: textOf(doc, '.job__location'),
        description: clampText(jobDescriptionText(doc, '.job__description'), MAX_JOB_DESCRIPTION_CHARS),
        sourceTitle: normalizeText(doc.title)
      },
      form: {
        selector: 'form#application-form',
        action: String(form.getAttribute?.('action') || ''),
        method: String(form.getAttribute?.('method') || '').toLowerCase(),
        fields,
        submit
      },
      confirmation: {
        pathSuffix: '/confirmation',
        expectedTitlePrefix: 'Thank you for applying'
      },
      capabilities: this.capabilities,
      evidence: this.evidence,
      blocked: submit.found ? null : { reason: REASON.SUBMIT_UNAVAILABLE, detail: 'No enabled submit control was found.' }
    };
  },

  fill(doc: DomDocument, preparedFields: PreparedFields, resume: ResumeAttachment | null): FillReport {
    const form = doc.querySelector?.('form#application-form') ?? null;
    if (!form) return { filled: [], skipped: [], fields: [], resumeAttached: false, resumeRequired: false };
    const report = fillPreparedFields(form, preparedFields);
    const selectedResume = Boolean(resume?.bytes?.length);
    const attachment: AttachmentResult = selectedResume
      ? attachResume(form, resume?.bytes, resume?.filename, resume?.mime)
      : { attached: false, reason: 'no_resume_selected' };
    return {
      ...report,
      fields: valuesFromForm(form),
      resumeAttached: attachment.attached,
      resumeRequired: resumeRequired(form),
      resume: attachment
    };
  },

  submit(): SubmitResult {
    return {
      clicked: false,
      manualSubmitRequired: true,
      reason: 'manual_submit_required',
      detail: 'Manual submission on site required: Greenhouse SSR fallback has method=get and unverified live submit handler; programmatic submit is blocked to prevent PII leakage.'
    };
  },

  isConfirmationUrl(rawUrl: string, expectedJobUrl?: string): boolean {
    try {
      const expectedJob = parseGreenhouseJob(String(expectedJobUrl));
      const url = new URL(rawUrl);
      if (!expectedJob || !GREENHOUSE_HOSTS.has(url.hostname)) return false;
      const parts = url.pathname.split('/').filter(Boolean);
      if (parts.at(-1) !== 'confirmation') return false;
      const jobId = parts.at(-2);
      const board = parts.at(-3) === 'jobs' ? parts.at(-4) : parts.at(-3);
      return board === expectedJob.board && jobId === expectedJob.jobId;
    } catch {
      return false;
    }
  },

  confirmation(doc: DomDocument, rawUrl: string, expectedJobUrl?: string): ConfirmationResult {
    const heading = textOf(doc, '.confirmation__content h1');
    const title = normalizeText(doc.title || '');
    const confirmed = Boolean(this.isConfirmationUrl?.(rawUrl, expectedJobUrl))
      && (heading.toLowerCase().startsWith('thank you for applying') || title.toLowerCase().startsWith('thank you for applying'));
    return {
      confirmed,
      url: rawUrl,
      text: heading || title,
      selector: '.confirmation__content h1',
      evidence: this.evidence
    };
  }
};
