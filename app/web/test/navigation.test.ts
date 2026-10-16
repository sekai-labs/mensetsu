import { describe, it, expect } from 'vitest'
import {
  WORKSPACE_NAV_SECTIONS,
  ALL_WORKSPACE_NAV_ROUTES,
} from '../components/navigation/types'

describe('Navigation Architecture & Contracts', () => {
  it('defines all 11 workspace destinations', () => {
    expect(ALL_WORKSPACE_NAV_ROUTES).toHaveLength(11)
    const expected = [
      'overview',
      'match',
      'skills',
      'company',
      'role',
      'sources',
      'learning',
      'questions',
      'practice',
      'knowledge',
      'settings',
    ]
    for (const route of expected) {
      expect(ALL_WORKSPACE_NAV_ROUTES).toContain(route)
    }
  })

  it('groups routes into 4 semantic categories with Lucide icons', () => {
    expect(WORKSPACE_NAV_SECTIONS).toHaveLength(4)
    expect(WORKSPACE_NAV_SECTIONS.map((s) => s.title)).toEqual([
      'Overview & Match',
      'Company & Role',
      'Preparation & Questions',
      'Settings',
    ])
    for (const section of WORKSPACE_NAV_SECTIONS) {
      expect(section.items.length).toBeGreaterThan(0)
      for (const item of section.items) {
        expect(item.id).toBeTruthy()
        expect(item.name).toBeTruthy()
        expect(item.path).toBeTruthy()
        expect(item.icon).toBeDefined()
      }
    }
  })
})
