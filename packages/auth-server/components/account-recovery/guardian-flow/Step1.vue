<template>
  <p class="text-center text-gray-600 dark:text-gray-400">
    Guardian recovery allows you to designate trusted contacts who can help you
    recover your account if you lose access.
  </p>
  <ZkButton
    type="primary"
    class="w-full md:max-w-48 mt-4"
    @click="emit('next')"
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
const emit = defineEmits<{
  (e: "next" | "back"): void;
}>();
</script>
