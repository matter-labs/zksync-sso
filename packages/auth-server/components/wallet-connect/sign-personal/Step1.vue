<template>
  <p class="text-center text-neutral-700 dark:text-neutral-300">
    Please Review the following information before proceeding.
  </p>

  <account-recovery-confirm-info-card title="Message">
    <span class="mr-2 font-mono text-lg">{{ message }}</span>
    <common-copy-to-clipboard
      class="!inline-flex"
      :text="message"
    />
  </account-recovery-confirm-info-card>

  <ZkButton
    type="secondary"
    class="w-full"
    @click="emit('sign')"
  >
    Sign
  </ZkButton>
</template>

<script setup lang="ts">
import type { WalletKitTypes } from "@reown/walletkit";
import { fromHex } from "viem";

const { request } = defineProps<{ request: WalletKitTypes.SessionRequest }>();
const message = computed(() => {
  return fromHex(request.params.request.params[0], "string");
});

const emit = defineEmits<{
  (e: "sign"): void;
}>();
</script>
