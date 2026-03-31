import { defineConfig, devices } from "@playwright/test";

/**
 * Playwright configuration for ERC-4337 web SDK tests.
 * This config is specifically for testing the web-sdk-test page
 * which tests deploy, fund, and transfer operations.
 */
export default defineConfig({
  testDir: "./tests",
  /* Only run the web-sdk-test spec */
  testMatch: "**/web-sdk-test.spec.ts",
  /* Run tests in files in parallel */
  fullyParallel: true,
  /* Fail the build on CI if you accidentally left test.only in the source code. */
  forbidOnly: !!process.env.CI,
  /* Retry on CI only */
  retries: process.env.CI ? 2 : 0,
  /* Opt out of parallel tests on CI. */
  workers: 1,
  /* Reporter to use. See https://playwright.dev/docs/test-reporters */
  reporter: "html",
  /* Shared settings for all the projects below. See https://playwright.dev/docs/api/class-testoptions. */
  use: {
    /* Base URL to use in actions like `await page.goto('/')`. */
    baseURL: "http://localhost:3005",

    /* Collect trace when retrying the failed test. See https://playwright.dev/docs/trace-viewer */
    trace: "on",
  },

  /* Configure projects for major browsers */
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],

  /* Run your local server before starting the tests */
  webServer: [
    {
      command: "pnpm nx preview demo-app",
      url: "http://localhost:3005",
      reuseExistingServer: !process.env.CI,
      timeout: 180_000,
    },
  ],
});
