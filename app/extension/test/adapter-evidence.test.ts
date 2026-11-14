import { describe, it } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { parseHtml } from './fixtures/dom-helper.js';
import { greenhouseAdapter } from '../src/content/adapters/greenhouse.js';
import { topcvAdapter } from '../src/content/adapters/topcv.js';
import { linkedinAdapter } from '../src/content/adapters/linkedin.js';
import { getAdapterForUrl } from '../src/content/adapters/index.js';
import { REASON, SITE } from '../src/shared/constants.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const evidenceDir = path.resolve(__dirname, '..', 'evidence');

interface PageMetadata {
  source_url: string;
}

describe('Adapter Evidence Verification', () => {
  describe('Greenhouse Adapter Evidence', () => {
    const greenhouseDir = path.join(evidenceDir, 'greenhouse-2026-11-10');
    const formHtml = fs.readFileSync(path.join(greenhouseDir, 'job-post.application-form.sanitized.html'), 'utf8');
    const confirmationHtml = fs.readFileSync(path.join(greenhouseDir, 'confirmation.sanitized.html'), 'utf8');
    const metadata = JSON.parse(fs.readFileSync(path.join(greenhouseDir, 'job-post.metadata.json'), 'utf8')) as PageMetadata;

    it('matches Greenhouse job post URLs and canonicalizes to stable identity', () => {
      const url1 = 'https://job-boards.greenhouse.io/gitlab/jobs/8629019002';
      const url2 = 'https://boards.greenhouse.io/gitlab/jobs/8629019002';
      assert.strictEqual(greenhouseAdapter.matches(url1), true);
      assert.strictEqual(greenhouseAdapter.matches(url2), true);
      assert.strictEqual(greenhouseAdapter.canonicalize(url1), 'https://boards.greenhouse.io/gitlab/8629019002');
      assert.strictEqual(greenhouseAdapter.canonicalize(url2), 'https://boards.greenhouse.io/gitlab/8629019002');
    });

    it('scans captured Greenhouse form and extracts observed core fields', () => {
      const doc = parseHtml(formHtml, 'Job Application for Director, Support at GitLab');
      const scan = greenhouseAdapter.scan(doc, metadata.source_url);

      assert.strictEqual(scan.supported, true);
      assert.strictEqual(scan.site, SITE.GREENHOUSE);
      assert.ok(scan.job, 'Job details were found');
      assert.strictEqual(scan.job.companyName, 'GitLab');
      assert.ok(scan.form, 'Form was found');
      assert.strictEqual(scan.form.selector, 'form#application-form');
      assert.ok(scan.form.fields, 'Form fields were collected');

      const fieldIds = scan.form.fields.map((field) => field.id);
      assert.ok(fieldIds.includes('first_name'), 'contains first_name');
      assert.ok(fieldIds.includes('last_name'), 'contains last_name');
      assert.ok(fieldIds.includes('email'), 'contains email');
      assert.ok(fieldIds.includes('resume'), 'contains resume file input');

      assert.strictEqual(scan.form.submit?.found, true);
    });

    it('disallows programmatic submission of observed method=get form to prevent PII exposure', () => {
      const doc = parseHtml(formHtml, 'Greenhouse Job');
      const submitResult = greenhouseAdapter.submit(doc);
      assert.strictEqual(submitResult.clicked, false);
      assert.strictEqual(submitResult.manualSubmitRequired, true);
    });

    it('confirms only the captured job identity and evidence-backed confirmation', () => {
      const confDoc = parseHtml(confirmationHtml, 'Confirmation');
      const expectedJobUrl = 'https://boards.greenhouse.io/gitlab/8629019002';
      const confUrl = 'https://job-boards.greenhouse.io/gitlab/jobs/8629019002/confirmation';
      const result = greenhouseAdapter.confirmation(confDoc, confUrl, expectedJobUrl);

      assert.strictEqual(result.confirmed, true);
      assert.match(result.text ?? '', /Thank you for applying to GitLab!/i);

      const otherJobUrl = 'https://job-boards.greenhouse.io/gitlab/jobs/9999999999/confirmation';
      assert.strictEqual(greenhouseAdapter.confirmation(confDoc, otherJobUrl, expectedJobUrl).confirmed, false);
    });
  });

  describe('TopCV Adapter Evidence', () => {
    const topcvDir = path.join(evidenceDir, 'topcv-2026-11-10');
    const jobDetailHtml = fs.readFileSync(path.join(topcvDir, 'job-detail.sanitized.html'), 'utf8');
    const metadata = JSON.parse(fs.readFileSync(path.join(topcvDir, 'job-detail.metadata.json'), 'utf8')) as PageMetadata;

    it('matches TopCV URLs and canonicalizes to stable viec-lam path', () => {
      assert.strictEqual(topcvAdapter.matches(metadata.source_url), true);
      const canonical = topcvAdapter.canonicalize(metadata.source_url);
      assert.ok(canonical.startsWith('https://topcv.vn/viec-lam/'));
    });

    it('extracts only title and company data observed in the captured page', () => {
      const doc = parseHtml(jobDetailHtml, 'TopCV Job');
      const scan = topcvAdapter.scan(doc, metadata.source_url);

      assert.ok(scan.job, 'Job details were found');
      assert.match(scan.job.title ?? '', /Backend Developer/);
      assert.strictEqual(scan.job.companyName, 'CÔNG TY TNHH DỊCH VỤ SỐ DIGILIFE VIỆT NAM');
      assert.strictEqual(scan.job.description, '');
    });

    it('blocks with login_required and unverified form, never claiming success', () => {
      const doc = parseHtml(jobDetailHtml, 'TopCV Job');
      const scan = topcvAdapter.scan(doc, metadata.source_url);

      assert.strictEqual(scan.supported, true);
      assert.strictEqual(scan.site, SITE.TOPCV);
      assert.ok(scan.blocked, 'TopCV is blocked');
      assert.strictEqual(scan.blocked.reason, REASON.LOGIN_REQUIRED);
      assert.strictEqual(scan.form, null);

      assert.throws(() => topcvAdapter.fill(doc, {}, null), /unverified and login-gated/);
      assert.throws(() => topcvAdapter.submit(doc), /unverified and login-gated/);
    });
  });

  describe('LinkedIn Adapter Evidence', () => {
    const linkedinDir = path.join(evidenceDir, 'linkedin-2026-11-10');
    const observations = JSON.parse(fs.readFileSync(path.join(linkedinDir, 'observations.json'), 'utf8')) as Record<string, unknown>;
    const url = 'https://www.linkedin.com/jobs/view/4300000000/';

    it('matches LinkedIn job view URLs and canonicalizes', () => {
      assert.ok(observations);
      assert.strictEqual(linkedinAdapter.matches(url), true);
      assert.strictEqual(linkedinAdapter.canonicalize(url), 'https://linkedin.com/jobs/view/4300000000');
    });

    it('blocks safely with unsupported_form/login_required per 429 evidence', () => {
      const doc = parseHtml('<html><body><h1>LinkedIn</h1></body></html>', 'LinkedIn 429');
      const scan = linkedinAdapter.scan(doc, url);

      assert.strictEqual(scan.supported, true);
      assert.strictEqual(scan.site, SITE.LINKEDIN);
      assert.ok(scan.blocked, 'LinkedIn is blocked');
      assert.strictEqual(scan.blocked.reason, REASON.UNSUPPORTED_FORM);
      assert.strictEqual(scan.form, null);

      assert.throws(() => linkedinAdapter.fill(doc, {}, null), /unverified/);
      assert.throws(() => linkedinAdapter.submit(doc), /unverified/);
    });
  });

  describe('Adapter Registry', () => {
    it('resolves correct adapter for URLs and null for unsupported sites', () => {
      assert.strictEqual(getAdapterForUrl('https://boards.greenhouse.io/company/jobs/123'), greenhouseAdapter);
      assert.strictEqual(getAdapterForUrl('https://www.topcv.vn/viec-lam/test/123.html'), topcvAdapter);
      assert.strictEqual(getAdapterForUrl('https://www.linkedin.com/jobs/view/999/'), linkedinAdapter);
      assert.strictEqual(getAdapterForUrl('https://example.com/careers/software-engineer'), null);
    });
  });
});
