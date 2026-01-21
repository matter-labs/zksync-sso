import { defineNuxtConfig } from "nuxt/config";
import { defineChain } from "viem";
import { localhost } from "viem/chains";
import wasm from "vite-plugin-wasm";

import contracts from "./public/contracts.json";

// TODO: Deploy NFT Quest to ZKsync OS Testnet and update contract addresses
const zksyncOsTestnet = defineChain({
  id: 8022833,
  name: "ZKsyncOS Testnet",
  nativeCurrency: {
    name: "Ether",
    symbol: "ETH",
    decimals: 18,
  },
  rpcUrls: {
    default: {
      http: ["https://zksync-os-testnet-alpha.zksync.dev"],
    },
  },
  blockExplorers: {
    default: {
      name: "ZKsyncOS Testnet Explorer",
      url: "https://zksync-os-testnet-alpha.staging-scan-v2.zksync.dev",
    },
  },
});

// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  modules: [
    "@nuxt/eslint",
    "@nuxtjs/color-mode",
    "@nuxtjs/google-fonts",
    "@nuxtjs/tailwindcss",
    "@pinia/nuxt",
    "@vueuse/nuxt",
    "radix-vue/nuxt",
    "@nuxtjs/color-mode",
    "@nuxtjs/seo",
    "@vueuse/motion/nuxt",
    "nuxt-gtag",
  ],
  $production: {
    runtimeConfig: {
      public: {
        chain: zksyncOsTestnet,
        contracts: {
          // TODO: Deploy contracts to ZKsync OS Testnet
          nft: "0x0000000000000000000000000000000000000000",
          paymaster: "0x0000000000000000000000000000000000000000",
        },
        bundlerUrl: "https://bundler-api.stage-sso.zksync.dev",
        entryPoint: "0x0000000071727De22E5E9d8BAf0edAc6f37da032",
        baseUrl: "https://nft.zksync.dev",
        authServerUrl: "https://auth-test.zksync.dev/confirm",
        explorerUrl: "https://zksync-os-testnet-alpha.staging-scan-v2.zksync.dev",
      },
    },
  },
  devtools: { enabled: false },
  app: {
    pageTransition: { name: "page", mode: "out-in" },
    head: {
      link: [
        { rel: "icon", type: "image/x-icon", href: "/favicon.ico", sizes: "32x32" },
        { rel: "icon", type: "image/png", href: "/icon-96x96.png", sizes: "96x96" },
        { rel: "icon", type: "image/svg+xml", href: "/favicon.svg" },
        { rel: "apple-touch-icon", href: "/apple-touch-icon.png" },
      ],
      bodyAttrs: {
        class: "dark-mode",
      },
    },
  },
  css: ["@/assets/style.scss"],
  site: {
    url: "https://nft-quest.zksync.io",
    name: "ZK NFT Quest",
    description: "Mint your own ZKsync NFT gas-free",
    defaultLocale: "en",
  },
  colorMode: {
    preference: "dark",
  },
  runtimeConfig: {
    public: {
      chain: localhost,
      contracts: {
        nft: contracts.nftContract,
        paymaster: contracts.mockPaymaster,
      },
      bundlerUrl: contracts.bundlerUrl,
      entryPoint: contracts.entryPoint,
      baseUrl: "http://localhost:3006",
      authServerUrl: "http://localhost:3002/confirm",
      explorerUrl: "http://localhost:3010",
    },
  },
  compatibilityDate: "2024-04-03",
  // required for dealing with bigInt
  nitro: {
    esbuild: {
      options: {
        target: "esnext",
      },
    },
  },
  vite: {
    plugins: [wasm()],
    build: {
      target: "esnext",
    },
    css: {
      preprocessorOptions: {
        scss: {
          api: "modern", // Fix warning: "The legacy JS API is deprecated and will be removed in Dart Sass 2.0.0"
        },
      },
    },
  },
  // ssr: false,
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
  googleFonts: {
    families: {
      Inter: [200, 300, 400, 500, 600, 700],
    },
  },
});
