import { REASON, SITE } from '../../shared/constants.js';
import { normalizeText, nowIso } from '../../shared/util.js';
import {
  absoluteHttpsUrl,
  createDiscoveredJob,
  dedupeJobsByCanonicalUrl,
  DISCOVERY_LIMITS,
  detectCaptchaChallenge,
  detectVisibleLoginWall,
  pickContactEmail,
  textOf,
  visibleAttributeOf,
  visibleElements,
  visibleTextOf
} from './dom.js';
import type { DomDocument, DomElement } from './dom-types.js';
import type {
  DiscoveredJob,
  DiscoveryEvidence,
  DiscoveryResult,
  FillReport,
  PreparedFields,
  ResumeAttachment,
  Scan,
  DiscoveryAdapter,
  SubmitResult,
  ConfirmationResult
} from '../../shared/types.js';

const TOPCV_HOSTS = new Set(['topcv.vn', 'www.topcv.vn']);
const TOPCV_BASE_URL = 'https://www.topcv.vn';
const TOPCV_CARD_SELECTORS: readonly string[] = Object.freeze([
  'div.job-item-search-result',
  'li.job-item-search-result',
  'div.job-item'
]);
const TOPCV_CARD_LINK_SELECTORS: readonly string[] = Object.freeze([
  'h3.title a[href]',
  'a[href*="/viec-lam/"]'
]);
const TOPCV_CARD_TITLE_ATTRIBUTE_SELECTORS: readonly string[] = Object.freeze([
  'h3.title a span[title]',
  'h3.title span[title]'
]);
const TOPCV_CARD_TITLE_SELECTORS: readonly string[] = Object.freeze(['h3.title a', 'h3.title']);
const TOPCV_CARD_COMPANY_ATTRIBUTE_SELECTORS: readonly string[] = Object.freeze(['.company-name[title]', 'a.company[title]']);
const TOPCV_CARD_COMPANY_SELECTORS: readonly string[] = Object.freeze(['.company-name', 'a.company']);
const TOPCV_CARD_LOCATION_SELECTORS: readonly string[] = Object.freeze(['.city-text', '.address .city-text', '.address']);
const TOPCV_CARD_APPLY_ATTRIBUTE_SELECTORS: readonly string[] = Object.freeze([
  'button.btn-apply-now[data-apply-url]',
  'a.btn-apply-job[data-apply-url]'
]);
const TOPCV_CARD_APPLY_HREF_SELECTORS: readonly string[] = Object.freeze(['a.btn-apply-job[href]', 'a.btn-apply[href]']);
const TOPCV_DETAIL_TITLE_SELECTORS: readonly string[] = Object.freeze(['h1.box-header-job__title']);
const TOPCV_DETAIL_COMPANY_ATTRIBUTE_SELECTORS: readonly string[] = Object.freeze([
  '.box-company-info__detail a.name[title]',
  '.box-company-info-detail__company--link a[title]'
]);
const TOPCV_DETAIL_COMPANY_SELECTORS: readonly string[] = Object.freeze([
  '.box-company-info__detail a.name',
  '.box-company-info-detail__company--link a',
  'a.company-name'
]);
const TOPCV_DETAIL_LOCATION_SELECTORS: readonly string[] = Object.freeze([
  '.box-address',
  '.box-job-information-general-info-list__item.place',
  '.job-detail-info .address'
]);
const TOPCV_DETAIL_DESCRIPTION_SELECTORS: readonly string[] = Object.freeze([
  '.job-description__content',
  '.job-description',
  '.box-job-description',
  '#job-description',
  '.job-data__content',
  '.box-job-information-detail'
]);
const TOPCV_DETAIL_APPLY_SELECTORS: readonly string[] = Object.freeze([
  'a.btn-apply-job[href]',
  '.box-group-button-apply a[href]',
  'a.btn-apply.btn-apply-job[href]'
]);

export interface TopcvJobIdentity {
  path: string;
}

export function parseTopcvJob(rawUrl: string): TopcvJobIdentity | null {
  try {
    const url = new URL(rawUrl);
    if (!TOPCV_HOSTS.has(url.hostname)) return null;
    const path = url.pathname.replace(/\/+$/, '');
    if (path.startsWith('/viec-lam-') || path.startsWith('/viec-lam/')) {
      return { path };
    }
    return null;
  } catch {
    return null;
  }
}

