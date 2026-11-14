import { DEFAULT_SETTINGS, STORAGE_KEY } from '../shared/constants.js';
import { callApi, getExtensionApi } from '../shared/browser-api.js';
import type {
  ExtensionStore,
  PendingEntry,
  PendingSubmission,
  Review,
  Settings,
  StoreAreaName,
  StoredTokens,
  SubmitState
} from '../shared/types.js';
import type { ExtensionStorageArea } from '../shared/webextension.js';

const memorySession = new Map<string, unknown>();

function storageArea(name: StoreAreaName): ExtensionStorageArea | null {
  const api = getExtensionApi();
  if (api?.storage?.[name]) return api.storage[name] as ExtensionStorageArea;
  if (name === 'session') return null;
  throw new Error(`storage.${name} is unavailable in this browser`);
}

async function get(areaName: StoreAreaName, key: string): Promise<unknown> {
  const area = storageArea(areaName);
  if (!area) return memorySession.get(key);
  const result = await callApi<Record<string, unknown>>(area.get as (...args: unknown[]) => unknown, area, [key]);
  return result ? result[key] : undefined;
}

async function set(areaName: StoreAreaName, key: string, value: unknown): Promise<void> {
  const area = storageArea(areaName);
  if (!area) {
    memorySession.set(key, value);
    return;
  }
  await callApi(area.set as (...args: unknown[]) => unknown, area, [{ [key]: value }]);
}

async function remove(areaName: StoreAreaName, key: string): Promise<void> {
  const area = storageArea(areaName);
  if (!area) {
    memorySession.delete(key);
    return;
  }
  await callApi(area.remove as (...args: unknown[]) => unknown, area, [key]);
}

function pendingEntries(entries: Record<string, unknown> | null | undefined): PendingEntry[] {
  return Object.entries(entries || {})
    .filter(([key]) => key.startsWith(STORAGE_KEY.PENDING_PREFIX))
    .map(([key, value]) => ({ applicationId: key.slice(STORAGE_KEY.PENDING_PREFIX.length), pending: value as PendingSubmission }));
}

export function createExtensionStore(): ExtensionStore {
  return {
    async getSettings(): Promise<Settings> {
      const stored = await get('local', STORAGE_KEY.SETTINGS);
      return { ...DEFAULT_SETTINGS, ...((stored || {}) as Partial<Settings>) };
    },
    async setSettings(settings: Partial<Settings>): Promise<void> {
      await set('local', STORAGE_KEY.SETTINGS, settings);
    },
    async getTokens(): Promise<StoredTokens | null> {
      return ((await get('session', STORAGE_KEY.TOKENS)) as StoredTokens | null) || null;
    },
    async setTokens(tokens: StoredTokens): Promise<void> {
      await set('session', STORAGE_KEY.TOKENS, tokens);
    },
    async clearTokens(): Promise<void> {
      await remove('session', STORAGE_KEY.TOKENS);
    },
    async getTempToken(): Promise<string | null> {
      return ((await get('session', STORAGE_KEY.TEMP_TOKEN)) as string | null) || null;
    },
    async setTempToken(token: string): Promise<void> {
      await set('session', STORAGE_KEY.TEMP_TOKEN, token);
    },
    async clearTempToken(): Promise<void> {
      await remove('session', STORAGE_KEY.TEMP_TOKEN);
    },
    async getReview(tabId: number | string): Promise<Review | null> {
      return ((await get('session', `${STORAGE_KEY.REVIEW_PREFIX}${tabId}`)) as Review | null) || null;
    },
    async setReview(tabId: number | string, review: Review): Promise<void> {
      await set('session', `${STORAGE_KEY.REVIEW_PREFIX}${tabId}`, review);
    },
    async deleteReview(tabId: number | string): Promise<void> {
      await remove('session', `${STORAGE_KEY.REVIEW_PREFIX}${tabId}`);
    },
    async getSubmitState(tabId: number | string): Promise<SubmitState | null> {
      return ((await get('session', `${STORAGE_KEY.SUBMIT_PREFIX}${tabId}`)) as SubmitState | null) || null;
    },
    async setSubmitState(tabId: number | string, state: SubmitState): Promise<void> {
      await set('session', `${STORAGE_KEY.SUBMIT_PREFIX}${tabId}`, state);
    },
    async deleteSubmitState(tabId: number | string): Promise<void> {
      await remove('session', `${STORAGE_KEY.SUBMIT_PREFIX}${tabId}`);
    },
    async queuePending(applicationId: string, pending: PendingSubmission): Promise<void> {
      await set('session', `${STORAGE_KEY.PENDING_PREFIX}${applicationId}`, pending);
    },
    async getPending(applicationId: string): Promise<PendingSubmission | null> {
      return ((await get('session', `${STORAGE_KEY.PENDING_PREFIX}${applicationId}`)) as PendingSubmission | null) || null;
    },
    async deletePending(applicationId: string): Promise<void> {
      await remove('session', `${STORAGE_KEY.PENDING_PREFIX}${applicationId}`);
    },
    async listPending(): Promise<PendingEntry[]> {
      const area = storageArea('session');
      if (!area) return pendingEntries(Object.fromEntries(memorySession));
      const all = await callApi<Record<string, unknown>>(area.get as (...args: unknown[]) => unknown, area, [null]);
      return pendingEntries(all);
    }
  };
}

