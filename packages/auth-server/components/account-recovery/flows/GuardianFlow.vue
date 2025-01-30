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
        v-if="currentStep === 1"
        class="gap-4 flex-1 flex flex-col justify-center items-center"
      >
        <p class="text-center text-gray-600 dark:text-gray-400 px-6">
          Guardian recovery allows you to designate trusted contacts who can help you
          recover your account if you lose access.
        </p>
        <ZkButton
          type="primary"
          class="w-full max-w-xs mt-2"
          @click="currentStep = 2"
        >
          Continue
        </ZkButton>
        <ZkButton
          type="secondary"
          class="w-full max-w-xs"
          @click="$emit('back')"
        >
          Back
        </ZkButton>
      </div>

      <div
        v-if="currentStep === 2"
        class="flex flex-col gap-4 flex-1 text-left justify-center items-center"
      >
        <p class="text-center text-gray-600 dark:text-gray-400">
          Enter the wallet address of your trusted guardian
        </p>
        <ZkInput
          v-model="guardianAddress"
          placeholder="0x..."
          :error="!!guardianAddressError"
          :messages="guardianAddressError ? [guardianAddressError] : undefined"
          @input="validateAddress"
        />
        <ZkButton
          type="primary"
          :disabled="!guardianAddress || !isValidGuardianAddress"
          class="w-full max-w-xs mt-2"
          :loading="proposeGuardianInProgress"
          @click="proposeGuardian()"
        >
          Continue
        </ZkButton>
        <ZkButton
          type="secondary"
          class="w-full max-w-xs"
          @click="currentStep = 1"
        >
          Back
        </ZkButton>
      </div>

      <div
        v-if="currentStep === 3"
        class="flex flex-col gap-4 flex-1 justify-center items-center"
      >
        <p class="text-center text-gray-600 dark:text-gray-400 px-6">
          Your recovery address was saved. Please use this url to confirm the recovery method:
        </p>
        <div class="flex items-center">
          <a
            :href="recoveryUrl"
            class="max-w-md truncate underline"
            target="_blank"
          >
            {{ recoveryUrl }}
          </a>
          <common-copy-to-clipboard
            :text="recoveryUrl"
          />
        </div>
        <ZkButton
          type="primary"
          class="mt-2 w-full max-w-xs"
          @click="completeSetup"
        >
          Close
        </ZkButton>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Address } from "viem";
import { ref } from "vue";

const currentStep = ref(1);

const stepTitle = computed(() => {
  switch (currentStep.value) {
    case 1:
      return "Guardian Recovery";
    case 2:
      return "Add Guardian";
    case 3:
      return "Confirm Guardian";
    default:
      return "";
  }
});
const { proposeGuardian: proposeGuardianAction, proposeGuardianInProgress } = useRecoveryGuardian();
const config = useRuntimeConfig();
const { address } = useAccountStore();

const guardianAddress = ref("" as Address);
const guardianAddressError = ref("");
const isValidGuardianAddress = ref(false);

const recoveryUrl = computed(() => {
  return `${config.public.appUrl}/recovery/guardian/confirm-guardian?accountAddress=${address}&guardianAddress=${guardianAddress.value}`;
});

const props = defineProps<{
  closeModal: () => void;
}>();

defineEmits<{
  (e: "back"): void;
}>();

function completeSetup() {
  props.closeModal();
}

const proposeGuardian = async () => {
  await proposeGuardianAction(guardianAddress.value);
  currentStep.value = 3;
};

const validateAddress = () => {
  const result = AddressSchema.safeParse(guardianAddress.value);
  if (result.success) {
    guardianAddressError.value = "";
    isValidGuardianAddress.value = true;
  } else {
    guardianAddressError.value = "Not a valid address";
    isValidGuardianAddress.value = false;
  }
};
</script>
