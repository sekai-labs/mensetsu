import { REASON, SITE } from '../../shared/constants.js';
import { nowIso } from '../../shared/util.js';
import {
  absoluteHttpsUrl,
  createDiscoveredJob,
  dedupeJobsByCanonicalUrl,
  DISCOVERY_LIMITS,
  detectCaptchaChallenge,
  detectLoginWall,
  detectVisibleLoginWall,
  pickContactEmail,
  visibleAttributeOf,
  visibleElements,
  visibleTextOf
} from './dom.js';
import type { DomDocument, DomElement } from './dom-types.js';
import type {
  ConfirmationResult,
  DiscoveredJob,
  DiscoveryEvidence,
  DiscoveryResult,
  FillReport,
  PreparedFields,
  ResumeAttachment,
  Scan,
  DiscoveryAdapter,
  SubmitResult
} from '../../shared/types.js';

const LINKEDIN_HOSTS = new Set(['linkedin.com', 'www.linkedin.com']);
const LINKEDIN_BASE_URL = 'https://www.linkedin.com';
const LINKEDIN_CARD_SELECTORS: readonly string[] = Object.freeze([
  'li.jobs-search-results__list-item',
  'li.jobs-search-results-list__list-item',
  'div.job-card-container'
]);
const LINKEDIN_CARD_LINK_SELECTORS: readonly string[] = Object.freeze([
  'a.job-card-container__link[href]',
  'a.job-card-list__title--link[href]',
  'a[href*="/jobs/view/"]'
]);
const LINKEDIN_CARD_TITLE_SELECTORS: readonly string[] = Object.freeze([
  'a.job-card-container__link span[aria-hidden="true"]',
  'a.job-card-list__title--link span[aria-hidden="true"]',
  'a.job-card-list__title--link',
  'a.job-card-container__link'
]);
const LINKEDIN_CARD_COMPANY_SELECTORS: readonly string[] = Object.freeze([
  '.job-card-container__primary-description',
  '.job-card-container__company-name',
  '.artdeco-entity-lockup__subtitle'
]);
const LINKEDIN_CARD_LOCATION_SELECTORS: readonly string[] = Object.freeze([
  '.job-card-container__metadata-item',
  '.job-card-container__metadata-wrapper li',
  '.artdeco-entity-lockup__caption li'
]);
const LINKEDIN_CARD_APPLY_SELECTORS: readonly string[] = Object.freeze([
  'a.job-card-container__apply[href]',
  'a[data-live-test-job-apply-button][href]'
]);
const LINKEDIN_DETAIL_TITLE_SELECTORS: readonly string[] = Object.freeze([
  '.job-details-jobs-unified-top-card__job-title',
  '.jobs-unified-top-card__job-title',
  'h1.t-24.t-bold.inline'
]);
const LINKEDIN_DETAIL_COMPANY_ATTRIBUTE_SELECTORS: readonly string[] = Object.freeze([
  '.job-details-jobs-unified-top-card__company-name[title]',
  '.jobs-unified-top-card__company-name[title]'
]);
const LINKEDIN_DETAIL_COMPANY_SELECTORS: readonly string[] = Object.freeze([
  '.job-details-jobs-unified-top-card__company-name',
  '.jobs-unified-top-card__company-name',
  'a.topcard__org-name-link'
]);
const LINKEDIN_DETAIL_LOCATION_SELECTORS: readonly string[] = Object.freeze([
  '.job-details-jobs-unified-top-card__primary-description-container .t-black--light',
  '.job-details-jobs-unified-top-card__bullet',
  '.jobs-unified-top-card__bullet'
]);
const LINKEDIN_DETAIL_DESCRIPTION_SELECTORS: readonly string[] = Object.freeze([
  '.jobs-description-content__text',
  '.jobs-box__html-content',
  '#job-details',
  '.jobs-description__content'
]);
const LINKEDIN_DETAIL_APPLY_SELECTORS: readonly string[] = Object.freeze([
  'a.jobs-apply-button[href]',
  '.jobs-apply-button--top-card a[href]',
  'a[data-live-test-job-apply-button][href]'
]);

export interface LinkedinJobIdentity {
  jobId: string;
}

