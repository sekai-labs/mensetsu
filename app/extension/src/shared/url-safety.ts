const BLOCKED_HOST_SUFFIXES = ['.localhost', '.local', '.internal'] as const;
const IPV4_LITERAL = /^\d{1,3}(?:\.\d{1,3}){3}$/;

export function isUnsafeHost(hostname: string): boolean {
  const host = hostname.toLowerCase().replace(/\.+$/, '');
  if (!host) return true;
  if (host.startsWith('[') || host.includes(':')) return true;
  if (host === 'localhost') return true;
  if (BLOCKED_HOST_SUFFIXES.some((suffix) => host.endsWith(suffix))) return true;
  if (IPV4_LITERAL.test(host)) return true;
  return !host.includes('.');
}

export function safeHttpsUrl(value: unknown): string {
  try {
    const url = new URL(String(value));
    if (url.protocol !== 'https:' || url.username || url.password || url.port) return '';
    if (isUnsafeHost(url.hostname)) return '';
    return url.toString();
  } catch {
    return '';
  }
}
