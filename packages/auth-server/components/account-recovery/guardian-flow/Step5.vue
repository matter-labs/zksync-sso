<template>
  <div class="flex flex-col items-center justify-center h-full">
    <p class="text-center text-gray-600 dark:text-gray-400">
      Your guardian has been confirmed and is ready to help you recover your
      account.
    </p>
    <ZkButton
      type="primary"
      class="w-full mt-6"
      @click="emit('next')"
    >
      Done
    </ZkButton>
  </div>
</template>

<script setup lang="ts">
const emit = defineEmits<{
  (e: "next"): void;
}>();
</script>
