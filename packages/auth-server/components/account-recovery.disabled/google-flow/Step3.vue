<template>
  <div
    class="flex flex-col items-center gap-4"
  >
    <p class="text-center text-gray-600 dark:text-gray-400">
      Everything is set! You can now recover your wallet
      using your Google account.
    </p>
  </div>

  <ZkButton
    type="secondary"
    class="w-full md:max-w-48"
    @click="$emit('finish')"
  >
    Finish
  </ZkButton>
</template>

<script setup lang="ts">
defineEmits<{
  (e: "finish"): void;
}>();
</script>
