<template>
  <p class="text-center text-gray-600 dark:text-gray-400">
    Google recovery allows you to link your Google account for secure account recovery in case you lose access.
  </p>
  <div
    v-if="askForPopups"
    class="w-full bg-red-200 rounded-md border border-red-600 p-2 text-left"
  >
    <h3 class="font-semibold">Popups are blocked</h3>
    <p>Please allow popups for this site in order to continue</p>
  </div>
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

const { startGoogleOauth, jwt } = useGoogleOauth();

const askForPopups = ref<boolean>(false);

const emit = defineEmits<{
  (e: "next", jwt: JWT): void;
  (e: "back"): void;
}>();

async function loginWithGoogle() {
  const randomValues = new Uint8Array(32);
  const nonce = toHex(crypto.getRandomValues(randomValues));
  try {
    await startGoogleOauth(nonce, null, true);
  } catch (error) {
    if (error instanceof PopupNotAllowed) {
      askForPopups.value = true;
    }
  }
}

watch(jwt, () => {
  if (jwt.value !== null) {
    emit("next", jwt.value);
  }
});
</script>
