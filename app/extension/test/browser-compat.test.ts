import { describe, it, afterEach } from 'node:test';
import assert from 'node:assert/strict';

import {
  callApi,
  containsHostPermission,
  getExtensionApi,
  requestHostPermission
} from '../src/shared/browser-api.js';
import { configureSessionStorage, createExtensionStore } from '../src/services/storage.js';
import type { ExtensionApi } from '../src/shared/webextension.js';

type Bucket = { local: Record<string, unknown>; session: Record<string, unknown> };

function promiseArea(bucket: Record<string, unknown>) {
  return {
    async get(key: string | string[] | null): Promise<Record<string, unknown>> {
      if (key === null || key === undefined) return { ...bucket };
      if (Array.isArray(key)) {
        return Object.fromEntries(key.map((item) => [item, bucket[item]]));
      }
      return { [key]: bucket[key] };
    },
    async set(values: Record<string, unknown>): Promise<void> {
      Object.assign(bucket, values);
    },
    async remove(key: string | string[]): Promise<void> {
      if (Array.isArray(key)) {
        for (const item of key) delete bucket[item];
      } else {
        delete bucket[key];
      }
    }
  };
}

function callbackArea(bucket: Record<string, unknown>) {
  return {
    get(key: string, callback: (values: Record<string, unknown>) => void): void {
      callback({ [key]: bucket[key] });
    },
    set(values: Record<string, unknown>, callback: () => void): void {
      Object.assign(bucket, values);
      callback();
    },
    remove(key: string, callback: () => void): void {
      delete bucket[key];
      callback();
    }
  };
}

function installFirefox(bucket: Bucket): ExtensionApi {
  const api = {
    runtime: { lastError: null },
    storage: {
      local: promiseArea(bucket.local),
      session: promiseArea(bucket.session)
    },
    permissions: {
      async contains() {
        return true;
      },
      async request() {
        return true;
      }
    }
  } as unknown as ExtensionApi;
  globalThis.browser = api;
  return api;
}

function installChrome(bucket: Bucket, { session = true }: { session?: boolean } = {}): ExtensionApi {
  const api = {
    runtime: { lastError: null },
    storage: {
      local: callbackArea(bucket.local)
    },
    permissions: {
      contains(_details: unknown, callback: (granted: boolean) => void): void {
        callback(true);
      },
      request(_details: unknown, callback: (granted: boolean) => void): void {
        callback(true);
      }
    }
  } as unknown as ExtensionApi;
  if (session) {
    api.storage.session = callbackArea(bucket.session) as unknown as ExtensionApi['storage']['session'];
  }
  globalThis.chrome = api;
  return api;
}

afterEach(() => {
  delete globalThis.browser;
  delete globalThis.chrome;
});

describe('Cross-browser WebExtension API compatibility', () => {
  it('uses the Firefox promise namespace without appending callbacks', async () => {
    const calls: unknown[][] = [];
    const api = installFirefox({ local: {}, session: {} });
    const permissions = api.permissions!;
    permissions.request = (...args: unknown[]) => {
      calls.push(args);
      return Promise.resolve(true);
    };
    permissions.contains = (...args: unknown[]) => {
      calls.push(args);
      return Promise.resolve(true);
    };

    assert.strictEqual(getExtensionApi(), api);
    assert.strictEqual(await requestHostPermission('https://api.example.com'), true);
    assert.strictEqual(await containsHostPermission('https://api.example.com'), true);
    assert.strictEqual(calls.length, 2);
    assert.deepStrictEqual(calls[0], [{ origins: ['https://api.example.com/*'] }]);
    assert.deepStrictEqual(calls[1], [{ origins: ['https://api.example.com/*'] }]);
  });

  it('bridges Chrome callback permissions and storage', async () => {
    const bucket: Bucket = { local: {}, session: {} };
    const api = installChrome(bucket);
    const store = createExtensionStore();

    await store.setSettings({ apiOrigin: 'https://api.example.com', allowInsecureLoopback: false });
    assert.deepStrictEqual(await store.getSettings(), {
      apiOrigin: 'https://api.example.com',
      allowInsecureLoopback: false,
      linkedinOptIn: false
    });

    await store.setTokens({ access_token: 'a', refresh_token: 'r' });
    assert.deepStrictEqual(await store.getTokens(), { access_token: 'a', refresh_token: 'r' });
    await store.clearTokens();
    assert.strictEqual(await store.getTokens(), null);

    assert.strictEqual(await requestHostPermission('https://api.example.com'), true);
    assert.strictEqual(api.runtime.lastError, null);
  });

  it('falls back to in-memory session storage when storage.session is unavailable', async () => {
    const bucket: Bucket = { local: {}, session: {} };
    installChrome(bucket, { session: false });
    const store = createExtensionStore();

    await store.setTokens({ access_token: 'memory-access', refresh_token: 'memory-refresh' });
    await store.setTempToken('memory-temp');
    assert.deepStrictEqual(await store.getTokens(), {
      access_token: 'memory-access',
      refresh_token: 'memory-refresh'
    });
    assert.strictEqual(await store.getTempToken(), 'memory-temp');
    assert.deepStrictEqual(bucket.session, {}, 'tokens must not be written to disk-backed local storage');

    await store.clearTempToken();
    assert.strictEqual(await store.getTempToken(), null);
  });

  it('configures session access level only when the browser implements it', async () => {
    const bucket: Bucket = { local: {}, session: {} };
    const api = installFirefox(bucket);
    await assert.doesNotReject(() => configureSessionStorage());

    const accessLevels: string[] = [];
    api.storage.session!.setAccessLevel = ({ accessLevel }: { accessLevel: string }) => {
      accessLevels.push(accessLevel);
      return Promise.resolve();
    };
    await configureSessionStorage();
    assert.deepStrictEqual(accessLevels, ['TRUSTED_CONTEXTS']);
  });

  it('rejects when the requested extension API is missing', async () => {
    await assert.rejects(() => callApi(undefined, undefined, []), /required extension API/);
  });
});
