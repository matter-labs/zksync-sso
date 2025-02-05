<template>
  <NuxtLayout>
    <NuxtPage />
  </NuxtLayout>
</template>

<script lang="ts" setup>
import { createAppKit } from "@reown/appkit/vue";
import { WagmiAdapter } from "@reown/appkit-adapter-wagmi";

// BigInt polyfill
// eslint-disable-next-line @typescript-eslint/no-explicit-any
(BigInt.prototype as any).toJSON = function () {
  return this.toString();
};

// AppKit Configuration
const { defaultChain } = useClientStore();
const runtimeConfig = useRuntimeConfig();

const projectId = runtimeConfig.public.appKitProjectId;
const metadata = {
  name: "ZKsync SSO Auth Server",
  description: "ZKsync SSO Auth Server",
  url: runtimeConfig.public.appUrl,
  icons: [`${runtimeConfig.public.appUrl}/icon-512.png`],
};
const wagmiAdapter = new WagmiAdapter({
  networks: [defaultChain],
  projectId,
});

createAppKit({
  adapters: [wagmiAdapter],
  networks: [defaultChain],
  projectId,
  metadata,
});
</script>
