import { defineConfig, devices } from "@playwright/test";

/**
 * Playwright configuration for the web SDK tests.
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
  /* Fresh local deployments can take longer than Playwright's default 30s timeout. */
  timeout: 240_000,
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

  /* Run your local servers before starting the tests */
  webServer: process.env.PW_MANAGED_SERVERS
    ? undefined
    : [
        {
          command: "CORS_ORIGINS=http://localhost:3002,http://localhost:3004,http://localhost:3005 RPC_URL=http://127.0.0.1:3050 PORT=3004 pnpm nx run auth-server-api:dev",
          url: "http://localhost:3004",
          reuseExistingServer: !process.env.CI,
          timeout: 180_000,
        },
        {
          command: "pnpm nx run auth-server:build:local && NUXT_PUBLIC_AUTH_SERVER_API_URL=http://localhost:3004 NUXT_PUBLIC_CHAIN_RPC_URL=http://localhost:3050 PORT=3002 pnpm -C packages/auth-server exec nuxt preview",
          url: "http://localhost:3002",
          reuseExistingServer: !process.env.CI,
          timeout: 180_000,
        },
        {
          command: "NUXT_PUBLIC_AUTH_SERVER_CONFIRM_URL=http://localhost:3002/confirm pnpm nx run demo-app:build:local && NUXT_PUBLIC_AUTH_SERVER_CONFIRM_URL=http://localhost:3002/confirm PORT=3005 pnpm -C examples/demo-app exec nuxt preview",
          url: "http://localhost:3005",
          reuseExistingServer: !process.env.CI,
          timeout: 180_000,
        },
      ],
});
