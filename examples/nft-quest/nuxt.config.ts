import { defineNuxtConfig } from "nuxt/config";
import { zksyncInMemoryNode, zksyncSepoliaTestnet } from "viem/chains";

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
        chain: zksyncSepoliaTestnet,
        contracts: {
          nft: "0x4D533d3B20b50b57268f189F93bFaf8B39c36AB6",
          paymaster: "0x60eef092977DF2738480a6986e2aCD10236b1FA7",
        },
        baseUrl: "https://nft.zksync.dev",
        authServerUrl: "https://auth-test.zksync.dev/confirm",
        explorerUrl: "https://sepolia.explorer.zksync.io",
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
      chain: zksyncInMemoryNode,
      contracts: {
        nft: "0xF4E1ee85f0645b5871B03bc40d151C174F0e86f6",
        paymaster: "0x25B89fa6e157937f845ec0Fb41733B29bc20A4d3",
      },
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
