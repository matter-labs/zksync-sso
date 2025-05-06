<template>
  <p class="text-center text-neutral-700 dark:text-neutral-300">
    Please connect your wallet
  </p>
  <common-connect-button
    class="w-full lg:w-fit mt-6"
    :type="accountData.isConnected ? 'secondary' : 'primary'"
  />

  <ZkButton
    v-if="accountData.isConnected"
    class="w-full"
    @click="$emit('done')"
  >
    Next
  </ZkButton>
</template>

<script setup lang="ts">
import { useAppKitAccount } from "@reown/appkit/vue";

const accountData = useAppKitAccount();

defineEmits<{
  (e: "done"): void;
}>();
</script>
