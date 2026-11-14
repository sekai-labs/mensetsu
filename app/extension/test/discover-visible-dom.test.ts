import { describe, it } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';

import { parseHtml, type MockDocument } from './fixtures/dom-helper.js';
import { topcvAdapter } from '../src/content/adapters/topcv.js';
import { linkedinAdapter } from '../src/content/adapters/linkedin.js';
import { getAdapterForDiscoveryUrl } from '../src/content/adapters/index.js';
import { DISCOVERY_LIMITS } from '../src/content/adapters/dom.js';
import { handleMessage } from '../src/content/content.js';
import { MESSAGE, REASON, SITE } from '../src/shared/constants.js';
import type { DiscoveredJob, DiscoveryResult } from '../src/shared/types.js';

const topcvListHtml = fs.readFileSync(new URL('./fixtures/discover-topcv-list.synthetic.html', import.meta.url), 'utf8');
const topcvDetailHtml = fs.readFileSync(new URL('./fixtures/discover-topcv-detail.synthetic.html', import.meta.url), 'utf8');
const linkedinListHtml = fs.readFileSync(new URL('./fixtures/discover-linkedin-list.synthetic.html', import.meta.url), 'utf8');
const linkedinDetailHtml = fs.readFileSync(new URL('./fixtures/discover-linkedin-detail.synthetic.html', import.meta.url), 'utf8');
const linkedinLoginHtml = fs.readFileSync(new URL('./fixtures/discover-linkedin-login.synthetic.html', import.meta.url), 'utf8');

const TOPCV_LIST_URL = 'https://www.topcv.vn/tim-viec-lam-cong-nghe-thong-tin';
const TOPCV_DETAIL_URL = 'https://www.topcv.vn/viec-lam/backend-developer-java-upto-24m/2345299.html';
const LINKEDIN_LIST_URL = 'https://www.linkedin.com/jobs/search/?keywords=backend';
const LINKEDIN_DETAIL_URL = 'https://www.linkedin.com/jobs/view/4400000000/';
const SYNTHETIC_NOW = Date.parse('2026-11-10T00:00:00Z');

function assertIsoTimestamp(value: string): void {
  assert.equal(typeof value, 'string');
  assert.ok(value.length > 0);
  assert.ok(!Number.isNaN(Date.parse(value)));
}

function assertJobShape(job: DiscoveredJob): void {
  for (const key of [
    'site',
    'sourceUrl',
    'canonicalUrl',
    'roleTitle',
    'companyName',
    'jobDescription',
    'location',
    'applicationUrl',
    'contactEmail',
    'observedAt',
    'completeness',
    'evidence'
  ]) {
    assert.ok(Object.hasOwn(job, key), `job exposes ${key}`);
  }
  assert.equal(job.evidence.source, 'visible_dom');
  assert.equal(job.evidence.verified, false);
  assert.equal(typeof job.completeness.present, 'number');
  assert.equal(typeof job.completeness.total, 'number');
  assert.equal(typeof job.completeness.complete, 'boolean');
  assertIsoTimestamp(job.observedAt);
}

async function withPage<T>(url: string, doc: MockDocument, fn: () => Promise<T> | T): Promise<T> {
  const previousWindow = globalThis.window;
  const previousDocument = globalThis.document;
  globalThis.window = { location: { href: url } } as unknown as Window & typeof globalThis;
  globalThis.document = doc as unknown as Document;
  try {
    return await fn();
  } finally {
    if (previousWindow === undefined) delete (globalThis as { window?: unknown }).window;
    else globalThis.window = previousWindow;
    if (previousDocument === undefined) delete (globalThis as { document?: unknown }).document;
    else globalThis.document = previousDocument;
  }
}

