import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { defineConfig, devices } from "@playwright/test";

const __dirname = dirname(fileURLToPath(import.meta.url));
const authServerPort = "3102";
const authServerApiPort = "3104";
const demoAppPort = "3105";
const authServerUrl = `http://localhost:${authServerPort}`;
const authServerApiUrl = `http://localhost:${authServerApiPort}`;
const demoAppUrl = `http://localhost:${demoAppPort}`;

process.env.PW_AUTH_SERVER_URL = authServerUrl;
process.env.PW_DEMO_APP_URL = demoAppUrl;

// Load contracts.json for contract addresses used in e2e tests
const localNode = JSON.parse(
  readFileSync(resolve(__dirname, "../../packages/auth-server/stores/contracts.json"), "utf-8"),
);

// Auth server env vars derived from contracts.json
const authServerEnv = {
  NUXT_PUBLIC_AUTH_SERVER_API_URL: authServerApiUrl,
  NUXT_PUBLIC_CHAIN_ID: String(localNode.chainId),
  NUXT_PUBLIC_CHAIN_NAME: "Localhost",
  NUXT_PUBLIC_CHAIN_RPC_URL: localNode.rpcUrl,
  NUXT_PUBLIC_FACTORY_ADDRESS: localNode.factory,
  NUXT_PUBLIC_EOA_VALIDATOR_ADDRESS: localNode.eoaValidator,
  NUXT_PUBLIC_WEBAUTHN_VALIDATOR_ADDRESS: localNode.webauthnValidator,
  NUXT_PUBLIC_SESSION_VALIDATOR_ADDRESS: localNode.sessionValidator,
  NUXT_PUBLIC_GUARDIAN_EXECUTOR_ADDRESS: localNode.guardianExecutor,
  NUXT_PUBLIC_BEACON_ADDRESS: localNode.beacon,
  NUXT_PUBLIC_BUNDLER_URL: localNode.bundlerUrl,
  NUXT_PUBLIC_TEST_PAYMASTER_ADDRESS: localNode.testPaymaster || "",
  PORT: authServerPort,
};

const authServerApiEnv = {
  PORT: authServerApiPort,
  RPC_URL: localNode.rpcUrl,
  CHAIN_1_ID: String(localNode.chainId),
  CHAIN_1_RPC_URL: localNode.rpcUrl,
  CORS_ORIGINS: [
    "http://localhost:3000",
    "http://localhost:3002",
    "http://localhost:3003",
    "http://localhost:3004",
    "http://localhost:3005",
    demoAppUrl,
    authServerUrl,
    authServerApiUrl,
  ].join(","),
  DEPLOYER_PRIVATE_KEY: "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6",
};

const reuseExistingServer = !!process.env.CI;

/**
 * See https://playwright.dev/docs/test-configuration.
 */
export default defineConfig({
  testDir: "./tests",
  testMatch: "**/create-account.spec.ts",
  /* Run tests in files in parallel */
  fullyParallel: true,
  /* Fail the build on CI if you accidentally left test.only in the source code. */
  forbidOnly: !!process.env.CI,
  /* Retry on CI only */
  retries: process.env.CI ? 2 : 0,
  /* Run tests sequentially to avoid nonce collision on funding transactions */
  workers: 1,
  /* Reporter to use. See https://playwright.dev/docs/test-reporters */
  reporter: [
    ["list"],
    ["html", { open: process.env.CI ? "never" : "on-failure" }],
  ],
  /* Shared settings for all the projects below. See https://playwright.dev/docs/api/class-testoptions. */
  use: {
    /* Base URL to use in actions like `await page.goto('/')`. */
    baseURL: demoAppUrl,

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

  /* Run local servers before starting the tests */
  webServer: [
    {
      command: "pnpm nx dev auth-server-api",
      url: `${authServerApiUrl}/api/health`,
      reuseExistingServer,
      stdout: "pipe",
      stderr: "pipe",
      timeout: 180_000,
      env: { ...process.env, ...authServerApiEnv },
    },
    {
      command: `pnpm nx run auth-server:build:local && PORT=${authServerPort} pnpm -C ../../packages/auth-server exec nuxt preview`,
      url: authServerUrl,
      reuseExistingServer,
      stdout: "pipe",
      stderr: "pipe",
      timeout: 180_000,
      env: { ...process.env, ...authServerEnv },
    },
    {
      command: `NUXT_PUBLIC_AUTH_SERVER_CONFIRM_URL=${authServerUrl}/confirm pnpm nx run demo-app:build:local && NUXT_PUBLIC_AUTH_SERVER_CONFIRM_URL=${authServerUrl}/confirm PORT=${demoAppPort} pnpm exec nuxt preview`,
      url: demoAppUrl,
      reuseExistingServer,
      stdout: "pipe",
      stderr: "pipe",
      timeout: 180_000,
    },
  ],
});
