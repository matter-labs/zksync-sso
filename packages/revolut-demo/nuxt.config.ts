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
      revolutDemoDeployerKey: "0x3d3cbc973389cb26f657686445bcc75662b415b656078503592ac8c1abb8810e", // Rich Account 0
      network: zksyncInMemoryNode,
      session: "0x476F23ef274F244282252341792c8a610feF78ee",
      passkey: "0x455e8d86DC6728396f8d3B740Fc893F4E20b25Dc",
      accountFactory: "0x23b13d016E973C9915c6252271fF06cCA2098885",
      accountImplementation: "0x6cd5A2354Be0E656e7A1E94F1C0570E08EC4789B",
    }
  },
  $production: {
    runtimeConfig: {
      public: {
        aaveAddress: "0xBC989fDe9e54cAd2aB4392Af6dF60f04873A033A", // Rich Account 0
        revolutDemoDeployerKey: "0x3d3cbc973389cb26f657686445bcc75662b415b656078503592ac8c1abb8810e", // Rich Account 0
        network: {
          ...zksyncInMemoryNode,
          rpcUrls: {
            default: {
              http: ["http://34.55.28.37:8011"],
            },
          },
        },
        session: "0x514b521cAd6A66e14B4770514785d8A5C5fCDFB0",
        passkey: "0x264E0D55d5E2a624a808C216ADE6C21bAb4A3A60",
        accountFactory: "0x23b13d016E973C9915c6252271fF06cCA2098885",
        accountImplementation: "0x9Ef5f1B95D49340716f0cC2366a4b2718392DE08",
      }
    }
  }
});
