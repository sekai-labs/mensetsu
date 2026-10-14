import { defineConfig, devices } from '@playwright/test'
import * as path from 'node:path'

const authFile = path.resolve(__dirname, 'tests/e2e/.auth/user.json')

export default defineConfig({
  testDir: './tests/e2e',
  globalSetup: './tests/e2e/global-setup.ts',
  timeout: 900000,
  expect: {
    timeout: 30000,
  },
  fullyParallel: false,
  reporter: 'list',
  use: {
    baseURL: process.env.BASE_URL || 'http://localhost:3001',
    storageState: authFile,
    trace: 'on-first-retry',
  },
  projects: [
    {
      name: 'desktop',
      testMatch: /.*\.desktop\.spec\.ts/,
      use: {
        ...devices['Desktop Chrome'],
        viewport: { width: 1440, height: 900 },
      },
    },
    {
      name: 'mobile',
      testMatch: /.*\.mobile\.spec\.ts/,
      dependencies: ['desktop'],
      use: {
        ...devices['Pixel 7'],
      },
    },
  ],
})
