<template>
  <div class="flex flex-col gap-8 flex-1">
    <CommonStepper
      :current-step="currentStep"
      :total-steps="5"
      :disabled-steps="isConfirmLater ? [5] : undefined"
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
          @next="currentStep++"
          @back="$emit('back')"
        />
        <Step2
          v-if="currentStep === 2"
          v-model="guardianAddress"
          @next="currentStep++"
          @back="currentStep--"
        />
        <Step3
          v-if="currentStep === 3"
          @confirm-now="handleConfirmNow"
          @confirm-later="handleConfirmLater"
        />
        <Step4ConfirmNow
          v-if="currentStep === 4 && !isConfirmLater"
          :guardian-address="guardianAddress"
          @next="currentStep++"
          @back="currentStep--"
        />
        <Step4ConfirmLater
          v-if="currentStep === 4 && isConfirmLater"
          :guardian-address="guardianAddress"
          @next="completeSetup"
        />
        <Step5
          v-if="currentStep === 5"
          @next="completeSetup"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Address } from "viem";
import { ref } from "vue";

import Step1 from "./Step1.vue";
import Step2 from "./Step2.vue";
import Step3 from "./Step3.vue";
import Step4ConfirmLater from "./Step4ConfirmLater.vue";
import Step4ConfirmNow from "./Step4ConfirmNow.vue";
import Step5 from "./Step5.vue";

const guardianAddress = ref("" as Address);

const currentStep = ref(1);
const isConfirmLater = ref(false);

const stepTitle = computed(() => {
  switch (currentStep.value) {
    case 1:
      return "Guardian Recovery";
    case 2:
      return "Insert Guardian Address";
    case 3:
      return "Confirm Guardian";
    case 4:
      return isConfirmLater.value ? "Save Recovery URL" : "Connect Guardian Account";
    case 5:
      return "Guardian Confirmed";
    default:
      return "";
  }
});

const handleConfirmNow = () => {
  isConfirmLater.value = false;
  currentStep.value++;
};

const handleConfirmLater = () => {
  isConfirmLater.value = true;
  currentStep.value++;
};

function completeSetup() {
  props.closeModal();
}

const props = defineProps<{
  closeModal: () => void;
}>();

defineEmits<{
  (e: "back"): void;
}>();
</script>
