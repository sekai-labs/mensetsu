import { MESSAGE, OUTCOME } from '../shared/constants.js';
import {
  containsHostPermission,
  executeScript,
  getExtensionApi,
  isPromiseExtensionApi,
  sendTabMessage as browserSendTabMessage
} from '../shared/browser-api.js';
import { checkApiConnection } from '../features/connection/connection.js';
import { configureSessionStorage, createExtensionStore } from '../services/storage.js';
import { createApi } from '../services/api.js';
import {
  assertLinkedInDetectionAllowed,
  connectApiOrigin,
  evaluateStoredConnection
} from '../features/connection/onboarding.js';
import { createWorkflow } from '../features/workflows/workflow.js';
import { sleep } from '../shared/util.js';
import { confirmEmailApplication, confirmWebApplication } from '../features/applications/application-handoff.js';
import type {
  ApiClient,
  AuthStatus,
  BackgroundState,
  CandidateProfileDto,
  ConnectionStatus,
  ExtensionMessage,
  ExtensionResponse,
  ExtensionStore,
  FillReport,
  ResumePayload,
  Scan,
  Settings,
} from '../shared/types.js';
import type { SubmitCompletionResult, Workflow } from '../features/workflows/workflow.js';
import type { ExtensionMessageSender } from '../shared/webextension.js';

export interface Services {
  store: ExtensionStore;
  api: ApiClient;
  workflow: Workflow;
}

let services: Services | null = null;

export function getServices(): Services {
  if (!services) {
    const store = createExtensionStore();
    const api = createApi({ store });
    const workflow = createWorkflow({ api, store });
    services = { store, api, workflow };
  }
  return services;
}

export async function initBackground(): Promise<void> {
  await configureSessionStorage();
  getServices();
}

async function ensureContentScript(tabId: number): Promise<void> {
  try {
    const response = await browserSendTabMessage<ExtensionResponse<{ pong?: boolean }>>(tabId, { type: MESSAGE.PING });
    if (response?.ok && response?.data?.pong) return;
  } catch {}

  try {
    await executeScript({
      target: { tabId },
      files: ['content.js']
    });
    await sleep(100);
  } catch {
    return;
  }
}

async function sendTabMessage<T = unknown>(tabId: number, message: ExtensionMessage): Promise<T> {
  await ensureContentScript(tabId);
  const response = await browserSendTabMessage<ExtensionResponse>(tabId, message);
  if (!response?.ok) {
    throw new Error(response?.error || 'Content script operation failed.');
  }
  return response.data as T;
}

export async function loadResumePayload(
  api: Pick<ApiClient, 'downloadResume'>,
  resumeUploadId: string
): Promise<ResumePayload> {
  const download = await api.downloadResume(resumeUploadId);
  if (!download || !download.bytes || download.bytes.length === 0) {
    throw new Error('The resume download returned no bytes; refusing to fill without the selected resume.');
  }
  return {
    bytes: Array.from(download.bytes),
    filename: download.filename || 'resume.pdf',
    mime: download.contentType || 'application/pdf'
  };
}

