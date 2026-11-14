import { describe, expect, it } from 'vitest'
import { safeExternalHttpUrl } from '../utils/safe-external-url'

describe('safeExternalHttpUrl', () => {
  it('preserves absolute HTTP and HTTPS links, including local OAuth servers', () => {
    expect(safeExternalHttpUrl('https://example.com/source?id=1')).toBe('https://example.com/source?id=1')
    expect(safeExternalHttpUrl('http://127.0.0.1:8787/authorize')).toBe('http://127.0.0.1:8787/authorize')
  })

  it('rejects executable schemes, relative paths, credentials, and non-strings', () => {
    for (const value of [
      'javascript:alert(1)',
      'data:text/html,<script>alert(1)</script>',
      'file:///etc/passwd',
      '/settings',
      'https://user:password@example.com',
      null,
      undefined,
      123,
    ]) {
      expect(safeExternalHttpUrl(value)).toBe('')
    }
  })
})
