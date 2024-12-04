<template>
  <div class="rounded-xl bg-primary-50 dark:bg-primary-500/20 p-2">
    <div class="flex">
      <div
        v-if="$slots.icon"
        class="shrink-0 text-blue-400 size-5"
      >
        <slot name="icon" />
      </div>
      <div class="ml-3 text-primary-300 dark:text-primary-100">
        <slot />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">

</script>
