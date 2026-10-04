import { describe, it, expect } from 'vitest'
import { sessionManager } from '../server/utils/session'

describe('Server BFF Session & Redis Token Mapping Manager', () => {
  it('generates high-entropy token IDs', () => {
    const id1 = sessionManager.generateTokenId()
    const id2 = sessionManager.generateTokenId()
    expect(id1).toHaveLength(64) // 32 bytes hex
    expect(id2).toHaveLength(64)
    expect(id1).not.toBe(id2)
  })

  it('saves, retrieves, updates and deletes session state in memory fallback', async () => {
    const tokenId = sessionManager.generateTokenId()
    const sessionData = {
      userId: 'user_456',
      accessToken: 'access.token.jwt',
      refreshToken: 'opaque_refresh_789',
      createdAt: Date.now(),
    }

    await sessionManager.saveSession(tokenId, sessionData, 60)

    const retrieved = await sessionManager.getSession(tokenId)
    expect(retrieved).not.toBeNull()
    expect(retrieved?.userId).toBe('user_456')
    expect(retrieved?.accessToken).toBe('access.token.jwt')
    expect(retrieved?.refreshToken).toBe('opaque_refresh_789')

    await sessionManager.updateAccessToken(tokenId, 'new.access.token', 'new.refresh.token')
    const updated = await sessionManager.getSession(tokenId)
    expect(updated?.accessToken).toBe('new.access.token')
    expect(updated?.refreshToken).toBe('new.refresh.token')

    await sessionManager.deleteSession(tokenId)
    const afterDelete = await sessionManager.getSession(tokenId)
    expect(afterDelete).toBeNull()
  })
})
