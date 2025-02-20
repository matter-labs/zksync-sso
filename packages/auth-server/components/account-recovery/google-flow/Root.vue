<template>
  <div class="flex flex-col gap-8 flex-1">
    <CommonStepper
      :current-step="currentStep"
      :total-steps="2"
    />

    <div class="flex flex-col items-center gap-4 mt-4">
      <h2 class="text-2xl font-medium text-center text-gray-900 dark:text-white">
        {{ stepTitle }}
      </h2>

      <div
        class="gap-4 flex-1 flex flex-col justify-center items-center max-w-lg"
      >
        <Step1
          v-if="currentStep === 1"
          @next="onFinishStep1"
          @back="$emit('back')"
        />
        <Step2
          v-if="currentStep === 2"
          ref="step2Ref"
          :jwt="jwt"
          @finish="completeSetup"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import type { JWT } from "zksync-sso-circuits";

import Step1 from "./Step1.vue";
import Step2 from "./Step2.vue";

const currentStep = ref(1);
const jwt = ref<JWT | null>(null);

const stepTitle = computed(() => {
  switch (currentStep.value) {
    case 1:
      return "Google Recovery";
    case 2:
      return "Linking google account...";
    default:
      return "";
  }
});

function completeSetup() {
  props.closeModal();
}

function onFinishStep1(newJwt: JWT): void {
  jwt.value = newJwt!;
  currentStep.value = 2;
}

const props = defineProps<{
  closeModal: () => void;
}>();

defineEmits<{
  (e: "back"): void;
}>();
</script>
