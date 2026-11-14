import { test } from 'node:test';
import assert from 'node:assert/strict';

import { confirmEmailApplication, confirmWebApplication } from '../src/features/applications/application-handoff.js';
import type { ApplicationHandoffApi } from '../src/features/applications/application-handoff.js';
import { OUTCOME } from '../src/shared/constants.js';
import type { ApplicationRecord } from '../src/shared/types.js';
import type {
  ApproveApplicationRequestDto,
  ReconcileApplicationRequestDto,
  ResultApplicationRequestDto
} from '../src/shared/contracts.js';

interface StatefulApi extends ApplicationHandoffApi {
  calls: unknown[][];
}

function createStatefulApi(
  seed: ApplicationRecord,
  { uncertainResultStatus = 'unknown' }: { uncertainResultStatus?: ApplicationRecord['status'] } = {}
): StatefulApi {
  const application = structuredClone(seed);
  const calls: unknown[][] = [];
  return {
    calls,
    async getApplication(id: string) {
      calls.push(['get', id]);
      return { application: { ...application } };
    },
    async approveApplication(id: string, payload: ApproveApplicationRequestDto) {
      calls.push(['approve', id, payload]);
      if (application.status !== 'prepared' || payload.revision !== application.revision) {
        throw new Error('invalid approval transition');
      }
      application.status = 'approved';
      application.revision = String(Number(application.revision) + 1);
      return { data: { ...application } };
    },
    async startApplication(id: string) {
      calls.push(['start', id]);
      if (application.status !== 'approved') throw new Error('invalid start transition');
      application.status = 'submitting';
      return { ...application };
    },
    async resultApplication(id: string, payload: ResultApplicationRequestDto) {
      calls.push(['result', id, payload]);
      if (application.status !== 'submitting') throw new Error('invalid result transition');
      application.status = payload.outcome === OUTCOME.CONFIRMED ? 'submitted' : uncertainResultStatus;
      if (application.status === 'submitted') application.workspace_id = 'workspace:1';
      return { data: { ...application } };
    },
    async reconcileApplication(id: string, payload: ReconcileApplicationRequestDto) {
      calls.push(['reconcile', id, payload]);
      if (!['unknown', 'needs_review'].includes(String(application.status))) {
        throw new Error('invalid reconciliation transition');
      }
      if (payload.submitted) {
        application.status = 'submitted';
        application.workspace_id = 'workspace:1';
      }
      return { application: { ...application } };
    }
  };
}

function application(overrides: ApplicationRecord = {}): ApplicationRecord {
  return {
    id: 'application:1',
    revision: '4',
    status: 'prepared',
    application_method: 'email',
    recipient_email: 'hiring@example.com',
    ...overrides
  };
}

test('email confirmation records a manually sent email without inventing a confirmation URL', async () => {
  const api = createStatefulApi(application());

  const result = await confirmEmailApplication(api, 'application:1');

  assert.equal(result.status, 'submitted');
  assert.equal(result.workspace_id, 'workspace:1');
  assert.deepEqual(api.calls, [
    ['get', 'application:1'],
    ['approve', 'application:1', { revision: '4' }],
    ['start', 'application:1'],
    ['result', 'application:1', { outcome: OUTCOME.CONFIRMED }]
  ]);
});

test('website confirmation reconciles an uncertain result before interview handoff', async () => {
  const api = createStatefulApi(application({ application_method: 'web', recipient_email: null }));

  const result = await confirmWebApplication(api, 'application:1');

  assert.equal(result.status, 'submitted');
  assert.equal(result.workspace_id, 'workspace:1');
  assert.deepEqual(api.calls, [
    ['get', 'application:1'],
    ['approve', 'application:1', { revision: '4' }],
    ['start', 'application:1'],
    ['result', 'application:1', { outcome: OUTCOME.UNCERTAIN }],
    ['reconcile', 'application:1', { submitted: true }]
  ]);
});

test('email confirmation retries from unknown state through reconciliation', async () => {
  const api = createStatefulApi(application({ status: 'unknown' }));

  const result = await confirmEmailApplication(api, 'application:1');

  assert.equal(result.status, 'submitted');
  assert.deepEqual(api.calls, [
    ['get', 'application:1'],
    ['reconcile', 'application:1', { submitted: true }]
  ]);
});

test('already submitted applications are idempotent and wrong channels are rejected', async () => {
  const submittedApi = createStatefulApi(application({ status: 'submitted', workspace_id: 'workspace:1' }));
  const submitted = await confirmEmailApplication(submittedApi, 'application:1');
  assert.equal(submitted.status, 'submitted');
  assert.deepEqual(submittedApi.calls, [['get', 'application:1']]);

  const wrongChannelApi = createStatefulApi(application({ application_method: 'web' }));
  await assert.rejects(() => confirmEmailApplication(wrongChannelApi, 'application:1'), /not prepared for email/);
  assert.deepEqual(wrongChannelApi.calls, [['get', 'application:1']]);

  const missingRecipientApi = createStatefulApi(application({ recipient_email: null }));
  await assert.rejects(() => confirmEmailApplication(missingRecipientApi, 'application:1'), /not prepared for email/);
  assert.deepEqual(missingRecipientApi.calls, [['get', 'application:1']]);
});

test('website confirmation does not claim success when uncertain outcome is not reconcilable', async () => {
  const api = createStatefulApi(
    application({ application_method: 'web', recipient_email: null }),
    { uncertainResultStatus: 'submitting' }
  );

  await assert.rejects(() => confirmWebApplication(api, 'application:1'), /manual reconciliation/);
  assert.deepEqual(api.calls.at(-1), ['result', 'application:1', { outcome: OUTCOME.UNCERTAIN }]);
});
