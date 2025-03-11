<template>
  <div class="flex items-center justify-center gap-2">
    <div
      v-for="step in totalSteps"
      :key="step"
      class="flex items-center"
    >
      <div
        class="w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium transition-colors"
        :class="[
          disabledSteps?.includes(step)
            ? 'bg-neutral-50 text-neutral-400 border border-dashed border-neutral-300 dark:bg-neutral-900 dark:border-neutral-700'
            : step === currentStep
              ? 'bg-primary-600 text-white'
              : step < currentStep
                ? 'bg-primary-100 text-primary-600'
                : 'bg-neutral-100 text-neutral-600 dark:bg-neutral-800',
          'dark:border dark:border-neutral-700',
        ]"
      >
        {{ step }}
      </div>
      <div
        v-if="step < totalSteps"
        class="w-16 transition-colors relative"
        :class="[
          disabledSteps?.includes(step) || disabledSteps?.includes(step + 1)
            ? 'h-[2px]'
            : 'h-0.5',
        ]"
      >
        <div
          class="absolute inset-0"
          :class="[
            disabledSteps?.includes(step) || disabledSteps?.includes(step + 1)
              ? 'border-t border-dashed border-neutral-300 dark:border-neutral-700'
              : step < currentStep
                ? 'bg-primary-600'
                : 'bg-neutral-200 dark:bg-neutral-700',
          ]"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  currentStep: number;
  totalSteps: number;
  disabledSteps?: number[];
}>();
</script>
