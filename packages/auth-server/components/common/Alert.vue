<template>
  <div
    class="alert-container"
    :class="`variant-${variant}`"
  >
    <div class="flex">
      <div
        v-if="$slots.icon"
        class="alert-icon-container"
      >
        <slot name="icon" />
      </div>
      <div class="alert-content-container">
        <slot />
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
defineProps({
  variant: {
    type: String as PropType<"primary" | "error">,
    default: "primary",
  },
});
</script>

<style lang="scss" scoped>
.alert-container {
  @apply rounded-xl p-2;
  &.variant-primary {
    @apply bg-primary-50 dark:bg-primary-500/20;

    .alert-icon-container {
      @apply text-blue-400;
    }
    .alert-content-container {
      @apply text-primary-300 dark:text-primary-100;
    }
  }
  &.variant-error {
    @apply bg-error-50 dark:bg-error-500/20;

    .alert-icon-container {
      @apply text-red-400;
    }
    .alert-content-container {
      @apply text-error-300 dark:text-error-100;
    }
  }

  .alert-icon-container {
    @apply shrink-0 size-5;
  }
  .alert-content-container {
    @apply ml-3;
  }
}
</style>