describe('Synthetic visible-DOM discovery', () => {
  describe('TopCV listing layout', () => {
    const doc = parseHtml(topcvListHtml, 'Việc làm Công nghệ thông tin | TopCV');
    const result = topcvAdapter.discover(doc, TOPCV_LIST_URL, { now: SYNTHETIC_NOW });

    it('returns supported TopCV listing jobs with bounded visible card fields only', () => {
      assert.equal(result.supported, true);
      assert.equal(result.site, SITE.TOPCV);
      assert.equal(result.blocked, undefined);
      assert.equal(result.jobs.length, 3);

      const [first] = result.jobs;
      assertJobShape(first);
      assert.equal(first.site, SITE.TOPCV);
      assert.equal(first.roleTitle, 'Thực Tập Sinh CNTT Được Đào Tạo');
      assert.equal(first.companyName, 'CÔNG TY TNHH CÔNG NGHỆ PHẦN MỀM NASANI');
      assert.equal(first.location, 'Hồ Chí Minh');
      assert.equal(first.jobDescription, '');
      assert.equal(first.contactEmail, '');
      assert.equal(first.applicationUrl, '');
      assert.equal(first.sourceUrl, 'https://www.topcv.vn/viec-lam/thuc-tap-sinh-cntt/2349247.html');
      assert.equal(first.canonicalUrl, 'https://topcv.vn/viec-lam/thuc-tap-sinh-cntt/2349247.html');
      assert.equal(first.observedAt, new Date(SYNTHETIC_NOW).toISOString());
      assert.equal(first.completeness.complete, false);
      assert.equal(first.evidence.scope, 'job_card');
      assert.equal(first.evidence.adapter, SITE.TOPCV);
    });

    it('keeps observed application URLs from the card and drops javascript apply controls', () => {
      const second = result.jobs[1];
      assert.equal(second.roleTitle, 'Backend Developer (Java)');
      assert.equal(second.companyName, 'CÔNG TY TNHH DỊCH VỤ SỐ DIGILIFE VIỆT NAM');
      assert.equal(second.location, 'Hà Nội');
      assert.equal(second.canonicalUrl, 'https://topcv.vn/viec-lam/backend-developer-java-upto-24m/2345299.html');
      assert.match(second.applicationUrl, /^https:\/\/www\.topcv\.vn\/viec-lam\/backend-developer-java-upto-24m\/2345299\.html\?apply-form=1$/);
    });

    it('deduplicates canonical URLs and keeps the first observed card', () => {
      const canonicals = result.jobs.map((job) => job.canonicalUrl);
      assert.equal(new Set(canonicals).size, canonicals.length);
      assert.ok(!canonicals.some((url) => url.includes('duplicate-card')));
      assert.equal(result.jobs.filter((job) => job.canonicalUrl === 'https://topcv.vn/viec-lam/backend-developer-java-upto-24m/2345299.html').length, 1);
    });

    it('skips hidden cards and their links', () => {
      const canonicalText = result.jobs.map((job) => job.canonicalUrl).join('\n');
      assert.ok(!canonicalText.includes('9999001'));
      assert.ok(!canonicalText.includes('9999002'));
    });

    it('leaves missing card fields empty instead of inventing data', () => {
      const partial = result.jobs[2];
      assert.equal(partial.roleTitle, '');
      assert.equal(partial.companyName, '');
      assert.equal(partial.location, '');
      assert.equal(partial.jobDescription, '');
      assert.equal(partial.sourceUrl, 'https://www.topcv.vn/viec-lam/chi-tiet-cong-viec/8888001.html');
      assert.equal(partial.canonicalUrl, 'https://topcv.vn/viec-lam/chi-tiet-cong-viec/8888001.html');
      assert.equal(partial.completeness.present, 0);
      assert.equal(partial.completeness.complete, false);
    });

    it('never attributes page-wide, sidebar, or profile emails to listing cards', () => {
      for (const job of result.jobs) {
        assert.equal(job.contactEmail, '');
        assert.ok(!job.jobDescription.includes('@'));
      }
    });
  });

  describe('TopCV job detail layout', () => {
    const doc = parseHtml(topcvDetailHtml, 'Backend Developer (Java) - TopCV');
    const result = topcvAdapter.discover(doc, TOPCV_DETAIL_URL, { now: SYNTHETIC_NOW });

    it('extracts the visible detail job as a single job entry', () => {
      assert.equal(result.supported, true);
      assert.equal(result.site, SITE.TOPCV);
      assert.equal(result.jobs.length, 1);

      const [job] = result.jobs;
      assertJobShape(job);
      assert.match(job.roleTitle, /Backend Developer/);
      assert.equal(job.companyName, 'CÔNG TY TNHH DỊCH VỤ SỐ DIGILIFE VIỆT NAM');
      assert.equal(job.location, 'Hồ Chí Minh');
      assert.match(job.jobDescription, /Mô tả công việc/);
      assert.equal(job.sourceUrl, TOPCV_DETAIL_URL);
      assert.equal(job.canonicalUrl, 'https://topcv.vn/viec-lam/backend-developer-java-upto-24m/2345299.html');
      assert.equal(job.completeness.complete, true);
      assert.equal(job.evidence.scope, 'job_detail');
    });

    it('attributes contact email only from the visible job description region', () => {
      const [job] = result.jobs;
      assert.equal(job.contactEmail, 'tuyendung@digilife.example');
      assert.ok(!job.jobDescription.includes('hidden-block@digilife.example'));
      assert.ok(!job.jobDescription.includes('hidden-class@digilife.example'));
      assert.notEqual(job.contactEmail, 'hr.sidebar@example.com');
      assert.notEqual(job.contactEmail, 'ungvien@example.com');
      assert.notEqual(job.contactEmail, 'hotro@topcv.example');
      assert.notEqual(job.contactEmail, 'support@example.com');
    });

    it('drops login-gated javascript apply controls instead of inventing an application URL', () => {
      const [job] = result.jobs;
      assert.equal(job.applicationUrl, '');
    });

    it('fails closed for unknown TopCV layouts', () => {
      const unknown = parseHtml('<html><body><p>Trang không tồn tại</p></body></html>', 'TopCV');
      const blocked = topcvAdapter.discover(unknown, TOPCV_LIST_URL);
      assert.equal(blocked.supported, true);
      assert.equal(blocked.site, SITE.TOPCV);
      assert.deepEqual(blocked.jobs, []);
      assert.equal(blocked.blocked?.reason, REASON.UNVERIFIED_FORM);
    });

    it('fails closed when TopCV serves a login wall', () => {
      const login = parseHtml('<html><body><section class="login"><h1>Đăng nhập</h1><input type="text" name="email"><input type="password" name="password"></section></body></html>', 'Đăng nhập | TopCV');
      const blocked = topcvAdapter.discover(login, TOPCV_LIST_URL);
      assert.equal(blocked.supported, true);
      assert.equal(blocked.site, SITE.TOPCV);
      assert.deepEqual(blocked.jobs, []);
      assert.equal(blocked.blocked?.reason, REASON.LOGIN_REQUIRED);
    });
  });

  describe('LinkedIn listing layout', () => {
    const doc = parseHtml(linkedinListHtml, 'Job search | LinkedIn');
    const result = linkedinAdapter.discover(doc, LINKEDIN_LIST_URL, { now: SYNTHETIC_NOW });

    it('returns unverified LinkedIn cards without claiming live-site verification', () => {
      assert.equal(result.supported, true);
      assert.equal(result.site, SITE.LINKEDIN);
      assert.equal(result.blocked, undefined);
      assert.equal(result.jobs.length, 2);

      const [first, second] = result.jobs;
      assertJobShape(first);
      assert.equal(first.site, SITE.LINKEDIN);
      assert.equal(first.roleTitle, 'Senior Backend Engineer');
      assert.equal(first.companyName, 'Acme Corp');
      assert.equal(first.location, 'Ho Chi Minh City, Vietnam');
      assert.equal(first.jobDescription, '');
      assert.equal(first.contactEmail, '');
      assert.equal(first.applicationUrl, '');
      assert.equal(first.canonicalUrl, 'https://linkedin.com/jobs/view/4400000000');
      assert.equal(first.sourceUrl, 'https://www.linkedin.com/jobs/view/4400000000/?eBP=CwEAAAG');
      assert.equal(first.evidence.adapter, SITE.LINKEDIN);
      assert.equal(first.evidence.selectors.unverified, 'linkedin_job_card');

      assert.equal(second.roleTitle, 'Data Platform Engineer');
      assert.equal(second.companyName, 'Beta Ltd');
      assert.equal(second.location, 'Da Nang, Vietnam');
      assert.equal(second.canonicalUrl, 'https://linkedin.com/jobs/view/4400000001');
    });

    it('deduplicates LinkedIn canonical URLs and skips hidden cards', () => {
      const canonicals = result.jobs.map((job) => job.canonicalUrl);
      assert.equal(new Set(canonicals).size, 2);
      assert.ok(!canonicals.some((url) => url.includes('4400000002')));
      assert.equal(result.jobs[0].companyName, 'Acme Corp');
    });

    it('does not attribute navigation, sidebar, or profile emails to LinkedIn cards', () => {
      for (const job of result.jobs) {
        assert.equal(job.contactEmail, '');
      }
    });
  });

  describe('LinkedIn job detail layout', () => {
    const doc = parseHtml(linkedinDetailHtml, 'Staff Platform Engineer | LinkedIn');
    const result = linkedinAdapter.discover(doc, LINKEDIN_DETAIL_URL, { now: SYNTHETIC_NOW });

    it('extracts visible detail fields and keeps the layout marked unverified', () => {
      assert.equal(result.supported, true);
      assert.equal(result.jobs.length, 1);

      const [job] = result.jobs;
      assertJobShape(job);
      assert.equal(job.roleTitle, 'Staff Platform Engineer');
      assert.equal(job.companyName, 'Acme Corp');
      assert.equal(job.location, 'Ho Chi Minh City, Vietnam');
      assert.match(job.jobDescription, /Own the platform runtime/);
      assert.equal(job.canonicalUrl, 'https://linkedin.com/jobs/view/4400000000');
      assert.equal(job.sourceUrl, LINKEDIN_DETAIL_URL);
      assert.equal(job.completeness.complete, true);
      assert.equal(job.evidence.scope, 'job_detail');
      assert.equal(job.evidence.selectors.unverified, 'linkedin_job_detail');
      assert.equal(job.applicationUrl, 'https://www.linkedin.com/jobs/view/4400000000/apply/?openSDUIApplyFlow=true');
    });

    it('attributes contact email only from the visible LinkedIn job description region', () => {
      const [job] = result.jobs;
      assert.equal(job.contactEmail, 'hiring@acme.example');
      assert.ok(!job.jobDescription.includes('hidden-block@acme.example'));
      assert.ok(!job.jobDescription.includes('hidden-class@acme.example'));
      assert.notEqual(job.contactEmail, 'hr.sidebar@acme.example');
      assert.notEqual(job.contactEmail, 'mai.tran@example.com');
    });

    it('fails closed for unknown LinkedIn layouts', () => {
      const unknown = parseHtml('<html><body><div class="feed">No job posts</div></body></html>', 'LinkedIn');
      const blocked = linkedinAdapter.discover(unknown, LINKEDIN_LIST_URL);
      assert.equal(blocked.supported, true);
      assert.equal(blocked.site, SITE.LINKEDIN);
      assert.deepEqual(blocked.jobs, []);
      assert.equal(blocked.blocked?.reason, REASON.UNVERIFIED_FORM);
    });

    it('fails closed when LinkedIn serves a login wall', () => {
      const login = parseHtml(linkedinLoginHtml, 'Sign in | LinkedIn');
      const blocked = linkedinAdapter.discover(login, 'https://www.linkedin.com/jobs/');
      assert.equal(blocked.supported, true);
      assert.equal(blocked.site, SITE.LINKEDIN);
      assert.deepEqual(blocked.jobs, []);
      assert.equal(blocked.blocked?.reason, REASON.LOGIN_REQUIRED);
    });
  });

  describe('Visibility and bounds', () => {
    it('clamps long visible descriptions to the discovery description limit', () => {
      const longText = 'x'.repeat(DISCOVERY_LIMITS.maxDescriptionChars + 2500);
      const html = `<html><body><h1 class="box-header-job__title">Long Role</h1><div class="job-description__content">${longText}</div></body></html>`;
      const doc = parseHtml(html, 'TopCV');
      const result = topcvAdapter.discover(doc, TOPCV_DETAIL_URL);
      assert.equal(result.jobs.length, 1);
      assert.equal(result.jobs[0].jobDescription.length, DISCOVERY_LIMITS.maxDescriptionChars);
    });

    it('bounds the number of discovered cards', () => {
      const cards: string[] = [];
      for (let index = 0; index < DISCOVERY_LIMITS.maxJobs + 5; index += 1) {
        const id = 1000000 + index;
        cards.push(`<div class="job-item-search-result"><h3 class="title"><a href="https://www.topcv.vn/viec-lam/role-${index}/${id}.html"><span title="Role ${index}">Role ${index}</span></a></h3></div>`);
      }
      const doc = parseHtml(`<html><body><main>${cards.join('')}</main></body></html>`, 'TopCV');
      const result = topcvAdapter.discover(doc, TOPCV_LIST_URL);
      assert.equal(result.jobs.length, DISCOVERY_LIMITS.maxJobs);
      assert.equal(
        result.jobs.at(-1)?.canonicalUrl,
        `https://topcv.vn/viec-lam/role-${DISCOVERY_LIMITS.maxJobs - 1}/${1000000 + DISCOVERY_LIMITS.maxJobs - 1}.html`
      );
    });
  });

  describe('Discovery registry and content script routing', () => {
    it('resolves discovery adapters by host and rejects unsupported sites', () => {
      assert.equal(getAdapterForDiscoveryUrl(TOPCV_LIST_URL), topcvAdapter);
      assert.equal(getAdapterForDiscoveryUrl(TOPCV_DETAIL_URL), topcvAdapter);
      assert.equal(getAdapterForDiscoveryUrl(LINKEDIN_LIST_URL), linkedinAdapter);
      assert.equal(getAdapterForDiscoveryUrl(LINKEDIN_DETAIL_URL), linkedinAdapter);
      assert.equal(getAdapterForDiscoveryUrl('https://boards.greenhouse.io/gitlab/jobs/8629019002'), null);
      assert.equal(getAdapterForDiscoveryUrl('https://example.com/careers'), null);
      assert.equal(getAdapterForDiscoveryUrl('not a url'), null);
    });

    it('returns an unsupported discovery result for sites without a discovery adapter', async () => {
      const doc = parseHtml('<html><body><h1>Careers</h1></body></html>', 'Careers');
      const result = await withPage('https://example.com/careers', doc, () => handleMessage<DiscoveryResult>({ type: MESSAGE.DISCOVER }));
      assert.equal(result.supported, false);
      assert.equal(result.site, '');
      assert.deepEqual(result.jobs, []);
      assert.equal(result.blocked?.reason, REASON.UNSUPPORTED_SITE);
    });

    it('routes MESSAGE.DISCOVER to the active tab discovery adapter', async () => {
      const doc = parseHtml(topcvListHtml, 'Việc làm Công nghệ thông tin | TopCV');
      const result = await withPage(TOPCV_LIST_URL, doc, () => handleMessage<DiscoveryResult>({ type: MESSAGE.DISCOVER }));
      assert.equal(result.supported, true);
      assert.equal(result.site, SITE.TOPCV);
      assert.equal(result.jobs.length, 3);
      assert.equal(new Set(result.jobs.map((job) => job.canonicalUrl)).size, 3);
    });
  });
});
