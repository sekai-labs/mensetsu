export function nowIso(now: number = Date.now()): string {
  return new Date(now).toISOString();
}

export function stableStringify(value: unknown): string {
  if (value === null || typeof value !== 'object') return String(JSON.stringify(value));
  if (Array.isArray(value)) return `[${value.map(stableStringify).join(',')}]`;
  const record = value as Record<string, unknown>;
  const keys = Object.keys(record).sort();
  return `{${keys.map((key) => `${JSON.stringify(key)}:${stableStringify(record[key])}`).join(',')}}`;
}

export async function sha256Hex(value: unknown): Promise<string> {
  const text = typeof value === 'string' ? value : stableStringify(value);
  const bytes = new TextEncoder().encode(text);
  const digest = await crypto.subtle.digest('SHA-256', bytes);
  return [...new Uint8Array(digest)].map((b) => b.toString(16).padStart(2, '0')).join('');
}

export function bytesToBase64(bytes: ArrayBuffer | Uint8Array): string {
  let binary = '';
  const view = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);
  const chunk = 0x8000;
  for (let i = 0; i < view.length; i += chunk) {
    binary += String.fromCharCode(...view.subarray(i, i + chunk));
  }
  return btoa(binary);
}

export function base64ToBytes(base64: string): Uint8Array {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) bytes[i] = binary.charCodeAt(i);
  return bytes;
}

export function randomId(prefix = ''): string {
  const bytes = new Uint8Array(16);
  crypto.getRandomValues(bytes);
  const hex = [...bytes].map((b) => b.toString(16).padStart(2, '0')).join('');
  return `${prefix}${hex}`;
}

export function clampText(value: unknown, max: number): string {
  const text = typeof value === 'string' ? value : '';
  if (!(text.length > max)) return text;
  let end = Math.max(0, max);
  if (end > 0) {
    // Never cut between a high and low surrogate: back off one UTF-16 unit so
    // the result is still valid text within the same code-unit budget.
    const low = text.charCodeAt(end);
    const high = text.charCodeAt(end - 1);
    if (low >= 0xdc00 && low <= 0xdfff && high >= 0xd800 && high <= 0xdbff) end -= 1;
  }
  return text.slice(0, end);
}

export function isPlainObject(value: unknown): value is Record<string, unknown> {
  return Boolean(value) && typeof value === 'object' && !Array.isArray(value);
}

export function isNonEmptyString(value: unknown): value is string {
  return typeof value === 'string' && value.trim().length > 0;
}

export function normalizeText(value: unknown): string {
  return typeof value === 'string' ? value.replace(/\s+/g, ' ').trim() : '';
}

export function safeJsonParse<T = unknown>(text: string, fallback: T | null = null): T | null {
  try {
    return JSON.parse(text) as T;
  } catch {
    return fallback;
  }
}

export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
