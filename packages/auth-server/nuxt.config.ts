import { defineNuxtConfig } from "nuxt/config";
import { localhost } from "viem/chains";
import topLevelAwait from "vite-plugin-top-level-await";
import wasm from "vite-plugin-wasm";

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
      chainId: parseInt(process.env.NUXT_PUBLIC_DEFAULT_CHAIN_ID || "") || localhost.id,
      ssoAccountInterfaceId: "0xb9094997",
      appKitProjectId: process.env.NUXT_PUBLIC_APPKIT_PROJECT_ID || "9bc5059f6eed355858cc56a3388e9b50",
      authServerApiUrl: process.env.NUXT_PUBLIC_AUTH_SERVER_API_URL || "http://localhost:3004",
      prividiumMode: process.env.PRIVIDIUM_MODE === "true",
      prividium: {
        clientId: process.env.PRIVIDIUM_CLIENT_ID || "",
        proxyBaseUrl: process.env.PRIVIDIUM_RPC_PROXY_BASE_URL || "",
        rpcUrl: process.env.PRIVIDIUM_RPC_PROXY_BASE_URL ? `${process.env.PRIVIDIUM_RPC_PROXY_BASE_URL}/rpc` : "",
        authBaseUrl: process.env.PRIVIDIUM_AUTH_BASE_URL || "",
        permissionsApiBaseUrl: process.env.PRIVIDIUM_PERMISSIONS_BASE_URL || "",
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
