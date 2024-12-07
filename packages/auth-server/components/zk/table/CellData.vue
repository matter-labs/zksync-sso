<template>
  <div :class="cellUI">
    <div :class="cellMainUI">
      <slot />
    </div>
    <div :class="cellSubUI">
      <slot name="sub" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { twMerge } from "tailwind-merge";

const props = defineProps<{
  ui?: string;
  uiMain?: string;
  uiSub?: string;
}>();

const cellUI = computed(() => {
  return twMerge("flex flex-col justify-center", props.ui);
});

const cellMainUI = computed(() => {
  return twMerge("text-neutral-950 dark:text-inherit", props.uiMain);
});

const cellSubUI = computed(() => {
  return twMerge("text-neutral-700 dark:text-neutral-400", props.uiSub);
});
</script>
