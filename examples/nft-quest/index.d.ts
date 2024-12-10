declare module "nuxt/schema" {
  interface PublicRuntimeConfig {
    defaultChainId: number;
    baseUrl: string;
    authServerUrl: string;
    explorerUrl: string;
  }
}
// It is always important to ensure you import/export something when augmenting a type
export {};
