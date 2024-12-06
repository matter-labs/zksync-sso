import type { Chain } from "viem";

declare module "nuxt/schema" {
  interface PublicRuntimeConfig {
    chain: Chain;
    contracts: {
      room: `0x${string}`;
      paymaster: `0x${string}`;
    };
    baseUrl: string;
    authServerUrl: string;
    explorerUrl: string;
  }
}
// It is always important to ensure you import/export something when augmenting a type
export {};
