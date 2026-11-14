import { MESSAGE, REASON } from '../shared/constants.js';
import { getAdapterForDiscoveryUrl, getAdapterForUrl } from './adapters/index.js';
import { computeStructureFingerprint, computeValuesFingerprint } from '../features/applications/review.js';
import { getExtensionApi, isPromiseExtensionApi } from '../shared/browser-api.js';
import type { DomDocument } from './adapters/dom-types.js';
import type { ContentMessageResult, DiscoveryResult, ExtensionMessage, ResumeAttachment } from '../shared/types.js';

function isExtensionMessage(message: unknown): message is ExtensionMessage {
  return Boolean(message && typeof message === 'object' && 'type' in message && typeof message.type === 'string');
}

let initialized = false;

function currentPage(): DomDocument {
  return globalThis.document as unknown as DomDocument;
}

function currentUrl(): string {
  return globalThis.window?.location?.href ?? '';
}

export function initContentScript(): void {
  if (initialized) return;
  initialized = true;

  const api = getExtensionApi();
  if (!api?.runtime?.onMessage) return;

  if (isPromiseExtensionApi(api)) {
    api.runtime.onMessage.addListener((message) => {
      if (!isExtensionMessage(message)) {
        return Promise.resolve({ ok: false, error: 'Invalid extension message.' });
      }
      return handleMessage(message)
        .then((data) => ({ ok: true, data }))
        .catch((err) => ({ ok: false, error: String((err as Error)?.message || err) }));
    });
  } else {
    api.runtime.onMessage.addListener((message, _sender, sendResponse) => {
      if (!isExtensionMessage(message)) {
        sendResponse({ ok: false, error: 'Invalid extension message.' });
        return false;
      }
      handleMessage(message)
        .then((res) => sendResponse({ ok: true, data: res }))
        .catch((err) => sendResponse({ ok: false, error: String((err as Error)?.message || err) }));
      return true;
    });
  }
}

export async function handleMessage<T = ContentMessageResult>(message: ExtensionMessage): Promise<T> {
  return await handleContentMessage(message) as T;
}

async function handleContentMessage(message: ExtensionMessage): Promise<unknown> {
  const type = message?.type;
  const page = currentPage();
  const url = currentUrl();

  switch (type) {
    case MESSAGE.PING:
      return { pong: true, url };

    case MESSAGE.DISCOVER: {
      const adapter = getAdapterForDiscoveryUrl(url);
      const discover = adapter?.discover;
      if (!adapter || typeof discover !== 'function') {
        const unsupported: DiscoveryResult = {
          supported: false,
          site: '',
          jobs: [],
          blocked: {
            reason: REASON.UNSUPPORTED_SITE,
            detail: 'This site is not supported for job discovery. Open a LinkedIn or TopCV job listing page.'
          }
        };
        return unsupported;
      }
      return discover.call(adapter, page, url);
    }

    case MESSAGE.SCAN: {
      const adapter = getAdapterForUrl(url);
      if (!adapter) {
        return {
          supported: false,
          canonicalUrl: url,
          sourceUrl: url,
          blocked: {
            reason: REASON.UNSUPPORTED_SITE,
            detail: 'This site is not supported by Mensetsu.'
          }
        };
      }

      const scanResult = adapter.scan(page, url);
      if (scanResult.form) {
        scanResult.fingerprint = await computeStructureFingerprint(scanResult);
      }
      return scanResult;
    }

    case MESSAGE.FILL: {
      const adapter = getAdapterForUrl(url);
      const fill = adapter?.fill;
      if (!adapter || !adapter.capabilities?.fill || typeof fill !== 'function') {
        throw new Error('This site does not support automated form filling.');
      }

      let resume = message.resume ?? null;
      if (resume?.bytes && !(resume.bytes instanceof Uint8Array)) {
        resume = { ...resume, bytes: new Uint8Array(resume.bytes) };
      }

      const fillReport = fill.call(adapter, page, message.preparedFields ?? {}, resume as ResumeAttachment | null);
      const valuesFingerprint = await computeValuesFingerprint(fillReport.fields);
      return {
        ...fillReport,
        valuesFingerprint
      };
    }

    case MESSAGE.SUBMIT: {
      const adapter = getAdapterForUrl(url);
      const submit = adapter?.submit;
      if (!adapter || !adapter.capabilities?.submit || typeof submit !== 'function') {
        if (adapter?.capabilities?.manualSubmitRequired) {
          return {
            clicked: false,
            manualSubmitRequired: true,
            detail: 'Manual submission on site required to prevent PII exposure in GET URL.'
          };
        }
        throw new Error('This site does not support automated submit clicking.');
      }
      return submit.call(adapter, page);
    }

    case MESSAGE.CHECK_CONFIRMATION: {
      const adapter = getAdapterForUrl(url);
      const confirmation = adapter?.confirmation;
      if (!adapter || typeof confirmation !== 'function') {
        return { confirmed: false, url };
      }
      return confirmation.call(adapter, page, url, message.expectedJobUrl);
    }

    default:
      throw new Error(`Unknown message type: ${type}`);
  }
}

if (typeof window !== 'undefined' && getExtensionApi()?.runtime?.onMessage) {
  initContentScript();
}
