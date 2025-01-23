<template>
  <div class="flex items-center justify-center gap-2">
    <div
      v-for="step in totalSteps"
      :key="step"
      class="flex items-center"
    >
      <div
        class="w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium"
        :class="[
          step === currentStep
            ? 'bg-primary-600 text-white'
            : step < currentStep
              ? 'bg-primary-100 text-primary-600'
              : 'bg-neutral-100 text-neutral-600',
          'dark:border dark:border-neutral-700',
        ]"
      >
        {{ step }}
      </div>
      <div
        v-if="step < totalSteps"
        class="w-16 h-0.5"
        :class="[
          step < currentStep
            ? 'bg-primary-600'
            : 'bg-neutral-200 dark:bg-neutral-700',
        ]"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  currentStep: number;
  totalSteps: number;
}>();
</script>
