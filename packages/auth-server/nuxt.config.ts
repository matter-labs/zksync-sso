import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import { defineNuxtConfig } from "nuxt/config";
import topLevelAwait from "vite-plugin-top-level-await";
import wasm from "vite-plugin-wasm";

// Load contracts from stores/contracts.json as fallback for local development
const contractsJsonPath = join(fileURLToPath(new URL(".", import.meta.url)), "stores/contracts.json");
const contractsFromFile: {
  chainId?: number;
  rpcUrl?: string;
  factory?: string;
  eoaValidator?: string;
  webauthnValidator?: string;
  sessionValidator?: string;
  guardianExecutor?: string;
  beacon?: string;
  bundlerUrl?: string;
  testPaymaster?: string;
  entryPoint?: string;
} = existsSync(contractsJsonPath) ? JSON.parse(readFileSync(contractsJsonPath, "utf-8")) : {};

// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: "2024-07-08",
  devtools: { enabled: false },
  modules: [
    "@nuxt/eslint",
    "@pinia/nuxt",
    "@nuxtjs/tailwindcss",
    "@nuxtjs/google-fonts",
    "@vueuse/nuxt",
    "radix-vue/nuxt",
    "@nuxtjs/color-mode",
    "nuxt-gtag",
    "nuxt-typed-router",
  ],
  app: {
    head: {
      title: "ZKsync SSO",
      link: [
        { rel: "icon", type: "image/x-icon", href: "/favicon.ico", sizes: "32x32" },
        { rel: "icon", type: "image/png", href: "/icon-96x96.png", sizes: "96x96" },
        { rel: "icon", type: "image/svg+xml", href: "/favicon.svg" },
        { rel: "apple-touch-icon", href: "/apple-touch-icon.png" },
      ],
      script: [
        {
          src: "/snarkjs.min.js",
        },
      ],
    },
  },
  ssr: false,
  devServer: {
    port: 3002,
  },
  css: ["@/assets/css/tailwind.css", "@/assets/css/style.scss", "web3-avatar-vue/dist/style.css"],
  googleFonts: {
    families: {
      Inter: [400, 500, 600, 700],
    },
  },
  colorMode: {
    preference: "dark",
  },
  vite: {
    plugins: [
      wasm(),
      topLevelAwait(),
    ],
    optimizeDeps: {
      // Wait for full crawl before serving, preventing mid-session reloads from late dep discovery.
      holdUntilCrawlEnd: true,
    },
    css: {
      preprocessorOptions: {
        scss: {
          // Fix deprecation warnings with modern API
          api: "modern",
        },
      },
    },
    worker: {
      format: "es",
      plugins: () => [
        wasm(),
        topLevelAwait(),
      ],
    },
  },
  eslint: {
    config: {
      stylistic: {
        indent: 2,
        semi: true,
        quotes: "double",
        arrowParens: true,
        quoteProps: "as-needed",
        braceStyle: "1tbs",
      },
    },
  },
  runtimeConfig: {
    public: {
      // Chain configuration (env vars take priority, fall back to stores/contracts.json)
      chainId: parseInt(process.env.NUXT_PUBLIC_CHAIN_ID || "") || contractsFromFile.chainId || 0,
      chainName: process.env.NUXT_PUBLIC_CHAIN_NAME || "",
      chainRpcUrl: process.env.NUXT_PUBLIC_CHAIN_RPC_URL || contractsFromFile.rpcUrl || "",
      blockExplorerUrl: process.env.NUXT_PUBLIC_BLOCK_EXPLORER_URL || "",
      blockExplorerApiUrl: process.env.NUXT_PUBLIC_BLOCK_EXPLORER_API_URL || "",

      // Contract addresses (env vars take priority, fall back to stores/contracts.json)
      factoryAddress: process.env.NUXT_PUBLIC_FACTORY_ADDRESS || contractsFromFile.factory || "",
      eoaValidatorAddress: process.env.NUXT_PUBLIC_EOA_VALIDATOR_ADDRESS || contractsFromFile.eoaValidator || "",
      webauthnValidatorAddress: process.env.NUXT_PUBLIC_WEBAUTHN_VALIDATOR_ADDRESS || contractsFromFile.webauthnValidator || "",
      sessionValidatorAddress: process.env.NUXT_PUBLIC_SESSION_VALIDATOR_ADDRESS || contractsFromFile.sessionValidator || "",
      guardianExecutorAddress: process.env.NUXT_PUBLIC_GUARDIAN_EXECUTOR_ADDRESS || contractsFromFile.guardianExecutor || "",
      bundlerUrl: process.env.NUXT_PUBLIC_BUNDLER_URL || contractsFromFile.bundlerUrl || "",
      beaconAddress: process.env.NUXT_PUBLIC_BEACON_ADDRESS || contractsFromFile.beacon || "",
      testPaymasterAddress: process.env.NUXT_PUBLIC_TEST_PAYMASTER_ADDRESS || contractsFromFile.testPaymaster || "",
      entryPointAddress: process.env.NUXT_PUBLIC_ENTRY_POINT_ADDRESS || contractsFromFile.entryPoint || "",
      accountPaymasterAddress: process.env.NUXT_PUBLIC_ACCOUNT_PAYMASTER_ADDRESS || "",
      recoveryOidcAddress: process.env.NUXT_PUBLIC_RECOVERY_OIDC_ADDRESS || "",
      oidcKeyRegistryAddress: process.env.NUXT_PUBLIC_OIDC_KEY_REGISTRY_ADDRESS || "",
      oidcVerifierAddress: process.env.NUXT_PUBLIC_OIDC_VERIFIER_ADDRESS || "",
      passkeyAddress: process.env.NUXT_PUBLIC_PASSKEY_ADDRESS || "",

      ssoAccountInterfaceId: "0xb9094997",
      appKitProjectId: process.env.NUXT_PUBLIC_APPKIT_PROJECT_ID || "9bc5059f6eed355858cc56a3388e9b50",
      authServerApiUrl: process.env.NUXT_PUBLIC_AUTH_SERVER_API_URL || "http://localhost:3004",
      prividiumMode: process.env.NUXT_PUBLIC_PRIVIDIUM_MODE === "true",
      prividium: {
        clientId: process.env.NUXT_PUBLIC_PRIVIDIUM_CLIENT_ID || "",
        authBaseUrl: process.env.NUXT_PUBLIC_PRIVIDIUM_AUTH_BASE_URL || "",
        apiBaseUrl: process.env.NUXT_PUBLIC_PRIVIDIUM_API_BASE_URL || "",
      },
      oidc: {
        googlePublicClient: "69763429492-f7nl555i50akmail80pid3m4hhsg7u2n.apps.googleusercontent.com",
        saltServiceUrl: process.env.NUXT_PUBLIC_SALT_SERVICE_URL || "https://sso-oidc.zksync.dev/salt",
        zkeyUrl: process.env.NUXT_PUBLIC_ZKEY_URL || "https://storage.googleapis.com/test-sso-oidc-recovery/jwt-tx-validation.final.zkey",
        witnessUrl: process.env.NUXT_PUBLIC_WITNESS_WASM_URL || "https://storage.googleapis.com/test-sso-oidc-recovery/jwt-tx-validation.wasm",
      },
    },
  },
});
