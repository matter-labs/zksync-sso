<template>
  <div class="rounded-2xl bg-neutral-100/50 backdrop-blur-sm dark:bg-gray-800/50 p-6">
    <label class="block text-sm font-medium text-gray-600 dark:text-gray-400 mb-2">
      {{ props.title }}
    </label>
    <div class="text-gray-900 dark:text-gray-100">
      <slot />
    </div>
    <p class="mt-2 text-gray-600 dark:text-gray-400 font-mono text-xs">
      <slot name="footer" />
    </p>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{
  title: string;

}>();
</script>
