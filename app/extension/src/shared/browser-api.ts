import { originPattern } from './api-origin.js';
import type {
  ExtensionApi,
  ExtensionLastError,
  ExtensionScriptDetails,
  ExtensionTab
} from './webextension.js';

type ExtensionFunction = (...args: unknown[]) => unknown;

export function getExtensionApi(): ExtensionApi | null {
  if (typeof globalThis.browser !== 'undefined' && globalThis.browser?.runtime) {
    return globalThis.browser;
  }
  if (typeof globalThis.chrome !== 'undefined' && globalThis.chrome?.runtime) {
    return globalThis.chrome;
  }
  return null;
}

export function isPromiseExtensionApi(api: ExtensionApi | null = getExtensionApi()): boolean {
  return Boolean(api) && typeof globalThis.browser !== 'undefined' && api === globalThis.browser;
}

function lastError(): ExtensionLastError | null {
  return getExtensionApi()?.runtime?.lastError ?? null;
}

export function callApi<T = unknown>(
  fn: ExtensionFunction | undefined,
  thisArg: unknown,
  args: unknown[] = []
): Promise<T> {
  if (typeof fn !== 'function') {
    return Promise.reject(new Error('This browser does not expose the required extension API.'));
  }

  const api = getExtensionApi();
  if (isPromiseExtensionApi(api)) {
    try {
      return Promise.resolve(fn.apply(thisArg, args) as T);
    } catch (error) {
      return Promise.reject(error);
    }
  }

  return new Promise<T>((resolve, reject) => {
    const callback = (value: unknown): void => {
      const error = lastError();
      if (error) reject(new Error(error.message || 'Extension API request failed.'));
      else resolve(value as T);
    };

    let returned: unknown;
    try {
      returned = fn.apply(thisArg, [...args, callback]);
    } catch (error) {
      reject(error);
      return;
    }

    if (returned && typeof (returned as PromiseLike<unknown>).then === 'function') {
      Promise.resolve(returned as PromiseLike<T>).then(resolve, reject);
    }
  });
}

export function sendRuntimeMessage<T = unknown>(message: unknown): Promise<T> {
  const api = getExtensionApi();
  return callApi<T>(api?.runtime?.sendMessage as ExtensionFunction | undefined, api?.runtime, [message]);
}

export function sendTabMessage<T = unknown>(tabId: number, message: unknown): Promise<T> {
  const api = getExtensionApi();
  return callApi<T>(api?.tabs?.sendMessage as ExtensionFunction | undefined, api?.tabs, [tabId, message]);
}

export function queryActiveTab(): Promise<ExtensionTab[]> {
  const api = getExtensionApi();
  return callApi<ExtensionTab[]>(api?.tabs?.query as ExtensionFunction | undefined, api?.tabs, [{ active: true, currentWindow: true }]);
}

export function executeScript(details: ExtensionScriptDetails): Promise<unknown[]> {
  const api = getExtensionApi();
  return callApi<unknown[]>(api?.scripting?.executeScript as ExtensionFunction | undefined, api?.scripting, [details]);
}

export function requestHostPermission(origin: string): Promise<boolean> {
  const api = getExtensionApi();
  const origins = [originPattern(origin)];
  return callApi<boolean>(api?.permissions?.request as ExtensionFunction | undefined, api?.permissions, [{ origins }]);
}

export function containsHostPermission(origin: string): Promise<boolean> {
  const api = getExtensionApi();
  const origins = [originPattern(origin)];
  return callApi<boolean>(api?.permissions?.contains as ExtensionFunction | undefined, api?.permissions, [{ origins }]).then(
    (granted) => granted === true
  );
}
