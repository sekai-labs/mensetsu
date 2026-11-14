import { describe, it } from 'node:test';
import assert from 'node:assert/strict';

import { createApi } from '../src/services/api.js';
import { createMemoryStore } from '../src/services/storage.js';
import { normalizeApiOrigin, OriginError } from '../src/shared/api-origin.js';
import { OUTCOME } from '../src/shared/constants.js';
import type { FetchInit, FetchResponse } from '../src/shared/types.js';

interface CapturedRequest extends FetchInit {
  url: string;
  bodyParsed: Record<string, unknown> | null;
}

describe('Rust API Contract and Security Integration', () => {
  describe('Origin Security Validation', () => {
    it('requires HTTPS for remote origins', () => {
      assert.strictEqual(
        normalizeApiOrigin('https://api.mensetsu.example'),
        'https://api.mensetsu.example'
      );
      assert.throws(
        () => normalizeApiOrigin('http://api.mensetsu.example'),
        (err: unknown) => err instanceof OriginError && err.code === 'api_origin_invalid'
      );
    });

    it('permits plain HTTP on loopback only when explicitly configured', () => {
      assert.throws(
        () => normalizeApiOrigin('http://localhost:8000', { allowInsecureLoopback: false }),
        (err: unknown) => err instanceof OriginError
      );
      assert.strictEqual(
        normalizeApiOrigin('http://localhost:8000', { allowInsecureLoopback: true }),
        'http://localhost:8000'
      );
      assert.strictEqual(
        normalizeApiOrigin('http://127.0.0.1:3000', { allowInsecureLoopback: true }),
        'http://127.0.0.1:3000'
      );
    });

    it('rejects credentials, paths, queries, and fragments in API origin', () => {
      assert.throws(() => normalizeApiOrigin('https://user:pass@api.example.com'), /credentials/);
      assert.throws(() => normalizeApiOrigin('https://api.example.com/v1/api'), /path/);
      assert.throws(() => normalizeApiOrigin('https://api.example.com?foo=bar'), /query/);
      assert.throws(() => normalizeApiOrigin('https://api.example.com#section'), /fragment/);
    });
  });

  describe('Contract Endpoint Paths and DTO Shapes', () => {
    function setupMockHttp(): { api: ReturnType<typeof createApi>; requests: CapturedRequest[] } {
      const requests: CapturedRequest[] = [];
      const store = createMemoryStore({
        local: {
          'mensetsu.local.settings': {
            apiOrigin: 'https://api.mensetsu.example',
            allowInsecureLoopback: false
          }
        },
        session: {
          'mensetsu.session.tokens': {
            access_token: 'valid_access_token_jwt',
            refresh_token: 'valid_refresh_token_jwt'
          }
        }
      });

      const mockFetch = async (url: string, options: FetchInit = {}): Promise<FetchResponse> => {
        requests.push({ url, ...options, bodyParsed: options.body ? JSON.parse(options.body) as Record<string, unknown> : null });

        if (url.endsWith('/download')) {
          return {
            ok: true,
            status: 200,
            headers: new Headers({
              'content-type': 'application/pdf',
              'content-length': '8',
              'content-disposition': 'attachment; filename="test_resume.pdf"'
            }),
            arrayBuffer: async () => new Uint8Array([0x25, 0x50, 0x44, 0x46, 0x31, 0x2e, 0x37, 0x0a]).buffer
          };
        }

        return {
          ok: true,
          status: 200,
          headers: new Headers({ 'content-type': 'application/json' }),
          text: async () => JSON.stringify({ id: 'app_contract_1', status: 'prepared' })
        };
      };

      const api = createApi({ store, fetchImpl: mockFetch });
      return { api, requests };
    }

    it('calls GET /api/v1/candidate-profile', async () => {
      const { api, requests } = setupMockHttp();
      await api.candidateProfile();

      assert.strictEqual(requests.length, 1);
      assert.strictEqual(requests[0].url, 'https://api.mensetsu.example/api/v1/candidate-profile');
      assert.strictEqual(requests[0].method, 'GET');
    });

    it('calls GET /api/v1/resume-uploads and download path', async () => {
      const { api, requests } = setupMockHttp();
      await api.listResumes();
      await api.downloadResume('res_123');

      assert.strictEqual(requests[0].url, 'https://api.mensetsu.example/api/v1/resume-uploads');
      assert.strictEqual(requests[0].method, 'GET');

      assert.strictEqual(requests[1].url, 'https://api.mensetsu.example/api/v1/resume-uploads/res_123/download');
      assert.strictEqual(requests[1].method, 'GET');
    });

    it('calls POST /api/v1/applications/{id}/approve with revision only', async () => {
      const { api, requests } = setupMockHttp();
      await api.approveApplication('app_999', { revision: 'rev_hash_1' });

      assert.strictEqual(requests.length, 1);
      assert.strictEqual(requests[0].url, 'https://api.mensetsu.example/api/v1/applications/app_999/approve');
      assert.strictEqual(requests[0].method, 'POST');
      assert.deepStrictEqual(reqKeys(requests[0]), ['revision']);
    });

    it('calls POST /api/v1/applications/{id}/start with no body', async () => {
      const { api, requests } = setupMockHttp();
      await api.startApplication('app_999');

      assert.strictEqual(requests.length, 1);
      assert.strictEqual(requests[0].url, 'https://api.mensetsu.example/api/v1/applications/app_999/start');
      assert.strictEqual(requests[0].method, 'POST');
      assert.strictEqual(requests[0].body, undefined, 'Body must be undefined for start');
    });

    it('calls POST /api/v1/applications/{id}/result with outcome and confirmation_url', async () => {
      const { api, requests } = setupMockHttp();
      await api.resultApplication('app_999', {
        outcome: OUTCOME.CONFIRMED,
        confirmation_url: 'https://boards.greenhouse.io/gitlab/8629019002/confirmation'
      });

      assert.strictEqual(requests.length, 1);
      assert.strictEqual(requests[0].url, 'https://api.mensetsu.example/api/v1/applications/app_999/result');
      assert.strictEqual(requests[0].method, 'POST');
      assert.strictEqual(requests[0].bodyParsed?.outcome, 'confirmed');
      assert.strictEqual(requests[0].bodyParsed?.confirmation_url, 'https://boards.greenhouse.io/gitlab/8629019002/confirmation');
    });

    it('calls POST /api/v1/applications/{id}/block with reason_code', async () => {
      const { api, requests } = setupMockHttp();
      await api.blockApplication('app_999', { reason_code: 'login_required' });

      assert.strictEqual(requests.length, 1);
      assert.strictEqual(requests[0].url, 'https://api.mensetsu.example/api/v1/applications/app_999/block');
      assert.strictEqual(requests[0].method, 'POST');
      assert.deepStrictEqual(reqKeys(requests[0]), ['reason_code']);
      assert.strictEqual(requests[0].bodyParsed?.reason_code, 'login_required');
    });

    it('calls POST /api/v1/applications/{id}/reconcile with submitted boolean', async () => {
      const { api, requests } = setupMockHttp();
      await api.reconcileApplication('app_999', { submitted: true });

      assert.strictEqual(requests.length, 1);
      assert.strictEqual(requests[0].url, 'https://api.mensetsu.example/api/v1/applications/app_999/reconcile');
      assert.strictEqual(requests[0].method, 'POST');
      assert.deepStrictEqual(reqKeys(requests[0]), ['submitted']);
      assert.strictEqual(requests[0].bodyParsed?.submitted, true);
    });

    it('calls GET /api/v1/saved-jobs with cursor and limit and returns the page envelope', async () => {
      const requests: CapturedRequest[] = [];
      const store = createMemoryStore({
        local: {
          'mensetsu.local.settings': { apiOrigin: 'https://api.mensetsu.example' }
        },
        session: {
          'mensetsu.session.tokens': {
            access_token: 'valid_access_token_jwt',
            refresh_token: 'valid_refresh_token_jwt'
          }
        }
      });
      const mockFetch = async (url: string, options: FetchInit = {}): Promise<FetchResponse> => {
        requests.push({ url, ...options, bodyParsed: null });
        return {
          ok: true,
          status: 200,
          headers: new Headers({ 'content-type': 'application/json' }),
          text: async () => JSON.stringify({
            items: [{ id: 'saved_job:1', status: 'awaiting_approval' }],
            next_cursor: 'saved_job:1'
          })
        };
      };

      const api = createApi({ store, fetchImpl: mockFetch });
      const result = await api.listSavedJobs('cursor_1', 25);
      await api.listSavedJobs();

      assert.strictEqual(
        requests[0].url,
        'https://api.mensetsu.example/api/v1/saved-jobs?cursor=cursor_1&limit=25'
      );
      assert.strictEqual(requests[0].method, 'GET');
      assert.strictEqual(requests[0].body, undefined);
      assert.strictEqual(
        requests[1].url,
        'https://api.mensetsu.example/api/v1/saved-jobs'
      );
      assert.deepStrictEqual(result.items.map((item) => item.id), ['saved_job:1']);
      assert.strictEqual(result.next_cursor, 'saved_job:1');
    });

    it('normalizes a missing next_cursor to null', async () => {
      const store = createMemoryStore({
        local: {
          'mensetsu.local.settings': { apiOrigin: 'https://api.mensetsu.example' }
        },
        session: {
          'mensetsu.session.tokens': {
            access_token: 'valid_access_token_jwt',
            refresh_token: 'valid_refresh_token_jwt'
          }
        }
      });
      const mockFetch = async (): Promise<FetchResponse> => ({
        ok: true,
        status: 200,
        headers: new Headers({ 'content-type': 'application/json' }),
        text: async () => JSON.stringify({ items: [] })
      });

      const api = createApi({ store, fetchImpl: mockFetch });
      const result = await api.listSavedJobs();
      assert.deepStrictEqual(result.items, []);
      assert.strictEqual(result.next_cursor, null);
    });
  });

  describe('Auth Token Refresh on 401', () => {
    it('automatically refreshes tokens on 401 and retries original request', async () => {
      let attempts = 0;
      const store = createMemoryStore({
        local: {
          'mensetsu.local.settings': { apiOrigin: 'https://api.mensetsu.example' }
        },
        session: {
          'mensetsu.session.tokens': {
            access_token: 'expired_token',
            refresh_token: 'valid_refresh'
          }
        }
      });

      const mockFetch = async (url: string): Promise<FetchResponse> => {
        if (url.endsWith('/auth/refresh')) {
          return {
            ok: true,
            status: 200,
            headers: new Headers({ 'content-type': 'application/json' }),
            text: async () => JSON.stringify({
              access_token: 'fresh_token_123',
              refresh_token: 'fresh_refresh_456'
            })
          };
        }

        attempts += 1;
        if (attempts === 1) {
          return {
            ok: false,
            status: 401,
            headers: new Headers({ 'content-type': 'application/json' }),
            text: async () => JSON.stringify({ message: 'Token expired' })
          };
        }

        return {
          ok: true,
          status: 200,
          headers: new Headers({ 'content-type': 'application/json' }),
          text: async () => JSON.stringify({ id: 'app_recovered' })
        };
      };

      const api = createApi({ store, fetchImpl: mockFetch });
      const res = await api.candidateProfile();
      assert.strictEqual((res as unknown as { id: string }).id, 'app_recovered');

      const savedTokens = await store.getTokens();
      assert.strictEqual(savedTokens?.access_token, 'fresh_token_123');
      assert.strictEqual(savedTokens?.refresh_token, 'fresh_refresh_456');
    });
  });
});

function reqKeys(req: CapturedRequest): string[] {
  return Object.keys(req.bodyParsed ?? {}).sort();
}
