import { defineNuxtConfig } from "nuxt/config";
import { zksyncInMemoryNode } from "viem/chains";

const customChain = { ...zksyncInMemoryNode,
   id: 555271,
   rpcUrls: { default: { http: ["https://zkrpc.xsollazk.com"] } },
   blockExplorers: {
    default: {
      url: "https://x.la/explorer",
    },
  },
    };

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
      network: customChain,
      session: "0x76322847A0d0fd94D7a12Beeb946Bb63C3bc38Ea",
      passkey: "0x914bcA24A6022AFB6B7313C9f671950D1a7691DF",
      accountFactory: "0x5381427bf969Ce88442241477a36f443A70DF89A",
      explorerUrl: "https://x.la/explorer",
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