export async function handleBackgroundMessage(
  message: ExtensionMessage,
  _sender: ExtensionMessageSender = {}
): Promise<unknown> {
  const { store, api, workflow } = getServices();
  const type = message?.type;

  switch (type) {
    case MESSAGE.GET_STATE: {
      const settings: Settings = await store.getSettings();
      const connection: ConnectionStatus = await evaluateStoredConnection(settings);
      let auth: AuthStatus = { authenticated: false };
      let candidateProfile: CandidateProfileDto | null = null;

      if (connection.state === 'healthy') {
        try {
          auth = await api.authStatus();
          if (auth.authenticated) {
            try {
              candidateProfile = await api.candidateProfile();
            } catch {
              candidateProfile = null;
            }
          }
        } catch {
          auth = { authenticated: false };
        }
      }

      const tabId = message.tabId;
      const state = tabId ? await workflow.getState(tabId) : { review: null, pending: null };
      const result: BackgroundState = {
        settings,
        connection,
        auth,
        profile: candidateProfile,
        candidateProfile,
        review: state.review,
        pending: state.pending
      };
      return result;
    }

    case MESSAGE.CONNECT: {
      const settings = await store.getSettings();
      return connectApiOrigin({
        rawOrigin: message.origin,
        allowInsecureLoopback: message.allowInsecureLoopback === true,
        currentSettings: settings,
        store,
        checkConnection: (origin: string) => checkApiConnection(origin, { timeoutMs: message.timeoutMs }),
        permissionGranted: (origin: string) => containsHostPermission(origin)
      });
    }

    case MESSAGE.SET_LINKEDIN_OPT_IN: {
      const current = await store.getSettings();
      const updated: Settings = { ...current, linkedinOptIn: Boolean(message.optIn) };
      await store.setSettings(updated);
      return { linkedinOptIn: updated.linkedinOptIn };
    }

    case MESSAGE.LOGIN:
      return api.login(String(message.email || ''), String(message.password || ''));

    case MESSAGE.LOGIN_2FA:
      return api.login2fa(String(message.code || ''));

    case MESSAGE.LOGOUT:
      return api.logout();

    case MESSAGE.LIST_RESUMES:
      return api.listResumes();

    case MESSAGE.LIST_SAVED_JOBS:
      return api.listSavedJobs(
        typeof message.cursor === 'string' ? message.cursor : undefined,
        typeof message.limit === 'number' ? message.limit : undefined
      );

    case MESSAGE.APPROVE_SAVED_JOB:
      return api.approveSavedJob(String(message.savedJobId || ''));

    case MESSAGE.GET_APPLICATION:
      return api.getApplication(String(message.applicationId || ''));

    case MESSAGE.SAVE_JOB: {
      const job = message.job;
      if (!job) throw new Error('A discovered job is required to save it for approval.');
      return api.saveJob(job);
    }

    case MESSAGE.DISMISS_SAVED_JOB:
      return api.dismissSavedJob(String(message.savedJobId || ''));

    case MESSAGE.PREPARE_SAVED_JOB: {
      const resumeUploadId = String(message.resumeUploadId || '');
      const applicationMethod = message.applicationMethod === 'email' ? 'email' : 'web';
      return api.prepareSavedJob(String(message.savedJobId || ''), {
        resume_upload_id: resumeUploadId,
        application_method: applicationMethod
      });
    }

    case MESSAGE.CONFIRM_EMAIL_SENT:
      return confirmEmailApplication(api, String(message.applicationId || ''));

    case MESSAGE.CONFIRM_WEB_SUBMITTED:
      return confirmWebApplication(api, String(message.applicationId || ''));

    case MESSAGE.SCAN: {
      const tabId = message.tabId;
      if (!tabId) throw new Error('tabId is required for scan');
      assertLinkedInDetectionAllowed(message.url, await store.getSettings());
      return sendTabMessage(tabId, { type: MESSAGE.SCAN });
    }

    case MESSAGE.DISCOVER: {
      const tabId = message.tabId;
      if (!tabId) throw new Error('tabId is required for discovery');
      assertLinkedInDetectionAllowed(message.url, await store.getSettings());
      return sendTabMessage(tabId, { type: MESSAGE.DISCOVER });
    }

    case MESSAGE.PREPARE: {
      const tabId = message.tabId;
      if (!tabId) throw new Error('tabId is required for prepare');
      return workflow.prepare(tabId, message.scan as Scan, message.resumeUploadId);
    }

    case MESSAGE.APPROVE: {
      const tabId = message.tabId;
      if (!tabId) throw new Error('tabId is required for approve');
      return workflow.approve(tabId, message.scan as Scan);
    }

    case MESSAGE.BEGIN_FILL: {
      const tabId = message.tabId;
      if (!tabId) throw new Error('tabId is required for fill');
      const fillSpec = await workflow.beginFill(tabId, message.scan as Scan);
      let resumePayload: ResumePayload | null = null;

      if (fillSpec.resumeUploadId) {
        resumePayload = await loadResumePayload(api, fillSpec.resumeUploadId);
      }

      const fillReport = await sendTabMessage<FillReport>(tabId, {
        type: MESSAGE.FILL,
        preparedFields: fillSpec.fields,
        resume: resumePayload
      });

      return workflow.recordFill(tabId, message.scan as Scan, fillReport);
    }

    case MESSAGE.BEGIN_SUBMIT: {
      const tabId = message.tabId;
      if (!tabId) throw new Error('tabId is required for submit');
      const scan = message.scan as Scan;
      const startResult = await workflow.beginSubmit(tabId, scan, message.valuesFingerprint);
      if (startResult.alreadySubmitted) {
        return { alreadySubmitted: true, review: startResult.review };
      }

      const pollTimeoutMs = 15_000;
      const pollIntervalMs = 500;
      const started = Date.now();
      let confirmedEvidence: { confirmed?: boolean; url?: string } | null = null;

      while (Date.now() - started < pollTimeoutMs) {
        await sleep(pollIntervalMs);
        try {
          const conf = await sendTabMessage<{ confirmed?: boolean; url?: string }>(tabId, {
            type: MESSAGE.CHECK_CONFIRMATION,
            expectedJobUrl: startResult.review?.canonicalUrl || scan.canonicalUrl
          });
          if (conf?.confirmed) {
            confirmedEvidence = conf;
            break;
          }
        } catch {}
      }

      if (confirmedEvidence?.confirmed) {
        return workflow.completeSubmit(tabId, {
          outcome: OUTCOME.CONFIRMED,
          confirmed: true,
          confirmationUrl: confirmedEvidence.url || null
        });
      }

      const uncertain: SubmitCompletionResult = await workflow.completeSubmit(tabId, {
        outcome: OUTCOME.UNCERTAIN,
        confirmed: false,
        confirmationUrl: null
      });
      return uncertain;
    }

    case MESSAGE.RECONCILE: {
      const tabId = message.tabId;
      if (!tabId) throw new Error('tabId is required for reconcile');
      return workflow.reconcile(tabId, { submitted: Boolean(message.submitted) });
    }

    case MESSAGE.BLOCK: {
      const tabId = message.tabId;
      if (!tabId) throw new Error('tabId is required for block');
      return workflow.block(tabId, String(message.reasonCode || ''));
    }

    default:
      throw new Error(`Unknown background message: ${type}`);
  }
}

const extensionApi = getExtensionApi();

if (extensionApi?.runtime?.onMessage) {
  extensionApi.runtime.onInstalled?.addListener(() => {
    initBackground().catch(() => undefined);
  });

  extensionApi.runtime.onStartup?.addListener(() => {
    initBackground().catch(() => undefined);
  });

  if (isPromiseExtensionApi(extensionApi)) {
    extensionApi.runtime.onMessage.addListener((message, sender) =>
      handleBackgroundMessage(message as ExtensionMessage, sender)
        .then((data) => ({ ok: true, data }))
        .catch((err) => ({ ok: false, error: String((err as Error)?.message || err) }))
    );
  } else {
    extensionApi.runtime.onMessage.addListener((message, sender, sendResponse) => {
      handleBackgroundMessage(message as ExtensionMessage, sender)
        .then((data) => sendResponse({ ok: true, data }))
        .catch((err) => sendResponse({ ok: false, error: String((err as Error)?.message || err) }));
      return true;
    });
  }

  initBackground().catch(() => undefined);
}
