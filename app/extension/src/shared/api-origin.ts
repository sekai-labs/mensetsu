const HTTP_PERMISSION_HOSTS = new Set(['localhost', '127.0.0.1']);

export class OriginError extends Error {
  readonly code: string;

  constructor(code: string, message: string) {
    super(message);
    this.name = 'OriginError';
    this.code = code;
  }
}

export interface NormalizeApiOriginOptions {
  allowInsecureLoopback?: boolean;
}

export function normalizeApiOrigin(raw: unknown, options: NormalizeApiOriginOptions = {}): string {
  const input = String(raw || '').trim();
  if (!input) throw new OriginError('api_origin_missing', 'API origin is required.');

  let candidate = input;
  if (!/^[a-z][a-z0-9+.-]*:/i.test(candidate)) candidate = `https://${candidate}`;

  let url: URL;
  try {
    url = new URL(candidate);
  } catch {
    throw new OriginError('api_origin_invalid', 'API origin is not a valid URL.');
  }

  if (url.username || url.password) {
    throw new OriginError('api_origin_invalid', 'API origin must not contain credentials.');
  }
  if (url.search || url.hash) {
    throw new OriginError('api_origin_invalid', 'API origin must not contain a query or fragment.');
  }
  if (url.pathname && url.pathname !== '/') {
    throw new OriginError('api_origin_invalid', 'API origin must not contain a path.');
  }

  const hostname = url.hostname.toLowerCase();
  const httpLoopback = HTTP_PERMISSION_HOSTS.has(hostname) || hostname.endsWith('.localhost');
  if (url.protocol === 'http:') {
    if (!httpLoopback || options.allowInsecureLoopback !== true) {
      throw new OriginError(
        'api_origin_invalid',
        'HTTPS is required. Plain HTTP is allowed only for localhost, *.localhost, or 127.0.0.1 when loopback development is enabled.'
      );
    }
  } else if (url.protocol !== 'https:') {
    throw new OriginError('api_origin_invalid', 'API origin must use HTTPS.');
  }

  return url.origin;
}

export function resolveApiUrl(origin: string, path: string): URL {
  if (!origin) throw new OriginError('api_origin_missing', 'API origin is not configured.');
  if (typeof path !== 'string' || !path.startsWith('/') || path.startsWith('//')) {
    throw new OriginError('api_origin_invalid', 'API path must be an absolute path on the configured origin.');
  }
  const base = new URL(origin);
  const url = new URL(path, base);
  if (url.origin !== base.origin) {
    throw new OriginError('api_origin_invalid', 'Resolved API URL left the configured origin.');
  }
  return url;
}

export function isSameOriginUrl(candidate: string, origin: string): boolean {
  try {
    const expected = new URL(origin);
    const actual = new URL(candidate);
    return actual.protocol === expected.protocol
      && actual.hostname === expected.hostname
      && actual.port === expected.port;
  } catch {
    return false;
  }
}

export function originPattern(origin: string): string {
  const url = new URL(origin);
  return `${url.protocol}//${url.host}/*`;
}
