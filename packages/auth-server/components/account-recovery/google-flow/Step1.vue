<template>
  <p class="text-center text-gray-600 dark:text-gray-400">
    Google recovery allows you to link your Google account for secure account recovery in case you lose access.
  </p>
  <ZkButton
    type="primary"
    class="w-full md:max-w-48 mt-4"
    @click="loginWithGoogle"
  >
    Continue
  </ZkButton>
  <ZkButton
    type="secondary"
    class="w-full md:max-w-48"
    @click="$emit('back')"
  >
    Back
  </ZkButton>
</template>

<script setup lang="ts">
import { toHex } from "viem";
import type { JWT } from "zksync-sso-circuits";

import { useGoogleOauth } from "~/composables/useGoogleOauth";

const { startGoogleOauth, jwt } = useGoogleOauth();

const emit = defineEmits<{
  (e: "next", jwt: JWT): void;
  (e: "back"): void;
}>();

async function loginWithGoogle() {
  const randomValues = new Uint8Array(32);
  const nonce = toHex(crypto.getRandomValues(randomValues));
  await startGoogleOauth(nonce);
}

watch(jwt, () => {
  if (jwt.value !== null) {
    emit("next", jwt.value);
  }
});
</script>