export function parseLinkedinJob(rawUrl: string): LinkedinJobIdentity | null {
  try {
    const url = new URL(rawUrl);
    if (!LINKEDIN_HOSTS.has(url.hostname)) return null;
    const parts = url.pathname.split('/').filter(Boolean);
    if (parts.length >= 3 && parts[0] === 'jobs' && parts[1] === 'view' && /^\d+$/.test(parts[2])) {
      return { jobId: parts[2] };
    }
    return null;
  } catch {
    return null;
  }
}

function linkedinCanonical(rawUrl: string): string {
  const parsed = parseLinkedinJob(rawUrl);
  if (!parsed) return '';
  return `https://linkedin.com/jobs/view/${parsed.jobId}`;
}

function linkedinEvidence(scope: string, selectors: Record<string, string>): DiscoveryEvidence {
  return { source: 'visible_dom', adapter: SITE.LINKEDIN, scope, verified: false, selectors };
}

function collectLinkedinCards(doc: DomDocument): DomElement[] {
  for (const selector of LINKEDIN_CARD_SELECTORS) {
    const cards = visibleElements(doc, selector);
    if (cards.length) return cards;
  }
  return [];
}

function discoverLinkedinCard(card: DomElement, observedAt: string): DiscoveredJob | null {
  const link = visibleAttributeOf(card, LINKEDIN_CARD_LINK_SELECTORS, 'href');
  const sourceUrl = absoluteHttpsUrl(link.value, LINKEDIN_BASE_URL);
  const canonicalUrl = linkedinCanonical(sourceUrl);
  if (!sourceUrl || !canonicalUrl) return null;

  const title = visibleTextOf(card, LINKEDIN_CARD_TITLE_SELECTORS);
  const company = visibleTextOf(card, LINKEDIN_CARD_COMPANY_SELECTORS);
  const location = visibleTextOf(card, LINKEDIN_CARD_LOCATION_SELECTORS);
  const apply = visibleAttributeOf(card, LINKEDIN_CARD_APPLY_SELECTORS, 'href');

  const selectors: Record<string, string> = { sourceUrl: link.selector, unverified: 'linkedin_job_card' };
  if (title.selector) selectors.roleTitle = title.selector;
  if (company.selector) selectors.companyName = company.selector;
  if (location.selector) selectors.location = location.selector;
  if (apply.selector) selectors.applicationUrl = apply.selector;

  return createDiscoveredJob({
    site: SITE.LINKEDIN,
    sourceUrl,
    canonicalUrl,
    roleTitle: title.value,
    companyName: company.value,
    jobDescription: '',
    location: location.value,
    applicationUrl: apply.value,
    baseUrl: LINKEDIN_BASE_URL,
    contactEmail: '',
    observedAt,
    evidence: linkedinEvidence('job_card', selectors)
  });
}

function discoverLinkedinDetail(doc: DomDocument, rawUrl: string, observedAt: string): DiscoveredJob {
  const canonicalUrl = linkedinCanonical(rawUrl);
  const sourceUrl = absoluteHttpsUrl(rawUrl, LINKEDIN_BASE_URL);
  const title = visibleTextOf(doc, LINKEDIN_DETAIL_TITLE_SELECTORS);
  const companyAttr = visibleAttributeOf(doc, LINKEDIN_DETAIL_COMPANY_ATTRIBUTE_SELECTORS, 'title');
  const companyText = visibleTextOf(doc, LINKEDIN_DETAIL_COMPANY_SELECTORS);
  const location = visibleTextOf(doc, LINKEDIN_DETAIL_LOCATION_SELECTORS);
  const description = visibleTextOf(doc, LINKEDIN_DETAIL_DESCRIPTION_SELECTORS);
  const apply = visibleAttributeOf(doc, LINKEDIN_DETAIL_APPLY_SELECTORS, 'href');

  const selectors: Record<string, string> = { unverified: 'linkedin_job_detail' };
  if (title.selector) selectors.roleTitle = title.selector;
  if (companyAttr.selector) selectors.companyName = companyAttr.selector;
  else if (companyText.selector) selectors.companyName = companyText.selector;
  if (location.selector) selectors.location = location.selector;
  if (description.selector) selectors.jobDescription = description.selector;
  if (apply.selector) selectors.applicationUrl = apply.selector;

  return createDiscoveredJob({
    site: SITE.LINKEDIN,
    sourceUrl,
    canonicalUrl: canonicalUrl || sourceUrl,
    roleTitle: title.value,
    companyName: companyAttr.value || companyText.value,
    jobDescription: description.value,
    location: location.value,
    applicationUrl: apply.value,
    baseUrl: rawUrl,
    contactEmail: pickContactEmail(description.value),
    observedAt,
    evidence: linkedinEvidence('job_detail', selectors)
  });
}

