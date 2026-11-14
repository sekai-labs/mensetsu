import { describe, it } from 'node:test';
import assert from 'node:assert/strict';

import { createWorkflow, submitOutcomeNotice, WorkflowError } from '../src/features/workflows/workflow.js';
import type { Workflow } from '../src/features/workflows/workflow.js';
import { createMemoryStore } from '../src/services/storage.js';
import { OUTCOME, REASON } from '../src/shared/constants.js';
import { computeStructureFingerprint } from '../src/features/applications/review.js';
import type { ApiClient, PrepareApplicationBody, Scan } from '../src/shared/types.js';
import type {
  ApproveApplicationRequestDto,
  BlockApplicationRequestDto,
  ReconcileApplicationRequestDto,
  ResultApplicationRequestDto
} from '../src/shared/contracts.js';

interface MockApiOverrides {
  prepareResult?: Record<string, unknown>;
  approveResult?: Record<string, unknown>;
  startResult?: Record<string, unknown>;
  resultResult?: Record<string, unknown>;
  reconcileResult?: Record<string, unknown>;
  startError?: Error;
  prepareApplication?: (payload: PrepareApplicationBody) => Promise<unknown>;
  approveApplication?: (id: string, payload: ApproveApplicationRequestDto) => Promise<unknown>;
  startApplication?: (id: string) => Promise<unknown>;
  resultApplication?: (id: string, payload: ResultApplicationRequestDto) => Promise<unknown>;
  reconcileApplication?: (id: string, payload: ReconcileApplicationRequestDto) => Promise<unknown>;
  blockApplication?: (id: string, payload: BlockApplicationRequestDto) => Promise<unknown>;
}

function createMockApi(overrides: MockApiOverrides = {}): ApiClient {
  const {
    prepareResult,
    approveResult,
    startResult,
    resultResult,
    reconcileResult,
    startError,
    ...methods
  } = overrides;

  const base = {
    async prepareApplication(payload: PrepareApplicationBody): Promise<unknown> {
      return {
        id: 'app_123',
        site: payload.site,
        canonical_url: 'https://boards.greenhouse.io/gitlab/8629019002',
        source_url: payload.source_url,
        company_name: payload.company_name,
        role_title: payload.role_title,
        job_description: payload.job_description,
        resume_upload_id: payload.resume_upload_id,
        prepared_fields: { first_name: 'Jane', last_name: 'Doe', email: 'jane@example.com' },
        revision: 'rev_abc123',
        status: 'prepared',
        ...(prepareResult || {})
      };
    },
    async approveApplication(id: string, payload: ApproveApplicationRequestDto): Promise<unknown> {
      return {
        id,
        revision: payload.revision,
        status: 'approved',
        ...(approveResult || {})
      };
    },
    async startApplication(id: string): Promise<unknown> {
      if (startError) throw startError;
      return {
        id,
        status: 'submitting',
        ...(startResult || {})
      };
    },
    async resultApplication(id: string, payload: ResultApplicationRequestDto): Promise<unknown> {
      return {
        id,
        status: payload.outcome === OUTCOME.CONFIRMED ? 'submitted' : 'unknown',
        confirmation_url: payload.confirmation_url,
        ...(resultResult || {})
      };
    },
    async reconcileApplication(id: string, payload: ReconcileApplicationRequestDto): Promise<unknown> {
      return {
        id,
        status: payload.submitted ? 'submitted' : 'needs_review',
        ...(reconcileResult || {})
      };
    },
    async blockApplication(id: string, payload: BlockApplicationRequestDto): Promise<unknown> {
      return {
        id,
        status: 'needs_review',
        reason_code: payload.reason_code
      };
    }
  };

  return { ...base, ...methods } as unknown as ApiClient;
}

async function mockScan(overrides: Partial<Scan> = {}): Promise<Scan> {
  const scan: Scan = {
    supported: true,
    site: 'greenhouse',
    canonicalUrl: 'https://boards.greenhouse.io/gitlab/8629019002',
    sourceUrl: 'https://boards.greenhouse.io/gitlab/8629019002',
    job: {
      companyName: 'GitLab',
      title: 'Director, Support',
      description: 'Role overview'
    },
    form: {
      selector: 'form#application-form',
      fields: [{ id: 'first_name', name: 'first_name', type: 'text' }]
    }
  };
  Object.assign(scan, overrides);
  scan.fingerprint = await computeStructureFingerprint(scan);
  return scan;
}

