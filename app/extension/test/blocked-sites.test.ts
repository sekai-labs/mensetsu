import { describe, it } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';

import { createWorkflow, WorkflowError } from '../src/features/workflows/workflow.js';
import { createMemoryStore } from '../src/services/storage.js';
import { topcvAdapter } from '../src/content/adapters/topcv.js';
import { linkedinAdapter } from '../src/content/adapters/linkedin.js';
import { parseHtml } from './fixtures/dom-helper.js';
import { REASON, SERVER_BLOCK_REASON, toServerBlockReason } from '../src/shared/constants.js';
import type { ApiClient } from '../src/shared/types.js';

const topcvFixture = fs.readFileSync(new URL('../evidence/topcv-2026-11-10/job-detail.sanitized.html', import.meta.url), 'utf8');
const topcvMetadata = JSON.parse(fs.readFileSync(new URL('../evidence/topcv-2026-11-10/job-detail.metadata.json', import.meta.url), 'utf8')) as { source_url: string };

function partialApi(overrides: Partial<ApiClient>): ApiClient {
  return overrides as ApiClient;
}

describe('Blocked Sites Safety (TopCV and LinkedIn)', () => {
  it('maps block reasons to the API enum variants', () => {
    assert.strictEqual(toServerBlockReason(REASON.LOGIN_REQUIRED), SERVER_BLOCK_REASON.LOGIN_REQUIRED);
    assert.strictEqual(toServerBlockReason(REASON.CAPTCHA_REQUIRED), SERVER_BLOCK_REASON.CAPTCHA);
    assert.strictEqual(toServerBlockReason(REASON.FORM_CHANGED), SERVER_BLOCK_REASON.FORM_CHANGED);
    assert.strictEqual(toServerBlockReason(REASON.VALUES_CHANGED), SERVER_BLOCK_REASON.FORM_CHANGED);
    assert.strictEqual(toServerBlockReason(REASON.JOB_CHANGED), SERVER_BLOCK_REASON.FORM_CHANGED);
    assert.strictEqual(toServerBlockReason(REASON.PROFILE_INCOMPLETE), SERVER_BLOCK_REASON.MISSING_REQUIRED_FIELD);
    assert.strictEqual(toServerBlockReason(REASON.RESUME_MISSING), SERVER_BLOCK_REASON.MISSING_REQUIRED_FIELD);
    assert.strictEqual(toServerBlockReason(REASON.UNSUPPORTED_SITE), SERVER_BLOCK_REASON.UNSUPPORTED_FORM);
    assert.strictEqual(toServerBlockReason(REASON.UNSUPPORTED_FORM), SERVER_BLOCK_REASON.UNSUPPORTED_FORM);
    assert.strictEqual(toServerBlockReason('unknown_code'), SERVER_BLOCK_REASON.UNSUPPORTED_FORM);
  });

  it('blocks TopCV using captured login-gated evidence without fabricating missing job text', async () => {
    const doc = parseHtml(topcvFixture);
    const scan = topcvAdapter.scan(doc, topcvMetadata.source_url);
    assert.strictEqual(scan.blocked?.reason, REASON.LOGIN_REQUIRED);
    assert.ok(scan.job, 'Job details were found');
    assert.strictEqual(scan.job.companyName, 'CÔNG TY TNHH DỊCH VỤ SỐ DIGILIFE VIỆT NAM');
    assert.match(scan.job.title ?? '', /Backend Developer/);
    assert.strictEqual(scan.job.description, '');
    assert.strictEqual(scan.form, null);

    let prepareCalled = false;
    const workflow = createWorkflow({
      store: createMemoryStore(),
      api: partialApi({
        async prepareApplication() {
          prepareCalled = true;
          throw new Error('must not create an application without an observed job description');
        }
      })
    });
    await assert.rejects(
      () => workflow.prepare(201, scan, 'upload_uuid_topcv'),
      (error: unknown) => error instanceof WorkflowError && error.code === REASON.JOB_DETAILS_MISSING
    );
    assert.strictEqual(prepareCalled, false);
  });

  it('blocks LinkedIn 429 evidence without scraping or fabricating job details', async () => {
    const doc = parseHtml('<html><body><h1>Staff Engineer</h1></body></html>');
    const scan = linkedinAdapter.scan(doc, 'https://www.linkedin.com/jobs/view/4300000000/');
    assert.strictEqual(scan.blocked?.reason, REASON.UNSUPPORTED_FORM);
    assert.ok(scan.job, 'Job details were found');
    assert.strictEqual(scan.job.title, '');
    assert.strictEqual(scan.job.companyName, '');
    assert.strictEqual(scan.job.description, '');
    assert.strictEqual(scan.form, null);

    let prepareCalled = false;
    const workflow = createWorkflow({
      store: createMemoryStore(),
      api: partialApi({
        async prepareApplication() {
          prepareCalled = true;
          throw new Error('must not create an application from an anti-bot response');
        }
      })
    });
    await assert.rejects(
      () => workflow.prepare(202, scan, 'upload_uuid_li'),
      (error: unknown) => error instanceof WorkflowError && error.code === REASON.JOB_DETAILS_MISSING
    );
    assert.strictEqual(prepareCalled, false);
  });
});
