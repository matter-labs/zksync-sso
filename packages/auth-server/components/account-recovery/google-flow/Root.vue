<template>
  <div class="flex flex-col gap-8 flex-1">
    <CommonStepper
      :current-step="currentStep"
      :total-steps="3"
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
          v-if="currentStep === 2 && jwt !== null"
          ref="step2Ref"
          :jwt="jwt"
          @next="onFinishStep2"
        />
        <Step3
          v-if="currentStep === 3"
          @finish="onFinishStep3"
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
import Step3 from "./Step3.vue";

const currentStep = ref(1);
const jwt = ref<JWT | null>(null);

const props = defineProps<{
  closeModal: () => void;
}>();

defineEmits<{
  (e: "back"): void;
}>();

const stepTitle = computed(() => {
  switch (currentStep.value) {
    case 1:
      return "Google Recovery";
    case 2:
      return "Confirm account";
    case 3:
      return "Recovery finished";
    default:
      throw new Error(`Unknown step: ${currentStep.value}`);
  }
});

function onFinishStep1(newJwt: JWT): void {
  jwt.value = newJwt!;
  currentStep.value = 2;
}

function onFinishStep2(): void {
  currentStep.value = 3;
}

function onFinishStep3(): void {
  props.closeModal();
}
</script>
