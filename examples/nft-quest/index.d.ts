import type { Chain } from "viem/chains";

declare module "nuxt/schema" {
  interface PublicRuntimeConfig {
    chain: Chain;
    contracts: {
      nft: `0x${string}`;
      paymaster: `0x${string}`;
      marketplace: `0x${string}`;
    };
    baseUrl: string;
    authServerUrl: string;
    explorerUrl: string;
  }
}
// It is always important to ensure you import/export something when augmenting a type
export {};
