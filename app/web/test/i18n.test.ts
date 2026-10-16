import { describe, it, expect } from 'vitest'
import en from '../locales/en.json'
import ja from '../locales/ja.json'
import fr from '../locales/fr.json'

function getLeafKeys(obj: Record<string, any>, prefix = ''): string[] {
  let keys: string[] = []
  for (const [key, value] of Object.entries(obj)) {
    const fullKey = prefix ? `${prefix}.${key}` : key
    if (value && typeof value === 'object' && !Array.isArray(value)) {
      keys = keys.concat(getLeafKeys(value, fullKey))
    } else {
      keys.push(fullKey)
    }
  }
  return keys
}

describe('Internationalization (i18n) Catalogs Parity', () => {
  const enKeys = getLeafKeys(en).sort()
  const jaKeys = getLeafKeys(ja).sort()
  const frKeys = getLeafKeys(fr).sort()

  it('provides matching keys between English and Japanese catalogs', () => {
    expect(jaKeys).toEqual(enKeys)
  })

  it('provides matching keys between English and French catalogs', () => {
    expect(frKeys).toEqual(enKeys)
  })

  it('ensures no empty translation values in any locale', () => {
    const checkNotEmpty = (obj: Record<string, any>, path = '') => {
      for (const [key, value] of Object.entries(obj)) {
        const fullPath = path ? `${path}.${key}` : key
        if (typeof value === 'string') {
          expect(value.trim().length, `Empty value at ${fullPath}`).toBeGreaterThan(0)
        } else if (value && typeof value === 'object') {
          checkNotEmpty(value, fullPath)
        }
      }
    }

    checkNotEmpty(en)
    checkNotEmpty(ja)
    checkNotEmpty(fr)
  })
})
