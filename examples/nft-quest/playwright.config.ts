import { defineConfig, devices } from "@playwright/test";

/**
 * Read environment variables from file.
 * https://github.com/motdotla/dotenv
 */
// import dotenv from 'dotenv';
// import path from 'path';
// dotenv.config({ path: path.resolve(__dirname, '.env') });

/**
 * See https://playwright.dev/docs/test-configuration.
 */
export default defineConfig({
  testDir: "./tests",
  /* Run tests in files in parallel */
  fullyParallel: true,
  /* Fail the build on CI if you accidentally left test.only in the source code. */
  forbidOnly: !!process.env.CI,
  /* Retry on CI only */
  retries: process.env.CI ? 2 : 0,
  /* Opt out of parallel tests on CI. */
  workers: process.env.CI ? 1 : undefined,
  /* Reporter to use. See https://playwright.dev/docs/test-reporters */
  reporter: [
    ["list"],
    ["html", { open: "never" }],
  ],
  /* Shared settings for all the projects below. See https://playwright.dev/docs/api/class-testoptions. */
  use: {
    /* Base URL to use in actions like `await page.goto('/')`. */
    baseURL: "http://localhost:3006",

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
  /* IMPORTANT: Contracts must already be deployed BEFORE these servers start.
   * The nft-quest:e2e NX target depends on nft-quest-contracts:deploy:local,
   * which ensures contracts are deployed before Playwright launches.
   * Playwright starts webServers sequentially and waits for each URL before starting the next.
   */
  webServer: [
    {
      // Step 1: Start auth-server-api (reads fresh contracts.json)
      // Run directly with node to ensure PORT env var is passed correctly
      command: "cd ../../packages/auth-server-api && PORT=3004 node --experimental-wasm-modules --import tsx src/index.ts",
      url: "http://localhost:3004/api/health",
      reuseExistingServer: !process.env.CI,
      stdout: "pipe",
      stderr: "pipe",
      timeout: 180_000,
    },
    {
      // Step 2: Start auth-server (UI for account creation)
      // Use dev:nuxt-only since we start auth-server-api separately in step 1
      // Clean .nuxt cache first to ensure fresh config from newly deployed contracts
      command: "rm -rf ../../packages/auth-server/.nuxt && NUXT_PUBLIC_AUTH_SERVER_API_URL=http://localhost:3004 pnpm nx dev:nuxt-only auth-server",
      url: "http://localhost:3002",
      reuseExistingServer: !process.env.CI,
      stdout: "pipe",
      stderr: "pipe",
      timeout: 180_000,
    },
    {
      // Step 3: Start nft-quest dev server (contracts already deployed via NX dependency)
      // Clean .nuxt and Vite cache to force fresh config resolution from
      // the newly written contracts.json and .env.local (avoids stale cached addresses)
      command: "rm -rf .nuxt node_modules/.cache/vite && PORT=3006 pnpm nuxt dev",
      url: "http://localhost:3006",
      reuseExistingServer: !process.env.CI,
      stdout: "pipe",
      stderr: "pipe",
      timeout: 180_000,
    },
  ],
});
