import { describe, expect, it } from 'vitest'
import { getUserInitials } from '../utils/user-initials'

describe('getUserInitials', () => {
  it('keeps astral characters intact for names and email fallback', () => {
    expect(getUserInitials('😀 Ada')).toBe('😀A')
    expect(getUserInitials('😀x')).toBe('😀X')
    expect(getUserInitials('   ', '😀person@example.com')).toBe('😀P')
  })

  it('preserves the existing ASCII initials behavior', () => {
    expect(getUserInitials('Ada Lovelace')).toBe('AL')
    expect(getUserInitials('Ada')).toBe('AD')
    expect(getUserInitials(undefined, 'ada@example.com')).toBe('AD')
    expect(getUserInitials()).toBe('U')
  })
})
