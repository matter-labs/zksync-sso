import { defineNuxtConfig } from "nuxt/config";
import { zksyncInMemoryNode } from "viem/chains";
import localChainData from "./local-node.json";

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
      session: localChainData.session,
      passkey: localChainData.passkey,
      accountFactory: localChainData.accountFactory,
      recovery: localChainData.recovery,
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
        session: "0xdCdAC285612841db9Fa732098EAF04A917A71A28",
        passkey: "0xCeC63BD0f35e04F3Bef1128bA3A856A7BB4D88f1",
        accountFactory: "0x23b13d016E973C9915c6252271fF06cCA2098885",
        recovery: "0x6AA83E35439D71F28273Df396BC7768dbaA9849D",
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
