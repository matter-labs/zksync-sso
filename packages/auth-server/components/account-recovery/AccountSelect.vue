<template>
  <div class="relative">
    <select
      :id="id"
      v-model="selectedValue"
      class="w-full px-4 py-3 bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-zk text-neutral-900 dark:text-neutral-100 appearance-none cursor-pointer focus:outline-none focus:ring-2 focus:ring-primary-500 dark:focus:ring-primary-400 disabled:opacity-50 disabled:cursor-not-allowed truncate pr-8"
      :class="{
        'border-error-500 dark:border-error-400': error,
      }"
      :disabled="disabled || !accounts.length"
    >
      <option
        value=""
        disabled
      >
        {{ accounts.length ? 'Select an account' : 'No accounts found' }}
      </option>
      <option
        v-for="account in accounts"
        :key="account"
        :value="account"
      >
        {{ account }}
      </option>
    </select>

    <div class="absolute inset-y-0 right-0 flex items-center px-4 pointer-events-none">
      <ZkIcon icon="arrow_drop_down" />
    </div>

    <!-- Error messages -->
    <div
      v-if="error && messages?.length"
      class="mt-2 space-y-1"
    >
      <p
        v-for="(message, index) in messages"
        :key="index"
        class="text-sm text-error-500 dark:text-error-400"
      >
        {{ message }}
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Address } from "viem";
import { computed } from "vue";

const props = defineProps<{
  id?: string;
  modelValue: string;
  accounts: Address[];
  error?: boolean;
  messages?: string[];
  disabled?: boolean;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", value: string): void;
}>();

const selectedValue = computed({
  get: () => props.modelValue,
  set: (value) => emit("update:modelValue", value),
});
</script>
