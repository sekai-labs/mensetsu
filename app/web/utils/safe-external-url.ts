export function safeExternalHttpUrl(value: unknown): string {
  if (typeof value !== 'string') return ''
  const candidate = value.trim()
  if (!candidate) return ''

  try {
    const url = new URL(candidate)
    if ((url.protocol !== 'http:' && url.protocol !== 'https:') || !url.hostname || url.username || url.password) {
      return ''
    }
    return url.toString()
  } catch {
    return ''
  }
}
