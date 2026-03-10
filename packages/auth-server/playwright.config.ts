import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { defineConfig, devices } from "@playwright/test";

const __dirname = dirname(fileURLToPath(import.meta.url));

// Load contracts.json for contract addresses used in e2e tests
const localNode = JSON.parse(
  readFileSync(resolve(__dirname, "./stores/contracts.json"), "utf-8"),
);

// Auth server env vars derived from contracts.json
const authServerEnv = {
  NUXT_PUBLIC_AUTH_SERVER_API_URL: "http://localhost:3004",
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
  PORT: "3002",
};

/**
 * See https://playwright.dev/docs/test-configuration.
 */
export default defineConfig({
  testDir: "./tests",
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1,
  reporter: "html",
  use: {
    baseURL: "http://localhost:3002",
    trace: "on-first-retry",
    video: "retain-on-failure",
  },

  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],

  webServer: [
    {
      command: "PORT=3004 pnpm nx dev auth-server-api",
      url: "http://localhost:3004/api/health",
      reuseExistingServer: !process.env.CI,
      stdout: "pipe",
      stderr: "pipe",
      timeout: 180_000,
    },
    {
      command: "pnpm nx dev:no-deploy auth-server",
      url: "http://localhost:3002",
      reuseExistingServer: !process.env.CI,
      stdout: "pipe",
      stderr: "pipe",
      timeout: 180_000,
      env: { ...process.env, ...authServerEnv },
    },
  ],
});
