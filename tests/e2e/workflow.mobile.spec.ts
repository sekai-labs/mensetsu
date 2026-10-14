import { test, expect } from '@playwright/test'
import * as fs from 'node:fs'
import * as path from 'node:path'

test.describe('Mobile Viewport Adaptations', () => {
  const stateDir = path.resolve(process.cwd(), 'tests/e2e/.state')
  const runStateFile = path.join(stateDir, 'active_run.json')

  let runId = ''

  test.beforeAll(() => {
    if (fs.existsSync(runStateFile)) {
      const state = JSON.parse(fs.readFileSync(runStateFile, 'utf8'))
      runId = state.runId
    }
  })

  test('1. Mobile overview first, full-screen inspector sheet, separate logs tab', async ({ page }) => {
    expect(runId.length).toBeGreaterThan(0)
    await page.goto(`/workflows/runs/${runId}`)

    await expect(page.locator('[data-testid="run-header"]')).toBeVisible()
    await expect(page.locator('.vue-flow')).toBeVisible()

    const taskNode = page.locator('[data-testid="task-node"]').first()
    await expect(taskNode).toBeVisible({ timeout: 60000 })
    await taskNode.click()

    const inspector = page.locator('aside')
    await expect(inspector).toBeVisible()
    const box = await inspector.boundingBox()
    const viewport = page.viewportSize()
    expect(box).not.toBeNull()
    expect(viewport).not.toBeNull()
    expect(box!.width).toBeGreaterThanOrEqual(viewport!.width * 0.9)

    await inspector.locator('button').first().click()
    await expect(inspector).toBeHidden()

    await page.getByRole('button', { name: /Execution Logs/i }).click()
    await expect(page.locator('input[placeholder="Search logs..."]')).toBeVisible()
  })
})