export async function configureSessionStorage(): Promise<void> {
  const api = getExtensionApi();
  const session = api?.storage?.session;
  if (!session || typeof session.setAccessLevel !== 'function') return;
  try {
    await callApi(session.setAccessLevel as (...args: unknown[]) => unknown, session, [{ accessLevel: 'TRUSTED_CONTEXTS' }]);
  } catch {
    return;
  }
}

export interface MemoryStoreSeed {
  local?: Record<string, unknown>;
  session?: Record<string, unknown>;
}

export interface MemoryStore extends ExtensionStore {
  _state: { local: Record<string, unknown>; session: Record<string, unknown> };
}

export function createMemoryStore(seed: MemoryStoreSeed = {}): MemoryStore {
  const state = {
    local: seed.local ?? {},
    session: seed.session ?? {}
  };
  const read = (areaName: StoreAreaName, key: string): unknown => state[areaName]?.[key];
  const write = (areaName: StoreAreaName, key: string, value: unknown): void => {
    if (!state[areaName]) state[areaName] = {};
    state[areaName][key] = value;
  };
  const drop = (areaName: StoreAreaName, key: string): void => {
    if (state[areaName]) delete state[areaName][key];
  };
  return {
    _state: state,
    async getSettings(): Promise<Settings> {
      return { ...DEFAULT_SETTINGS, ...((read('local', STORAGE_KEY.SETTINGS) || {}) as Partial<Settings>) };
    },
    async setSettings(settings: Partial<Settings>): Promise<void> {
      write('local', STORAGE_KEY.SETTINGS, settings);
    },
    async getTokens(): Promise<StoredTokens | null> {
      return (read('session', STORAGE_KEY.TOKENS) as StoredTokens | null) || null;
    },
    async setTokens(tokens: StoredTokens): Promise<void> {
      write('session', STORAGE_KEY.TOKENS, tokens);
    },
    async clearTokens(): Promise<void> {
      drop('session', STORAGE_KEY.TOKENS);
    },
    async getTempToken(): Promise<string | null> {
      return (read('session', STORAGE_KEY.TEMP_TOKEN) as string | null) || null;
    },
    async setTempToken(token: string): Promise<void> {
      write('session', STORAGE_KEY.TEMP_TOKEN, token);
    },
    async clearTempToken(): Promise<void> {
      drop('session', STORAGE_KEY.TEMP_TOKEN);
    },
    async getReview(tabId: number | string): Promise<Review | null> {
      return (read('session', `${STORAGE_KEY.REVIEW_PREFIX}${tabId}`) as Review | null) || null;
    },
    async setReview(tabId: number | string, review: Review): Promise<void> {
      write('session', `${STORAGE_KEY.REVIEW_PREFIX}${tabId}`, review);
    },
    async deleteReview(tabId: number | string): Promise<void> {
      drop('session', `${STORAGE_KEY.REVIEW_PREFIX}${tabId}`);
    },
    async getSubmitState(tabId: number | string): Promise<SubmitState | null> {
      return (read('session', `${STORAGE_KEY.SUBMIT_PREFIX}${tabId}`) as SubmitState | null) || null;
    },
    async setSubmitState(tabId: number | string, state2: SubmitState): Promise<void> {
      write('session', `${STORAGE_KEY.SUBMIT_PREFIX}${tabId}`, state2);
    },
    async deleteSubmitState(tabId: number | string): Promise<void> {
      drop('session', `${STORAGE_KEY.SUBMIT_PREFIX}${tabId}`);
    },
    async queuePending(applicationId: string, pending: PendingSubmission): Promise<void> {
      write('session', `${STORAGE_KEY.PENDING_PREFIX}${applicationId}`, pending);
    },
    async getPending(applicationId: string): Promise<PendingSubmission | null> {
      return (read('session', `${STORAGE_KEY.PENDING_PREFIX}${applicationId}`) as PendingSubmission | null) || null;
    },
    async deletePending(applicationId: string): Promise<void> {
      drop('session', `${STORAGE_KEY.PENDING_PREFIX}${applicationId}`);
    },
    async listPending(): Promise<PendingEntry[]> {
      return Object.entries(state.session || {})
        .filter(([key]) => key.startsWith(STORAGE_KEY.PENDING_PREFIX))
        .map(([key, value]) => ({ applicationId: key.slice(STORAGE_KEY.PENDING_PREFIX.length), pending: value as PendingSubmission }));
    }
  };
}
