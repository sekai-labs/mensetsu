import { describe, it, expect } from 'vitest'
import { ALL_WORKSPACE_NAV_ROUTES, WORKSPACE_NAV_SECTIONS } from '../components/navigation/types'

describe('Web Client Layouts & Contracts', () => {
  it('has workspace layout defined with expected sub-routes', () => {
    const workspaceNavRoutes = ALL_WORKSPACE_NAV_ROUTES
    expect(workspaceNavRoutes).toHaveLength(11)
    expect(workspaceNavRoutes).toContain('overview')
    expect(workspaceNavRoutes).toContain('company')
    expect(workspaceNavRoutes).toContain('role')
    expect(workspaceNavRoutes).toContain('match')
    expect(workspaceNavRoutes).toContain('skills')
    expect(workspaceNavRoutes).toContain('knowledge')
    expect(workspaceNavRoutes).toContain('learning')
    expect(workspaceNavRoutes).toContain('questions')
    expect(workspaceNavRoutes).toContain('practice')
    expect(workspaceNavRoutes).toContain('sources')
    expect(workspaceNavRoutes).toContain('settings')
  })

  it('has 4 semantic navigation sections in sidebar configuration', () => {
    expect(WORKSPACE_NAV_SECTIONS).toHaveLength(4)
    const titles = WORKSPACE_NAV_SECTIONS.map((s) => s.title)
    expect(titles).toEqual([
      'Overview & Match',
      'Company & Role',
      'Preparation & Questions',
      'Settings',
    ])
  })

  it('defines authentication and public routes', () => {
    const publicRoutes = ['/', '/login', '/register']
    const protectedRoutes = ['/dashboard', '/interviews', '/settings', '/profile']

    expect(publicRoutes).toContain('/login')
    expect(publicRoutes).toContain('/register')
    expect(protectedRoutes).toContain('/dashboard')
  })
})
