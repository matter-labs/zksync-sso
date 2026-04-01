import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { defineConfig, devices } from "@playwright/test";

const __dirname = dirname(fileURLToPath(import.meta.url));
const authServerPort = "3202";
const authServerApiPort = "3204";
const authServerUrl = `http://localhost:${authServerPort}`;
const authServerApiUrl = `http://localhost:${authServerApiPort}`;

process.env.PW_AUTH_SERVER_URL = authServerUrl;

// Load contracts.json for contract addresses used in e2e tests
const localNode = JSON.parse(
  readFileSync(resolve(__dirname, "./stores/contracts.json"), "utf-8"),
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
  CHAIN_1_ID: String(localNode.chainId),
  CHAIN_1_RPC_URL: localNode.rpcUrl,
  CORS_ORIGINS: [
    "http://localhost:3000",
    "http://localhost:3002",
    "http://localhost:3003",
    "http://localhost:3004",
    "http://localhost:3005",
    authServerUrl,
    authServerApiUrl,
  ].join(","),
  DEPLOYER_PRIVATE_KEY: "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6",
  PORT: authServerApiPort,
  RPC_URL: localNode.rpcUrl,
};

const reuseExistingServer = !!process.env.CI;

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
    baseURL: authServerUrl,
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
      command: "pnpm nx dev auth-server-api",
      url: `${authServerApiUrl}/api/health`,
      reuseExistingServer,
      stdout: "pipe",
      stderr: "pipe",
      timeout: 180_000,
      env: { ...process.env, ...authServerApiEnv },
    },
    {
      command: `NUXT_PUBLIC_AUTH_SERVER_API_URL=${authServerApiUrl} pnpm nx run auth-server:build:local && PORT=${authServerPort} NUXT_PUBLIC_AUTH_SERVER_API_URL=${authServerApiUrl} pnpm exec nuxt preview`,
      url: authServerUrl,
      reuseExistingServer,
      stdout: "pipe",
      stderr: "pipe",
      timeout: 180_000,
      env: { ...process.env, ...authServerEnv },
    },
  ],
});
