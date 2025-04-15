<template>
  <p class="text-center text-gray-600 dark:text-gray-400">
    Do you wish to confirm your guardian now?
  </p>
  <ZkButton
    type="primary"
    class="w-full mt-4"
    @click="emit('confirm-now')"
  >
    Confirm Now
  </ZkButton>
  <ZkButton
    type="secondary"
    class="w-full"
    @click="emit('confirm-later')"
  >
    Confirm Later
  </ZkButton>
</template>

<script setup lang="ts">
const emit = defineEmits<{
  (e: "confirm-now" | "confirm-later"): void;
}>();
</script>
