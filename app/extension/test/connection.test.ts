import { describe, it } from 'node:test';
import assert from 'node:assert/strict';

import { checkApiConnection, CONNECTION_ERROR, HEALTH_PATH } from '../src/features/connection/connection.js';
import type { FetchInit, FetchResponse } from '../src/shared/types.js';

const ORIGIN = 'https://api.example.com';

function jsonResponse(
  body: unknown,
  { ok = true, status = 200, url = `${ORIGIN}${HEALTH_PATH}` }: { ok?: boolean; status?: number; url?: string } = {}
): FetchResponse {
  return {
    ok,
    status,
    url,
    async text() {
      return typeof body === 'string' ? body : JSON.stringify(body);
    }
  };
}

describe('API connection probe', () => {
  it('accepts only a real healthy response from the configured origin', async () => {
    const requests: Array<{ url: string; options: FetchInit }> = [];
    const result = await checkApiConnection(ORIGIN, {
      fetchImpl: async (url: string, options: FetchInit = {}) => {
        requests.push({ url, options });
        return jsonResponse({ status: 'healthy' });
      }
    });

    assert.deepStrictEqual(result, { ok: true, state: 'healthy', origin: ORIGIN });
    assert.strictEqual(requests.length, 1);
    assert.strictEqual(requests[0].url, `${ORIGIN}${HEALTH_PATH}`);
    assert.strictEqual(requests[0].options.method, 'GET');
    assert.strictEqual(requests[0].options.credentials, 'omit');
    assert.strictEqual(requests[0].options.redirect, 'error');
    assert.strictEqual(requests[0].options.headers?.Authorization, undefined);
  });

  it('rejects non-2xx responses', async () => {
    const result = await checkApiConnection(ORIGIN, {
      fetchImpl: async () => jsonResponse({ status: 'unhealthy' }, { ok: false, status: 503 })
    });
    assert.strictEqual(result.ok, false);
    assert.strictEqual(result.code, CONNECTION_ERROR.UNHEALTHY);
    assert.match(result.message ?? '', /503/);
  });

  it('rejects a 200 response without a healthy status payload', async () => {
    const result = await checkApiConnection(ORIGIN, {
      fetchImpl: async () => jsonResponse({ status: 'degraded' })
    });
    assert.strictEqual(result.ok, false);
    assert.strictEqual(result.code, CONNECTION_ERROR.UNHEALTHY);

    const notJson = await checkApiConnection(ORIGIN, {
      fetchImpl: async () => jsonResponse('not json')
    });
    assert.strictEqual(notJson.ok, false);
    assert.strictEqual(notJson.code, CONNECTION_ERROR.UNHEALTHY);
  });

  it('reports timeouts distinctly from offline/network failures', async () => {
    const timedOut = await checkApiConnection(ORIGIN, {
      timeoutMs: 5,
      fetchImpl: (_url: string, options: FetchInit = {}) => new Promise<FetchResponse>((_resolve, reject) => {
        options.signal?.addEventListener('abort', () => {
          const error = new Error('aborted');
          error.name = 'AbortError';
          reject(error);
        });
      })
    });
    assert.strictEqual(timedOut.ok, false);
    assert.strictEqual(timedOut.code, CONNECTION_ERROR.TIMEOUT);

    const unreachable = await checkApiConnection(ORIGIN, {
      fetchImpl: async () => {
        throw new TypeError('Failed to fetch');
      }
    });
    assert.strictEqual(unreachable.ok, false);
    assert.strictEqual(unreachable.code, CONNECTION_ERROR.UNREACHABLE);
  });

  it('rejects a response whose effective URL leaves the configured origin', async () => {
    const result = await checkApiConnection(ORIGIN, {
      fetchImpl: async () => jsonResponse({ status: 'healthy' }, { url: 'https://evil.example/health' })
    });
    assert.strictEqual(result.ok, false);
    assert.strictEqual(result.code, CONNECTION_ERROR.INVALID);
  });

  it('rejects a missing or invalid origin without making a request', async () => {
    let called = false;
    const fetchImpl = async (): Promise<FetchResponse> => {
      called = true;
      return jsonResponse({ status: 'healthy' });
    };

    const missing = await checkApiConnection('', { fetchImpl });
    assert.strictEqual(missing.code, CONNECTION_ERROR.MISSING);

    const invalid = await checkApiConnection('not a url', { fetchImpl });
    assert.strictEqual(invalid.ok, false);
    assert.strictEqual(called, false);
  });
});
