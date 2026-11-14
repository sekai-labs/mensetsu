import { chromium } from '@playwright/test'
import type { FullConfig } from '@playwright/test'
import * as fs from 'node:fs'
import * as path from 'node:path'
import { setTimeout as delay } from 'node:timers/promises'

async function globalSetup(config: FullConfig) {
  const baseURL = config.projects.find((project) => project.name === 'desktop')?.use.baseURL || 'http://localhost:3001'
  const apiURL = process.env.API_URL || 'http://localhost:3000'
  const readyTimeoutMs = 3 * 60_000
  const requestTimeoutMs = 5_000
  const retryDelayMs = 2_000
  const readyDeadline = Date.now() + readyTimeoutMs
  let isReady = false
  let lastProbeError = ''

  while (Date.now() < readyDeadline) {
    const timeoutMs = Math.max(1, Math.min(requestTimeoutMs, readyDeadline - Date.now()))
    const signal = AbortSignal.timeout(timeoutMs)
    try {
      const [webRes, apiRes] = await Promise.all([
        fetch(`${baseURL}/login`, { method: 'GET', signal }),
        fetch(`${apiURL}/health/ready`, { method: 'GET', signal }),
      ])
      if (webRes.ok && apiRes.ok) {
        isReady = true
        break
      }
      lastProbeError = `web status ${webRes.status}, API status ${apiRes.status}`
    } catch (error) {
      lastProbeError = error instanceof Error ? error.message : String(error)
    }

    const remainingMs = readyDeadline - Date.now()
    if (remainingMs > 0) {
      await delay(Math.min(retryDelayMs, remainingMs))
    }
  }

  if (!isReady) {
    const detail = lastProbeError ? ` Last probe: ${lastProbeError}.` : ''
    throw new Error(`[globalSetup] Backend/Frontend stack (web: ${baseURL}/login, api: ${apiURL}/health/ready) not healthy with HTTP 200 within 3 minutes.${detail}`)
  }

  const authDir = path.resolve(config.rootDir, 'tests/e2e/.auth')
  fs.mkdirSync(authDir, { recursive: true })
  const authFile = path.join(authDir, 'user.json')

  if (fs.existsSync(authFile)) {
    const browser = await chromium.launch()
    try {
      const existingContext = await browser.newContext({ baseURL, storageState: authFile })
      const meRes = await existingContext.request.get('/api/backend/auth/me')
      if (meRes.ok()) {
        fs.chmodSync(authFile, 0o600)
        return
      }
      console.warn(`[globalSetup] Existing auth state returned HTTP ${meRes.status()}; registering a fresh user.`)
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error)
      console.warn(`[globalSetup] Existing auth state could not be reused; registering a fresh user: ${detail}`)
    } finally {
      await browser.close()
    }
  }

  const browser = await chromium.launch()
  try {
    const context = await browser.newContext({ baseURL })
    const page = await context.newPage()

    const timestamp = Date.now()
    const email = `e2e_user_${timestamp}@example.com`
    const password = 'StrongPassword123!'

    await page.goto('/register')
    await page.fill('#name', 'E2E User')
    await page.fill('input[type="email"]', email)
    await page.fill('input[type="password"]', password)
    await page.click('button[type="submit"]')
    await page.waitForURL('**/dashboard', { timeout: 30000 })

    const meRes = await context.request.get('/api/backend/auth/me')
    if (!meRes.ok()) {
      throw new Error(`[globalSetup] Failed to verify authenticated user (/api/backend/auth/me): status ${meRes.status()}`)
    }

    await context.storageState({ path: authFile })
    fs.chmodSync(authFile, 0o600)
  } finally {
    await browser.close()
  }
}

export default globalSetup

