import { defineNuxtConfig } from "nuxt/config";
import { zksyncInMemoryNode } from "viem/chains";

// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: "2024-04-03",
  devtools: { enabled: true },
  modules: ["@nuxt/icon", "@vueuse/nuxt", "radix-vue/nuxt", "@nuxt/eslint", "@pinia/nuxt", "@nuxtjs/tailwindcss", "@nuxtjs/google-fonts"],
  ssr: false,
  googleFonts: {
    families: {
      Inter: [300, 400, 500, 600, 700],
    },
  },
  app: {
    head: {
      bodyAttrs: {
        class: "bg-khaki"
      }
    }
  },
  runtimeConfig: {
    public: {
      aaveAddress: "0xBC989fDe9e54cAd2aB4392Af6dF60f04873A033A", // Rich Account 0
      bankDemoDeployerKey: "0x3d3cbc973389cb26f657686445bcc75662b415b656078503592ac8c1abb8810e", // Rich Account 0
      network: zksyncInMemoryNode,
      session: "0x4Cf66C1CADcaCb14f380E347d0434479545Cbe8D",
      passkey: "0xECcED2CB000f39E402B73ae868255a2a20B944c8",
      accountFactory: "0x3391c36d2f5315Ff5f6f909ee9D8c58aAD7DAE06",
      explorerUrl: "http://localhost:3010/",
    }
  },
  $production: {
    runtimeConfig: {
      public: {
        aaveAddress: "0xBC989fDe9e54cAd2aB4392Af6dF60f04873A033A", // Rich Account 0
        bankDemoDeployerKey: "0x3d3cbc973389cb26f657686445bcc75662b415b656078503592ac8c1abb8810e", // Rich Account 0
        network: {
          ...zksyncInMemoryNode,
          rpcUrls: {
            default: {
              http: ["https://node.nvillanueva.com"],
            },
          },
        },
        session: "0x4Cf66C1CADcaCb14f380E347d0434479545Cbe8D",
        passkey: "0xECcED2CB000f39E402B73ae868255a2a20B944c8",
        accountFactory: "0x3391c36d2f5315Ff5f6f909ee9D8c58aAD7DAE06",
        explorerUrl: "http://34.121.229.57:3010/",
      }
    }
  },
  vite: {
    css: {
      preprocessorOptions: {
        scss: {
          // Fix deprecation warnings with modern API
          api: "modern",
        },
      },
    },
  },
});
