<template>
  <p class="text-center text-neutral-700 dark:text-neutral-300">
    Please Review the following information before proceeding.
  </p>

  <account-recovery-confirm-info-card title="To Address">
    <span class="mr-2 font-mono text-lg">{{ request.params.request.params[0].to }}</span>
    <common-copy-to-clipboard
      class="!inline-flex"
      :text="request.params.request.params.to"
    />
  </account-recovery-confirm-info-card>

  <account-recovery-confirm-info-card title="Data">
    <span class="mr-2 font-mono text-lg">{{ request.params.request.params[0].data }}</span>
    <common-copy-to-clipboard
      class="!inline-flex"
      :text="request.params.request.params.data"
    />
  </account-recovery-confirm-info-card>
  <account-recovery-confirm-info-card title="Value">
    <span class="mr-2 font-mono text-lg">{{ request.params.request.params[0].value }}</span>
    <common-copy-to-clipboard
      class="!inline-flex"
      :text="request.params.request.params.value"
    />
  </account-recovery-confirm-info-card>

  <ZkButton
    type="secondary"
    class="w-full"
    @click="emit('approve')"
  >
    Send
  </ZkButton>
</template>

<script setup lang="ts">
import type { WalletKitTypes } from "@reown/walletkit";

const { request } = defineProps<{ request: WalletKitTypes.SessionRequest }>();

const emit = defineEmits<{
  (e: "approve"): void;
}>();
</script>