function topcvCanonical(rawUrl: string): string {
  const parsed = parseTopcvJob(rawUrl);
  if (!parsed) return '';
  return `https://topcv.vn${parsed.path}`;
}

function topcvEvidence(scope: string, selectors: Record<string, string>): DiscoveryEvidence {
  return { source: 'visible_dom', adapter: SITE.TOPCV, scope, verified: false, selectors };
}

function collectTopcvCards(doc: DomDocument): DomElement[] {
  for (const selector of TOPCV_CARD_SELECTORS) {
    const cards = visibleElements(doc, selector);
    if (cards.length) return cards;
  }
  return [];
}

function discoverTopcvCard(card: DomElement, observedAt: string): DiscoveredJob | null {
  const link = visibleAttributeOf(card, TOPCV_CARD_LINK_SELECTORS, 'href');
  const sourceUrl = absoluteHttpsUrl(link.value, TOPCV_BASE_URL);
  const canonicalUrl = topcvCanonical(sourceUrl);
  if (!sourceUrl || !canonicalUrl) return null;

  const titleAttr = visibleAttributeOf(card, TOPCV_CARD_TITLE_ATTRIBUTE_SELECTORS, 'title');
  const titleText = visibleTextOf(card, TOPCV_CARD_TITLE_SELECTORS);
  const companyAttr = visibleAttributeOf(card, TOPCV_CARD_COMPANY_ATTRIBUTE_SELECTORS, 'title');
  const companyText = visibleTextOf(card, TOPCV_CARD_COMPANY_SELECTORS);
  const location = visibleTextOf(card, TOPCV_CARD_LOCATION_SELECTORS);
  const applyAttr = visibleAttributeOf(card, TOPCV_CARD_APPLY_ATTRIBUTE_SELECTORS, 'data-apply-url');
  const applyHref = visibleAttributeOf(card, TOPCV_CARD_APPLY_HREF_SELECTORS, 'href');

  const selectors: Record<string, string> = { sourceUrl: link.selector };
  if (titleAttr.selector) selectors.roleTitle = titleAttr.selector;
  else if (titleText.selector) selectors.roleTitle = titleText.selector;
  if (companyAttr.selector) selectors.companyName = companyAttr.selector;
  else if (companyText.selector) selectors.companyName = companyText.selector;
  if (location.selector) selectors.location = location.selector;
  if (applyAttr.selector) selectors.applicationUrl = applyAttr.selector;
  else if (applyHref.selector) selectors.applicationUrl = applyHref.selector;

  return createDiscoveredJob({
    site: SITE.TOPCV,
    sourceUrl,
    canonicalUrl,
    roleTitle: titleAttr.value || titleText.value,
    companyName: companyAttr.value || companyText.value,
    jobDescription: '',
    location: location.value,
    applicationUrl: applyAttr.value || applyHref.value,
    baseUrl: TOPCV_BASE_URL,
    contactEmail: '',
    observedAt,
    evidence: topcvEvidence('job_card', selectors)
  });
}

function discoverTopcvDetail(doc: DomDocument, rawUrl: string, observedAt: string): DiscoveredJob {
  const canonicalUrl = topcvCanonical(rawUrl);
  const sourceUrl = absoluteHttpsUrl(rawUrl, TOPCV_BASE_URL);
  const title = visibleTextOf(doc, TOPCV_DETAIL_TITLE_SELECTORS);
  const companyAttr = visibleAttributeOf(doc, TOPCV_DETAIL_COMPANY_ATTRIBUTE_SELECTORS, 'title');
  const companyText = visibleTextOf(doc, TOPCV_DETAIL_COMPANY_SELECTORS);
  const location = visibleTextOf(doc, TOPCV_DETAIL_LOCATION_SELECTORS);
  const description = visibleTextOf(doc, TOPCV_DETAIL_DESCRIPTION_SELECTORS);
  const apply = visibleAttributeOf(doc, TOPCV_DETAIL_APPLY_SELECTORS, 'href');

  const selectors: Record<string, string> = {};
  if (title.selector) selectors.roleTitle = title.selector;
  if (companyAttr.selector) selectors.companyName = companyAttr.selector;
  else if (companyText.selector) selectors.companyName = companyText.selector;
  if (location.selector) selectors.location = location.selector;
  if (description.selector) selectors.jobDescription = description.selector;
  if (apply.selector) selectors.applicationUrl = apply.selector;

  return createDiscoveredJob({
    site: SITE.TOPCV,
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
    evidence: topcvEvidence('job_detail', selectors)
  });
}

