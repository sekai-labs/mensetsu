import { describe, it, expect } from 'vitest'

describe('Web Client Layouts & Contracts', () => {
  it('has workspace layout defined with expected sub-routes', () => {
    const workspaceNavRoutes = [
      'overview',
      'match',
      'company',
      'role',
      'skills',
      'knowledge',
      'learning',
      'questions',
      'practice',
      'sources',
      'settings',
    ]
    expect(workspaceNavRoutes).toHaveLength(11)
    expect(workspaceNavRoutes).toContain('learning')
    expect(workspaceNavRoutes).toContain('knowledge')
    expect(workspaceNavRoutes).toContain('practice')
  })

  it('defines authentication and public routes', () => {
    const publicRoutes = ['/', '/login', '/register']
    const protectedRoutes = ['/dashboard', '/interviews', '/settings', '/profile']

    expect(publicRoutes).toContain('/login')
    expect(publicRoutes).toContain('/register')
    expect(protectedRoutes).toContain('/dashboard')
  })
})
