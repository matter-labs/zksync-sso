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
  /* IMPORTANT: Order matters! Contracts must be deployed BEFORE auth-server-api starts,
   * because auth-server-api reads contract addresses from contracts.json at startup.
   * Playwright starts webServers sequentially and waits for each URL before starting the next.
   */
  webServer: [
    {
      // Step 1: Deploy all contracts first (creates/updates contracts.json)
      // This deploys MSA factory, validators, paymaster, and NFT contract
      // The "server" is just a simple echo to satisfy playwright's URL check
      command: "pnpm nx deploy:local nft-quest-contracts && echo 'Contracts deployed' && node -e \"require('http').createServer((req,res)=>{res.writeHead(200);res.end('ok')}).listen(3099)\"",
      url: "http://localhost:3099",
      reuseExistingServer: !process.env.CI,
      stdout: "pipe",
      stderr: "pipe",
      timeout: 180_000,
    },
    {
      // Step 2: Start auth-server-api (reads fresh contracts.json)
      // Run directly with node to ensure PORT env var is passed correctly
      command: "cd ../../packages/auth-server-api && PORT=3004 node --experimental-wasm-modules --import tsx src/index.ts",
      url: "http://localhost:3004/api/health",
      reuseExistingServer: !process.env.CI,
      stdout: "pipe",
      stderr: "pipe",
      timeout: 180_000,
    },
    {
      // Step 3: Start auth-server (UI for account creation)
      // Use dev:nuxt-only since we start auth-server-api separately in step 2
      command: "NUXT_PUBLIC_AUTH_SERVER_API_URL=http://localhost:3004 pnpm nx dev:nuxt-only auth-server",
      url: "http://localhost:3002",
      reuseExistingServer: !process.env.CI,
      stdout: "pipe",
      stderr: "pipe",
      timeout: 180_000,
    },
    {
      // Step 4: Start nft-quest dev server (contracts already deployed in step 1)
      command: "PORT=3006 pnpm nuxt dev",
      cwd: "../../examples/nft-quest",
      url: "http://localhost:3006",
      reuseExistingServer: !process.env.CI,
      stdout: "pipe",
      stderr: "pipe",
      timeout: 180_000,
    },
  ],
});