export const topcvAdapter: DiscoveryAdapter = {
  id: SITE.TOPCV,
  label: 'TopCV',
  verified: false,
  evidence: 'evidence/topcv-2026-11-10/',
  capabilities: { scan: true, fill: false, submit: false, verifiedConfirmation: false },

  matches(rawUrl: string): boolean {
    return Boolean(parseTopcvJob(rawUrl));
  },

  matchesDiscovery(rawUrl: string): boolean {
    try {
      return TOPCV_HOSTS.has(new URL(rawUrl).hostname);
    } catch {
      return false;
    }
  },

  discover(doc: DomDocument | null, rawUrl: string, options: { now?: number } = {}): DiscoveryResult {
    const site = SITE.TOPCV;
    if (!doc) {
      return {
        supported: true,
        site,
        jobs: [],
        blocked: {
          reason: REASON.UNVERIFIED_FORM,
          detail: 'No document was available to inspect visible TopCV job content.'
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
          detail: 'TopCV sign-in is required; no visible job cards or job detail were read.'
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
          detail: 'A TopCV CAPTCHA challenge was detected; no job data was read.'
        }
      };
    }

    const observedAt = nowIso(options.now);
    const detailTitle = visibleTextOf(doc, TOPCV_DETAIL_TITLE_SELECTORS);
    if (detailTitle.value) {
      const job = discoverTopcvDetail(doc, rawUrl, observedAt);
      return { supported: true, site, jobs: dedupeJobsByCanonicalUrl([job]).slice(0, DISCOVERY_LIMITS.maxJobs) };
    }

    const cards = collectTopcvCards(doc);
    const jobs = dedupeJobsByCanonicalUrl(cards.map((card) => discoverTopcvCard(card, observedAt)).filter(Boolean)).slice(0, DISCOVERY_LIMITS.maxJobs);
    if (!jobs.length) {
      return {
        supported: true,
        site,
        jobs: [],
        blocked: {
          reason: REASON.UNVERIFIED_FORM,
          detail: 'The visible TopCV layout matched no known job card or job detail structure; no data was extracted.'
        }
      };
    }
    return { supported: true, site, jobs };
  },

  canonicalize(rawUrl: string): string {
    const parsed = parseTopcvJob(rawUrl);
    if (!parsed) throw new Error('Not a TopCV job post URL.');
    return `https://topcv.vn${parsed.path}`;
  },

  scan(doc: DomDocument, rawUrl: string): Scan {
    const canonicalUrl = this.canonicalize(rawUrl);
    const title = textOf(doc, 'h1.box-header-job__title');
    const companyLink = doc.querySelector?.('.box-company-info-detail__company--link a') ?? null;
    const companyName = textOf(doc, '.box-company-info__detail a.name')
      || normalizeText(companyLink?.getAttribute?.('title') || '');

    return {
      supported: true,
      site: SITE.TOPCV,
      canonicalUrl,
      sourceUrl: canonicalUrl,
      job: {
        title,
        companyName,
        description: '',
        sourceTitle: normalizeText(doc.title || '')
      },
      form: null,
      blocked: {
        reason: REASON.LOGIN_REQUIRED,
        detail: 'TopCV application is login-gated; form selectors are unverified in public evidence. Manual application required.'
      },
      capabilities: this.capabilities,
      evidence: this.evidence
    };
  },

  fill(_doc: DomDocument, _preparedFields: PreparedFields, _resume: ResumeAttachment | null): FillReport {
    throw new Error('TopCV form is unverified and login-gated; automated fill is blocked.');
  },

  submit(): SubmitResult {
    throw new Error('TopCV form is unverified and login-gated; automated submit is blocked.');
  },

  confirmation(): ConfirmationResult {
    return { confirmed: false, reason: 'unverified' };
  }
};
