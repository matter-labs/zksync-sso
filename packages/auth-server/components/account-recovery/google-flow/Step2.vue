<template>
  <div
    v-if="isLoading"
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

const { addOidcAccount, isLoading } = useRecoveryOidc();
const oidcData = {
  oidcDigest: "0xdeadbeef",
  iss: "0xdeadbeef",
  aud: "0xdeadbeef",
} as OidcData;
addOidcAccount(oidcData);

defineExpose({ isLoading });
</script>