export const linkedinAdapter: DiscoveryAdapter = {
  id: SITE.LINKEDIN,
  label: 'LinkedIn',
  verified: false,
  evidence: 'evidence/linkedin-2026-11-10/',
  capabilities: { scan: true, fill: false, submit: false, verifiedConfirmation: false },

  matches(rawUrl: string): boolean {
    return Boolean(parseLinkedinJob(rawUrl));
  },

  matchesDiscovery(rawUrl: string): boolean {
    try {
      return LINKEDIN_HOSTS.has(new URL(rawUrl).hostname);
    } catch {
      return false;
    }
  },

  discover(doc: DomDocument | null, rawUrl: string, options: { now?: number } = {}): DiscoveryResult {
    const site = SITE.LINKEDIN;
    if (!doc) {
      return {
        supported: true,
        site,
        jobs: [],
        blocked: {
          reason: REASON.UNVERIFIED_FORM,
          detail: 'No document was available to inspect visible LinkedIn job content.'
        }
      };
    }
    if (detectVisibleLoginWall(doc, rawUrl)) {
      return {
        supported: true,
        site,
        jobs: [],
        blocked: {
          reason: REASON.LOGIN_REQUIRED,
          detail: 'LinkedIn sign-in is required; no visible job cards or job detail were read.'
        }
      };
    }
    if (detectCaptchaChallenge(doc)) {
      return {
        supported: true,
        site,
        jobs: [],
        blocked: {
          reason: REASON.CAPTCHA_REQUIRED,
          detail: 'A LinkedIn anti-bot or CAPTCHA challenge was detected; no job data was read.'
        }
      };
    }

    const observedAt = nowIso(options.now);
    const detailTitle = visibleTextOf(doc, LINKEDIN_DETAIL_TITLE_SELECTORS);
    if (detailTitle.value) {
      const job = discoverLinkedinDetail(doc, rawUrl, observedAt);
      return { supported: true, site, jobs: dedupeJobsByCanonicalUrl([job]).slice(0, DISCOVERY_LIMITS.maxJobs) };
    }

    const cards = collectLinkedinCards(doc);
    const jobs = dedupeJobsByCanonicalUrl(cards.map((card) => discoverLinkedinCard(card, observedAt)).filter(Boolean)).slice(0, DISCOVERY_LIMITS.maxJobs);
    if (!jobs.length) {
      return {
        supported: true,
        site,
        jobs: [],
        blocked: {
          reason: REASON.UNVERIFIED_FORM,
          detail: 'The visible LinkedIn layout matched no known job card or job detail structure; LinkedIn selectors remain unverified and no data was extracted.'
        }
      };
    }
    return { supported: true, site, jobs };
  },

  canonicalize(rawUrl: string): string {
    const parsed = parseLinkedinJob(rawUrl);
    if (!parsed) throw new Error('Not a LinkedIn job post URL.');
    return `https://linkedin.com/jobs/view/${parsed.jobId}`;
  },

  scan(doc: DomDocument, rawUrl: string): Scan {
    const canonicalUrl = this.canonicalize(rawUrl);
    const isLogin = detectLoginWall(doc, rawUrl);
    const reasonCode = isLogin ? REASON.LOGIN_REQUIRED : REASON.UNSUPPORTED_FORM;
    const detail = isLogin
      ? 'LinkedIn sign-in is required; unauthenticated view has no accessible form.'
      : 'LinkedIn returned 429 rate-limit / anti-bot challenge; no job details or form selectors are verified.';

    return {
      supported: true,
      site: SITE.LINKEDIN,
      canonicalUrl,
      sourceUrl: canonicalUrl,
      job: {
        title: '',
        companyName: '',
        description: ''
      },
      form: null,
      blocked: {
        reason: reasonCode,
        detail
      },
      capabilities: this.capabilities,
      evidence: this.evidence
    };
  },

  fill(_doc: DomDocument, _preparedFields: PreparedFields, _resume: ResumeAttachment | null): FillReport {
    throw new Error('LinkedIn form selectors are unverified; automated fill is blocked.');
  },

  submit(): SubmitResult {
    throw new Error('LinkedIn form selectors are unverified; automated submit is blocked.');
  },

  confirmation(): ConfirmationResult {
    return { confirmed: false, reason: 'unverified' };
  }
};
