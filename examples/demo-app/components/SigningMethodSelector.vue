<template>
  <div class="mb-3 p-3 bg-white rounded border border-indigo-300">
    <label class="block text-sm font-medium mb-2">Signing Method:</label>
    <div class="space-y-2">
      <label class="flex items-center">
        <input
          :checked="modelValue === 'eoa'"
          type="radio"
          value="eoa"
          class="mr-2"
          @change="$emit('update:modelValue', 'eoa')"
        >
        <span class="text-sm">EOA Validator (Private Key)</span>
      </label>
      <label class="flex items-center">
        <input
          :checked="modelValue === 'passkey'"
          type="radio"
          value="passkey"
          class="mr-2"
          @change="$emit('update:modelValue', 'passkey')"
        >
        <span class="text-sm">WebAuthn Passkey (Hardware Key)</span>
      </label>
    </div>
  </div>
</template>

<script setup>
// Props
defineProps({
  modelValue: {
    type: String,
    required: true,
    validator: (value) => ["eoa", "passkey"].includes(value),
  },
});

// Emits
defineEmits(["update:modelValue"]);
</script>
