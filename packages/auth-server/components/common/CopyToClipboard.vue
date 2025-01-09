<template>
  <button
    class="relative w-5 h-5 flex items-center justify-center"
    @click="copyToClipboard(text)"
  >
    <Transition name="fade">
      <span
        v-if="copied"
        class="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-2 py-1 text-xs text-white dark:text-gray-900 bg-gray-800 dark:bg-gray-100 rounded-2xl whitespace-nowrap"
      >
        Copied!
      </span>
    </Transition>
    <Transition
      name="scale"
      mode="out-in"
    >
      <CheckIcon
        v-if="copied"
        class="w-5 h-5 text-green-500 dark:text-green-400"
      />
      <DocumentDuplicateIcon
        v-else
        class="w-4 h-4"
      />
    </Transition>
  </button>
</template>

<script setup lang="ts">
import { CheckIcon, DocumentDuplicateIcon } from "@heroicons/vue/24/solid";
import { ref } from "vue";

const copied = ref(false);

const copyToClipboard = async (text: string) => {
  await navigator.clipboard.writeText(text);
  copied.value = true;

  setTimeout(() => {
    copied.value = false;
  }, 2000);
};

defineProps<{ text: string }>();
</script>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.scale-enter-active,
.scale-leave-active {
  transition: all 0.1s ease-out;
}

.scale-enter-from,
.scale-leave-to {
  transform: scale(0.95);
  opacity: 0;
}
</style>
