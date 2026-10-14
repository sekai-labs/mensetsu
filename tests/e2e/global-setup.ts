import { FullConfig, chromium } from '@playwright/test'
import * as fs from 'node:fs'
import * as path from 'node:path'

async function globalSetup(config: FullConfig) {
  const baseURL = config.projects[0].use.baseURL || 'http://localhost:3001'
  const apiURL = process.env.API_URL || 'http://localhost:3000'
  const maxAttempts = 90
  const delayMs = 2000

  let isReady = false
  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    try {
      const [webRes, apiRes] = await Promise.all([
        fetch(`${baseURL}/login`, { method: 'GET' }),
        fetch(`${apiURL}/health/ready`, { method: 'GET' }),
      ])
      if (webRes.ok && apiRes.ok) {
        isReady = true
        break
      }
    } catch {
    }
    await new Promise((resolve) => setTimeout(resolve, delayMs))
  }

  if (!isReady) {
    throw new Error(`[globalSetup] Backend/Frontend stack (web: ${baseURL}/login, api: ${apiURL}/health/ready) not healthy with HTTP 200 within 3 minutes.`)
  }

  const authDir = path.resolve(process.cwd(), 'tests/e2e/.auth')
  const stateDir = path.resolve(process.cwd(), 'tests/e2e/.state')
  fs.mkdirSync(authDir, { recursive: true })
  fs.mkdirSync(stateDir, { recursive: true })

  const authFile = path.join(authDir, 'user.json')

  if (fs.existsSync(authFile)) {
    try {
      const browser = await chromium.launch()
      const existingContext = await browser.newContext({ baseURL, storageState: authFile })
      const meRes = await existingContext.request.get('/api/backend/auth/me')
      if (meRes.ok()) {
        await browser.close()
        return
      }
      await browser.close()
    } catch {
    }
  }

  const browser = await chromium.launch()
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
  await browser.close()
}

export default globalSetup
