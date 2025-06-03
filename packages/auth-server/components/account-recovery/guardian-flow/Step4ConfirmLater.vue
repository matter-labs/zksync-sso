<template>
  <div class="flex flex-col gap-4 flex-1 justify-center items-center">
    <p class="text-gray-600 dark:text-gray-400">
      Your recovery address was saved. Please use this url to confirm the recovery method:
    </p>
    <div class="flex items-center">
      <a
        :href="recoveryUrl"
        class="max-w-md truncate underline"
        target="_blank"
        rel="noopener noreferrer"
        tabindex="0"
        aria-label="Recovery confirmation URL"
      >
        {{ recoveryUrl }}
      </a>
      <CommonCopyToClipboard
        :text="recoveryUrl"
      />
    </div>
    <ZkButton
      type="primary"
      class="mt-4 w-full"
      @click="emit('next')"
    >
      Close
    </ZkButton>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";

import { useAccountStore } from "~/stores/account";

const { address } = useAccountStore();

const props = defineProps<{
  guardianAddress: string;
}>();

const emit = defineEmits<{
  (e: "next"): void;
}>();

const recoveryUrl = computed(() => {
  return new URL(`/recovery/guardian/confirm-guardian?accountAddress=${address}&guardianAddress=${props.guardianAddress}`, window.location.origin).toString();
});
</script>
