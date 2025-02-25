<template>
  <div class="flex flex-col gap-4 text-center text-neutral-700 dark:text-neutral-300">
    <p>
      Your passkeys have been generated successfully.
    </p>
    <p>
      Do you wish to confirm the recovery now?
    </p>
  </div>

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
