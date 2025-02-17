<template>
  <div
    class="gap-4 flex-1 flex flex-col justify-center items-center max-w-md w-full"
  >
    <Step1
      v-if="relativeStep === 1"
      @back="currentStep--"
      @save-passkey="handleSavePasskey"
    />
    <Step2
      v-if="relativeStep === 2"
      @confirm-now="handleConfirmNow"
      @confirm-later="handleConfirmLater"
    />
    <Step3ConfirmNow
      v-if="relativeStep === 3 && !isConfirmLater"
      :account-address="props.accountAddress"
      :new-passkey="newPasskey!"
      @next="currentStep++"
      @back="currentStep--"
    />
    <Step3ConfirmLater
      v-if="relativeStep === 3 && isConfirmLater"
      :account-address="props.accountAddress"
      :new-passkey="newPasskey!"
      @back="currentStep--"
    />
    <Step4
      v-if="relativeStep === 4"
      @back="currentStep--"
    />
  </div>
</template>

<script setup lang="ts">
import type { Address } from "viem";
import { ref } from "vue";
import type { RegisterNewPasskeyReturnType } from "zksync-sso/client/passkey";

import Step1 from "./Step1.vue";
import Step2 from "./Step2.vue";
import Step3ConfirmLater from "./Step3ConfirmLater.vue";
import Step3ConfirmNow from "./Step3ConfirmNow.vue";
import Step4 from "./Step4.vue";

const props = defineProps<{
  startingStep: number;
  accountAddress: Address;
}>();
const stepTitle = defineModel<string>("stepTitle", { required: true });
const currentStep = defineModel<number>("currentStep", { required: true });
const disabledSteps = defineModel<number[]>("disabledSteps", { required: true });

const isConfirmLater = ref(false);
const relativeStep = computed(() => currentStep.value - props.startingStep + 1);
const newPasskey = ref<RegisterNewPasskeyReturnType | null>(null);

watchEffect(() => {
  switch (relativeStep.value) {
    case 1:
      stepTitle.value = "Generate Passkeys";
      break;
    case 2:
      stepTitle.value = "Recovery Started";
      break;
    case 3:
      stepTitle.value = "Confirm Recovery";
      break;
    case 4:
      stepTitle.value = isConfirmLater.value ? "Save Recovery URL" : "Connect Guardian Account";
      break;
    case 5:
      stepTitle.value = "Guardian Confirmed";
      break;
  }
});

const handleSavePasskey = (passkey: RegisterNewPasskeyReturnType) => {
  newPasskey.value = passkey;
  currentStep.value++;
};

const handleConfirmNow = () => {
  isConfirmLater.value = false;
  disabledSteps.value = [];
  currentStep.value++;
};

const handleConfirmLater = () => {
  isConfirmLater.value = true;
  disabledSteps.value = [props.startingStep + 4 - 1];
  currentStep.value++;
};
</script>
