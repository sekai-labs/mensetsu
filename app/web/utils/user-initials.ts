export function getUserInitials(
  name?: string | null,
  email?: string | null
): string {
  const parts = (name ?? '').trim().split(/\s+/).filter(Boolean)
  if (parts.length > 1) {
    const first = Array.from(parts[0] ?? '')[0] ?? ''
    const last = Array.from(parts[1] ?? '')[0] ?? ''
    return `${first}${last}`.toUpperCase()
  }
  if (parts.length === 1) {
    return Array.from(parts[0] ?? '')
      .slice(0, 2)
      .join('')
      .toUpperCase()
  }
  if (email) {
    return Array.from(email)
      .slice(0, 2)
      .join('')
      .toUpperCase()
  }
  return 'U'
}
