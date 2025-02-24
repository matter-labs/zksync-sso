import { defineNuxtConfig } from "nuxt/config";
import { zksyncInMemoryNode, zksyncSepoliaTestnet } from "viem/chains";

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
  nitro: {
    esbuild: {
      options: {
        target: "es2022",
      },
    },
  },
  vite: {
    optimizeDeps: {
      esbuildOptions: {
        target: "es2022",
      },
    },
    css: {
      preprocessorOptions: {
        scss: {
          // Fix deprecation warnings with modern API
          api: "modern",
        },
      },
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
      chainId: parseInt(process.env.NUXT_PUBLIC_DEFAULT_CHAIN_ID || "") || zksyncInMemoryNode.id,
      [zksyncInMemoryNode.id]: {
        nftQuestAddress: "0x111C3E89Ce80e62EE88318C2804920D4c96f92bb",
      },
      [zksyncSepoliaTestnet.id]: {
        nftQuestAddress: "0x4D533d3B20b50b57268f189F93bFaf8B39c36AB6",
      },
      appUrl: process.env.NUXT_PUBLIC_APP_URL || "https://auth-test.zksync.dev",
      ssoAccountInterfaceId: "0xb9094997",
      appKitProjectId: process.env.NUXT_PUBLIC_APPKIT_PROJECT_ID || "",
      googlePublicClient: "866068535821-e9em0h73pee93q4evoajtnnkldsjhqdk.apps.googleusercontent.com",
      saltServiceUrl: process.env.NUXT_PUBLIC_SALT_SERVICE_URL || "http://localhost:3003/salt",
    },
  },
});
