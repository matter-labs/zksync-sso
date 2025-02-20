<template>
  <div
    v-if="addOidcAccountIsLoading"
    class="flex flex-col items-center gap-4"
  >
    <common-spinner class="w-8 h-8" />
    <p class="text-center text-gray-600 dark:text-gray-400">
      Linking your Google account...
    </p>
  </div>

  <div
    v-else
    class="flex flex-col items-center justify-center h-full"
  >
    <p class="text-center text-gray-600 dark:text-gray-400">
      Your Google account has been linked and is ready to help you recover your account.
    </p>
  </div>
</template>

<script setup lang="ts">
import type { OidcData } from "zksync-sso/client";

import { useRecoveryOidc } from "~/composables/useRecoveryOidc";

const { addOidcAccount, addOidcAccountIsLoading } = useRecoveryOidc();
const oidcData = {
  oidcDigest: "0xdeadbeef",
  iss: "0x68747470733a2f2f6163636f756e74732e676f6f676c652e636f6d",
  aud: "0x7367696e2e617070732e676f6f676c6575736572636f6e74656e742e636f6d",
} as OidcData;
addOidcAccount(oidcData);

defineExpose({ addOidcAccountIsLoading });
</script>
