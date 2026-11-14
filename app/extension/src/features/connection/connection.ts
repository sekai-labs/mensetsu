import { isSameOriginUrl, resolveApiUrl } from '../../shared/api-origin.js';
import { isPlainObject, safeJsonParse } from '../../shared/util.js';
import type { ConnectionOptions, ConnectionResult, FetchLike } from '../../shared/types.js';

export const HEALTH_PATH = '/health';
export const CONNECTION_TIMEOUT_MS = 8_000;

export const CONNECTION_ERROR = Object.freeze({
  MISSING: 'api_origin_missing',
  INVALID: 'api_origin_invalid',
  TIMEOUT: 'api_timeout',
  UNREACHABLE: 'api_unreachable',
  UNHEALTHY: 'api_unhealthy'
} as const);

export type ConnectionErrorCode = (typeof CONNECTION_ERROR)[keyof typeof CONNECTION_ERROR];

export function isHealthyPayload(value: unknown): boolean {
  return isPlainObject(value)
    && typeof value.status === 'string'
    && value.status.trim().toLowerCase() === 'healthy';
}

export async function checkApiConnection(origin: string, options: ConnectionOptions = {}): Promise<ConnectionResult> {
  const fetchImpl = (options.fetchImpl || globalThis.fetch) as FetchLike | undefined;
  const timeoutMs = Number.isFinite(options.timeoutMs) && Number(options.timeoutMs) > 0
    ? Math.min(Number(options.timeoutMs), 60_000)
    : CONNECTION_TIMEOUT_MS;

  let url: URL;
  try {
    url = resolveApiUrl(origin, HEALTH_PATH);
  } catch (error) {
    const failure = error as { code?: string; message?: string };
    return {
      ok: false,
      state: 'error',
      code: failure?.code || CONNECTION_ERROR.INVALID,
      message: String(failure?.message || 'The API origin is not valid.')
    };
  }

  if (typeof fetchImpl !== 'function') {
    return {
      ok: false,
      state: 'error',
      code: CONNECTION_ERROR.UNREACHABLE,
      message: 'This browser cannot make network requests from the extension background context.'
    };
  }

  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);
  let response;
  try {
    response = await fetchImpl(url.toString(), {
      method: 'GET',
      headers: { Accept: 'application/json' },
      credentials: 'omit',
      redirect: 'error',
      cache: 'no-store',
      referrerPolicy: 'no-referrer',
      signal: controller.signal
    });
  } catch (error) {
    const timedOut = (error as Error)?.name === 'AbortError';
    return {
      ok: false,
      state: 'error',
      code: timedOut ? CONNECTION_ERROR.TIMEOUT : CONNECTION_ERROR.UNREACHABLE,
      message: timedOut
        ? `Connection to ${url.origin} timed out after ${timeoutMs} ms.`
        : `Could not reach ${url.origin}. Check the URL, network, and that the API is running.`
    };
  } finally {
    clearTimeout(timer);
  }

  if (response?.url && !isSameOriginUrl(response.url, url.origin)) {
    return {
      ok: false,
      state: 'error',
      code: CONNECTION_ERROR.INVALID,
      message: 'The API responded from a different origin; refusing to continue.'
    };
  }

  if (!response?.ok) {
    return {
      ok: false,
      state: 'error',
      code: CONNECTION_ERROR.UNHEALTHY,
      message: `${url.origin} returned HTTP ${response?.status ?? 'unknown'} from ${HEALTH_PATH}.`
    };
  }

  let body: unknown;
  try {
    body = typeof response.text === 'function'
      ? safeJsonParse(await response.text(), null)
      : response.json
        ? await response.json()
        : null;
  } catch {
    body = null;
  }

  if (!isHealthyPayload(body)) {
    return {
      ok: false,
      state: 'error',
      code: CONNECTION_ERROR.UNHEALTHY,
      message: `${url.origin}${HEALTH_PATH} did not report a healthy status.`
    };
  }

  return { ok: true, state: 'healthy', origin: url.origin };
}
