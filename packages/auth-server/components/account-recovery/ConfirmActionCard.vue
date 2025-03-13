<template>
  <div
    :class="styles.container"
  >
    <component
      :is="icon"
      :class="styles.icon"
    />
    <div class="flex flex-col flex-1 gap-2">
      <span :class="styles.title">
        {{ props.title }}
      </span>
      <p :class="styles.message">
        <slot />
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { CheckCircleIcon, ExclamationTriangleIcon } from "@heroicons/vue/24/solid";

const props = defineProps<{
  title: string;
  type: "success" | "error" | "warning";
}>();

const icon = computed(() => {
  if (props.type === "success") {
    return CheckCircleIcon;
  } else {
    return ExclamationTriangleIcon;
  }
});

const styles = computed(() => ({
  container: ["rounded-2xl flex gap-4", {
    "bg-error-50/50 dark:bg-error-900/30 backdrop-blur-sm p-6 border border-error-200 dark:border-error-700/50": props.type === "error",
    "bg-warning-50/50 dark:bg-warning-900/30 backdrop-blur-sm p-6 border border-warning-200 dark:border-warning-700/50": props.type === "warning",
    "bg-green-50/80 dark:bg-green-900/30 backdrop-blur-sm p-6 border border-green-200 dark:border-green-700/50": props.type === "success",
  }],
  title: ["text-lg font-medium font-semibold", {
    "text-error-600 dark:text-error-400": props.type === "error",
    "text-yellow-600 dark:text-yellow-400": props.type === "warning",
    "text-green-800 dark:text-green-400": props.type === "success",
  }],
  message: ["", {
    "text-error-600 dark:text-error-400": props.type === "error",
    "text-yellow-600 dark:text-yellow-400": props.type === "warning",
    "text-green-800 dark:text-green-400": props.type === "success",
  }],
  icon: ["w-6 h-6", {
    "text-error-600 dark:text-error-400": props.type === "error",
    "text-yellow-600 dark:text-yellow-400": props.type === "warning",
    "text-green-600 dark:text-green-400": props.type === "success",
  }],
}));
</script>
