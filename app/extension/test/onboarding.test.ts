import assert from 'node:assert/strict';
import test from 'node:test';

import {
  assertLinkedInDetectionAllowed,
  connectApiOrigin,
  deriveViewState,
  isLinkedInPageUrl,
  VIEW
} from '../src/features/connection/onboarding.js';
import { createMemoryStore } from '../src/services/storage.js';
import type { Settings } from '../src/shared/types.js';

test('popup only shows sign-in or workspace after a healthy API connection', () => {
  assert.equal(
    deriveViewState({ apiOrigin: '', connection: { state: 'healthy' }, authenticated: true }),
    VIEW.SETUP
  );
  assert.equal(
    deriveViewState({ apiOrigin: 'https://api.example', connection: { state: 'error' }, authenticated: true }),
    VIEW.CONNECTION_ERROR
  );
  assert.equal(
    deriveViewState({ apiOrigin: 'https://api.example', connection: { state: 'healthy' }, authenticated: false }),
    VIEW.LOGIN
  );
  assert.equal(
    deriveViewState({ apiOrigin: 'https://api.example', connection: { state: 'healthy' }, authenticated: true }),
    VIEW.WORKSPACE
  );
});

test('failed API health check does not save the origin or discard existing tokens', async () => {
  const store = createMemoryStore();
  await store.setSettings({ apiOrigin: 'https://old.example', allowInsecureLoopback: false });
  await store.setTokens({ accessToken: 'old-access' });
  await store.setTempToken('old-temp');

  const result = await connectApiOrigin({
    rawOrigin: 'https://new.example/',
    currentSettings: await store.getSettings(),
    store,
    permissionGranted: async () => true,
    checkConnection: async () => ({ ok: false, message: 'API is unhealthy.' })
  });

  assert.equal(result.connected, false);
  assert.match(result.connection.message ?? '', /unhealthy/i);
  assert.equal((await store.getSettings()).apiOrigin, 'https://old.example');
  assert.deepEqual(await store.getTokens(), { accessToken: 'old-access' });
  assert.equal(await store.getTempToken(), 'old-temp');
});

test('successful API connection clears prior-origin tokens before saving the new origin', async () => {
  const store = createMemoryStore();
  await store.setSettings({ apiOrigin: 'https://old.example', allowInsecureLoopback: false });
  await store.setTokens({ accessToken: 'old-access' });
  await store.setTempToken('old-temp');
  const calls: string[] = [];
  const clearTokens = store.clearTokens.bind(store);
  const clearTempToken = store.clearTempToken.bind(store);
  const setSettings = store.setSettings.bind(store);
  store.clearTokens = async () => { calls.push('clearTokens'); await clearTokens(); };
  store.clearTempToken = async () => { calls.push('clearTempToken'); await clearTempToken(); };
  store.setSettings = async (settings: Partial<Settings>) => { calls.push('setSettings'); await setSettings(settings); };

  const result = await connectApiOrigin({
    rawOrigin: 'https://new.example/',
    currentSettings: await store.getSettings(),
    store,
    permissionGranted: async (origin: string) => origin === 'https://new.example',
    checkConnection: async (origin: string) => ({ ok: origin === 'https://new.example' })
  });

  assert.equal(result.connected, true);
  assert.equal(result.tokensCleared, true);
  assert.deepEqual(calls, ['clearTokens', 'clearTempToken', 'setSettings']);
  assert.equal((await store.getSettings()).apiOrigin, 'https://new.example');
  assert.equal(await store.getTokens(), null);
  assert.equal(await store.getTempToken(), null);
});

test('denied host permission prevents probing and leaves settings unchanged', async () => {
  const store = createMemoryStore();
  await store.setSettings({ apiOrigin: 'https://old.example' });
  let probeCount = 0;

  const result = await connectApiOrigin({
    rawOrigin: 'https://new.example',
    currentSettings: await store.getSettings(),
    store,
    permissionGranted: async () => false,
    checkConnection: async () => { probeCount += 1; return { ok: true }; }
  });

  assert.equal(result.connected, false);
  assert.equal(probeCount, 0);
  assert.equal((await store.getSettings()).apiOrigin, 'https://old.example');
});

test('linkedin pages are recognized by host only', () => {
  assert.equal(isLinkedInPageUrl('https://www.linkedin.com/jobs/view/123'), true);
  assert.equal(isLinkedInPageUrl('https://linkedin.com/jobs/view/123'), true);
  assert.equal(isLinkedInPageUrl('https://jobs.linkedin.com/view/123'), true);
  assert.equal(isLinkedInPageUrl('https://notlinkedin.com/jobs/view/123'), false);
  assert.equal(isLinkedInPageUrl('https://linkedin.com.evil.example/jobs'), false);
  assert.equal(isLinkedInPageUrl('not a url'), false);
  assert.equal(isLinkedInPageUrl(undefined), false);
});

test('linkedin detection stays gated until the operator opts in', () => {
  assert.throws(
    () => assertLinkedInDetectionAllowed('https://www.linkedin.com/jobs/view/123', { linkedinOptIn: false }),
    /linkedin_opt_in_required/
  );
  assert.throws(
    () => assertLinkedInDetectionAllowed('https://www.linkedin.com/jobs/view/123', null),
    /linkedin_opt_in_required/
  );
  assert.doesNotThrow(() => assertLinkedInDetectionAllowed('https://www.linkedin.com/jobs/view/123', { linkedinOptIn: true }));
  assert.doesNotThrow(() => assertLinkedInDetectionAllowed('https://boards.greenhouse.io/acme/123', { linkedinOptIn: false }));
  assert.doesNotThrow(() => assertLinkedInDetectionAllowed('https://www.topcv.vn/viec-lam-123', {}));
});
