import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright configuration for Backstage Server E2E tests
 *
 * Starts the backstage server before running tests and verifies
 * key pages load with expected data-testid attributes.
 */
export default defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',

  use: {
    baseURL: 'http://localhost:7007',
    trace: 'on-first-retry',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],

  // Start the Backstage server before running tests
  webServer: {
    command: 'bun run start',
    url: 'http://localhost:7007/health',
    reuseExistingServer: !process.env.CI,
    timeout: 120000, // 2 minutes to start
  },
});
