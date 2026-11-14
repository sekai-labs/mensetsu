import { OUTCOME } from '../../shared/constants.js';
import type { ApplicationRecord } from '../../shared/types.js';
import type {
  ApproveApplicationRequestDto,
  BlockApplicationRequestDto,
  ReconcileApplicationRequestDto,
  ResultApplicationRequestDto
} from '../../shared/contracts.js';

export interface ApplicationHandoffApi {
  getApplication(applicationId: string): Promise<unknown>;
  approveApplication(applicationId: string, payload: ApproveApplicationRequestDto): Promise<unknown>;
  startApplication(applicationId: string): Promise<unknown>;
  blockApplication?(applicationId: string, payload: BlockApplicationRequestDto): Promise<unknown>;
  resultApplication(applicationId: string, payload: ResultApplicationRequestDto): Promise<unknown>;
  reconcileApplication(applicationId: string, payload: ReconcileApplicationRequestDto): Promise<unknown>;
}

function asRecord(value: unknown): Record<string, unknown> | null {
  return value && typeof value === 'object' ? value as Record<string, unknown> : null;
}

function unwrapApplication(response: unknown): ApplicationRecord {
  const record = asRecord(response);
  const nested = record?.application ?? record?.data ?? record;
  return (asRecord(nested) ?? {}) as ApplicationRecord;
}

async function startSavedApplication(
  api: ApplicationHandoffApi,
  applicationId: string,
  application: ApplicationRecord
): Promise<ApplicationRecord> {
  let current = application;
  if (current.status === 'prepared') {
    current = unwrapApplication(await api.approveApplication(applicationId, { revision: String(current.revision ?? '') }));
  }
  if (current.status === 'approved') {
    current = unwrapApplication(await api.startApplication(applicationId));
  }
  return current;
}

export async function confirmEmailApplication(api: ApplicationHandoffApi, applicationId: string): Promise<ApplicationRecord> {
  let application = unwrapApplication(await api.getApplication(applicationId));
  if (application.application_method !== 'email' || !application.recipient_email) {
    throw new Error('This application is not prepared for email.');
  }
  if (application.status === 'submitted') return application;
  if (application.status === 'unknown' || application.status === 'needs_review') {
    return unwrapApplication(await api.reconcileApplication(applicationId, { submitted: true }));
  }
  application = await startSavedApplication(api, applicationId, application);
  if (application.status !== 'submitting') throw new Error('The application is not ready to record an email submission.');
  return unwrapApplication(await api.resultApplication(applicationId, { outcome: OUTCOME.CONFIRMED }));
}

export async function confirmWebApplication(api: ApplicationHandoffApi, applicationId: string): Promise<ApplicationRecord> {
  let application = unwrapApplication(await api.getApplication(applicationId));
  if (application.application_method !== 'web') {
    throw new Error('This application is not prepared for website submission.');
  }
  if (application.status === 'submitted') return application;
  if (application.status === 'unknown' || application.status === 'needs_review') {
    return unwrapApplication(await api.reconcileApplication(applicationId, { submitted: true }));
  }
  application = await startSavedApplication(api, applicationId, application);
  if (application.status !== 'submitting') throw new Error('The application is not ready to record a website submission.');
  application = unwrapApplication(await api.resultApplication(applicationId, { outcome: OUTCOME.UNCERTAIN }));
  if (!['unknown', 'needs_review'].includes(String(application.status))) {
    throw new Error('The website submission needs manual reconciliation in Mensetsu.');
  }
  return unwrapApplication(await api.reconcileApplication(applicationId, { submitted: true }));
}


