import Redis from 'ioredis'
import crypto from 'node:crypto'

export interface SessionData {
  userId: string
  accessToken: string
  refreshToken: string
  createdAt: number
}

const memoryStore = new Map<string, { data: SessionData; expiresAt: number }>()

let redisClient: Redis | null = null
let isRedisAvailable = false

const getRedisClient = (): Redis | null => {
  if (redisClient) return redisClient
  const redisUrl = process.env.REDIS_URL || 'redis://127.0.0.1:6379'
  try {
    redisClient = new Redis(redisUrl, {
      lazyConnect: true,
      maxRetriesPerRequest: 1,
      enableOfflineQueue: false,
      retryStrategy: (times) => {
        if (times > 3) return null
        return Math.min(times * 100, 1000)
      },
    })

    redisClient.on('connect', () => {
      isRedisAvailable = true
    })

    redisClient.on('error', () => {
      isRedisAvailable = false
    })

    redisClient.connect().catch(() => {
      isRedisAvailable = false
    })
  } catch {
    isRedisAvailable = false
  }

  return redisClient
}

const SESSION_PREFIX = 'mensetsu:session:'
const DEFAULT_TTL_SECONDS = 7 * 24 * 60 * 60 // 7 days (aligns with refresh token TTL)

export const sessionManager = {
  generateTokenId(): string {
    return crypto.randomBytes(32).toString('hex')
  },

  async saveSession(tokenId: string, data: SessionData, ttlSeconds = DEFAULT_TTL_SECONDS): Promise<void> {
    const client = getRedisClient()
    if (client && isRedisAvailable) {
      try {
        await client.set(`${SESSION_PREFIX}${tokenId}`, JSON.stringify(data), 'EX', ttlSeconds)
        return
      } catch {
        isRedisAvailable = false
      }
    }

    // In-memory fallback
    memoryStore.set(tokenId, {
      data,
      expiresAt: Date.now() + ttlSeconds * 1000,
    })
  },

  async getSession(tokenId: string): Promise<SessionData | null> {
    if (!tokenId) return null
    const client = getRedisClient()
    if (client && isRedisAvailable) {
      try {
        const raw = await client.get(`${SESSION_PREFIX}${tokenId}`)
        if (raw) {
          return JSON.parse(raw) as SessionData
        }
      } catch {
        isRedisAvailable = false
      }
    }

    // In-memory fallback
    const item = memoryStore.get(tokenId)
    if (!item) return null
    if (Date.now() > item.expiresAt) {
      memoryStore.delete(tokenId)
      return null
    }
    return item.data
  },

  async updateAccessToken(tokenId: string, newAccessToken: string, newRefreshToken?: string): Promise<void> {
    const session = await this.getSession(tokenId)
    if (!session) return
    session.accessToken = newAccessToken
    if (newRefreshToken) {
      session.refreshToken = newRefreshToken
    }
    await this.saveSession(tokenId, session)
  },

  async deleteSession(tokenId: string): Promise<void> {
    if (!tokenId) return
    const client = getRedisClient()
    if (client && isRedisAvailable) {
      try {
        await client.del(`${SESSION_PREFIX}${tokenId}`)
      } catch {
        isRedisAvailable = false
      }
    }
    memoryStore.delete(tokenId)
  },
}
