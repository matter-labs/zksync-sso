<template>
  <NuxtLayout>
    <NuxtPage />
  </NuxtLayout>
</template>

<script lang="ts" setup>
import type { AppKitNetwork } from "@reown/appkit/networks";
import { createAppKit } from "@reown/appkit/vue";
import { WagmiAdapter } from "@reown/appkit-adapter-wagmi";

import { supportedChains } from "./stores/client";

// BigInt polyfill
// eslint-disable-next-line @typescript-eslint/no-explicit-any
(BigInt.prototype as any).toJSON = function () {
  return this.toString();
};

// AppKit Configuration
const projectId = "9bc5059f6eed355858cc56a3388e9b50";
const metadata = {
  name: "ZKsync SSO Auth Server",
  description: "ZKsync SSO Auth Server",
  url: "https://auth-test.zksync.dev",
  icons: ["https://auth-test.zksync.dev/icon-512.png"],
};
const wagmiAdapter = new WagmiAdapter({
  networks: supportedChains,
  projectId,
});

createAppKit({
  adapters: [wagmiAdapter],
  networks: supportedChains as unknown as [AppKitNetwork, ...AppKitNetwork[]],
  projectId,
  metadata,
});
</script>