function requireReview(workflowResult: { review: { phase: string; status?: string; revision?: string; submitUsed?: boolean } | null }) {
  assert.ok(workflowResult.review, 'workflow returned a review');
  return workflowResult.review;
}

describe('Workflow Transitions and Single-Use State', () => {
  it('executes linear lifecycle: prepare -> approve -> fill -> start -> submit', async () => {
    const store = createMemoryStore();
    const api = createMockApi();
    const workflow: Workflow = createWorkflow({ api, store });

    const scan = await mockScan();
    const tabId = 101;

    const prep = await workflow.prepare(tabId, scan, 'upload_uuid_1');
    assert.strictEqual(requireReview(prep).phase, 'prepared');
    assert.strictEqual(requireReview(prep).revision, 'rev_abc123');

    const app = await workflow.approve(tabId, scan);
    assert.strictEqual(requireReview(app).phase, 'approved');

    const fillSpec = await workflow.beginFill(tabId, scan);
    assert.strictEqual(fillSpec.fields.first_name, 'Jane');

    const valuesFingerprint = 'b'.repeat(64);
    const recFill = await workflow.recordFill(tabId, scan, {
      valuesFingerprint,
      resumeRequired: false
    });
    assert.strictEqual(requireReview(recFill).phase, 'filled');

    const beginSub = await workflow.beginSubmit(tabId, scan, valuesFingerprint);
    assert.strictEqual(requireReview(beginSub).phase, 'submitting');
    assert.strictEqual(requireReview(beginSub).submitUsed, true);

    const comp = await workflow.completeSubmit(tabId, {
      confirmed: true,
      confirmationUrl: 'https://boards.greenhouse.io/gitlab/8629019002/confirmation'
    });
    assert.strictEqual(requireReview(comp).phase, 'submitted');
    assert.strictEqual(requireReview(comp).status, 'submitted');
  });

  it('rejects duplicate submit attempts (single-use enforcement)', async () => {
    const store = createMemoryStore();
    const api = createMockApi();
    const workflow = createWorkflow({ api, store });

    const scan = await mockScan();
    const tabId = 102;
    const valuesFingerprint = 'c'.repeat(64);

    await workflow.prepare(tabId, scan, 'upload_uuid_1');
    await workflow.approve(tabId, scan);
    await workflow.recordFill(tabId, scan, { valuesFingerprint });
    await workflow.beginSubmit(tabId, scan, valuesFingerprint);

    await assert.rejects(
      () => workflow.beginSubmit(tabId, scan, valuesFingerprint),
      (err: unknown) => err instanceof WorkflowError && (err.code === REASON.ALREADY_SUBMITTED || err.code === REASON.SUBMIT_IN_PROGRESS)
    );
  });

  it('rejects stale state: expired review (> 10 minutes)', async () => {
    const store = createMemoryStore();
    const api = createMockApi();
    let clock = 1_000_000;
    const workflow = createWorkflow({ api, store, now: () => clock, ttlMs: 600_000 });

    const scan = await mockScan();
    const tabId = 103;

    await workflow.prepare(tabId, scan, 'upload_uuid_1');
    clock += 600_001;

    await assert.rejects(
      () => workflow.approve(tabId, scan),
      (err: unknown) => err instanceof WorkflowError && err.code === REASON.REVIEW_EXPIRED
    );
  });

  it('rejects altered form fingerprint or values fingerprint', async () => {
    const store = createMemoryStore();
    const api = createMockApi();
    const workflow = createWorkflow({ api, store });

    const scan = await mockScan();
    const tabId = 104;

    await workflow.prepare(tabId, scan, 'upload_uuid_1');
    await workflow.approve(tabId, scan);

    const valuesFingerprint = 'd'.repeat(64);
    await workflow.recordFill(tabId, scan, { valuesFingerprint });

    const tamperedValues = 'e'.repeat(64);
    await assert.rejects(
      () => workflow.beginSubmit(tabId, scan, tamperedValues),
      (err: unknown) => err instanceof WorkflowError && err.code === REASON.VALUES_CHANGED
    );

    const tamperedScan = { ...scan, fingerprint: 'f'.repeat(64) };
    await assert.rejects(
      () => workflow.beginSubmit(tabId, tamperedScan, valuesFingerprint),
      (err: unknown) => err instanceof WorkflowError && err.code === REASON.FORM_CHANGED
    );
  });

  it('handles uncertain confirmation and manual reconciliation flow', async () => {
    const store = createMemoryStore();
    const api = createMockApi();
    const workflow = createWorkflow({ api, store });

    const scan = await mockScan();
    const tabId = 105;
    const valuesFingerprint = '1'.repeat(64);

    await workflow.prepare(tabId, scan, 'upload_uuid_1');
    await workflow.approve(tabId, scan);
    await workflow.recordFill(tabId, scan, { valuesFingerprint });
    await workflow.beginSubmit(tabId, scan, valuesFingerprint);

    const unconfirmed = await workflow.completeSubmit(tabId, { confirmed: false });
    assert.strictEqual(requireReview(unconfirmed).phase, 'unknown');
    assert.strictEqual(requireReview(unconfirmed).status, 'unknown');

    const recYes = await workflow.reconcile(tabId, { submitted: true });
    assert.strictEqual(requireReview(recYes).status, 'submitted');

    const recNo = await workflow.reconcile(tabId, { submitted: false });
    assert.strictEqual(requireReview(recNo).status, 'needs_review');
  });

  it('does not fabricate company, role, or job description when source evidence is missing', async () => {
    const store = createMemoryStore();
    let prepareCalled = false;
    const api = createMockApi({
      async prepareApplication() {
        prepareCalled = true;
        return {};
      }
    });
    const workflow = createWorkflow({ api, store });
    const scan = await mockScan({
      job: { companyName: '', title: '', description: '' }
    });

    await assert.rejects(
      () => workflow.prepare(106, scan, 'upload_uuid_1'),
      (error: unknown) => error instanceof WorkflowError && error.code === REASON.JOB_DETAILS_MISSING
    );
    assert.strictEqual(prepareCalled, false);
  });

  it('asks the operator to reconcile whenever a submit is not verified', async () => {
    const needsReview = submitOutcomeNotice({
      applicationId: 'app_1',
      site: 'greenhouse',
      canonicalUrl: 'https://boards.greenhouse.io/gitlab/8629019002',
      companyName: 'GitLab',
      roleTitle: 'Director, Support',
      phase: 'needs_review',
      status: 'needs_review',
      revision: 'rev_1',
      preparedFields: {},
      resumeUploadId: 'upload_1',
      preparedAt: 0,
      approvedAt: null,
      filledAt: null,
      filledFingerprint: null,
      submitUsed: true,
      confirmationUrl: null,
      reasonCode: REASON.CONFIRMATION_NOT_FOUND
    });
    assert.strictEqual(needsReview?.level, 'error');
    assert.match(needsReview?.message || '', /confirmation_not_found/);
    assert.match(needsReview?.message || '', /Choose Yes/);

    const submitted = submitOutcomeNotice({
      applicationId: 'app_1',
      site: 'greenhouse',
      canonicalUrl: 'https://boards.greenhouse.io/gitlab/8629019002',
      companyName: 'GitLab',
      roleTitle: 'Director, Support',
      phase: 'submitted',
      status: 'submitted',
      revision: 'rev_1',
      preparedFields: {},
      resumeUploadId: 'upload_1',
      preparedAt: 0,
      approvedAt: null,
      filledAt: null,
      filledFingerprint: null,
      submitUsed: true,
      confirmationUrl: 'https://boards.greenhouse.io/gitlab/8629019002/confirmation',
      reasonCode: null
    });
    assert.strictEqual(submitted?.level, 'info');
    assert.strictEqual(submitOutcomeNotice(null), null);
  });
});
