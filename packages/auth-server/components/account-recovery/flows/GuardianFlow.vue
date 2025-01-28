<template>
  <div class="flex flex-col gap-8 flex-1">
    <CommonStepper
      :current-step="currentStep"
      :total-steps="3"
    />

    <div
      v-else-if="currentStep === 'add-guardian'"
      class="flex flex-col gap-4 flex-1 text-left justify-center px-6"
    >
      <p>Insert address</p>

      <ZkInput
        id="address"
        v-model="address"
        placeholder="0x..."
        :error="!!addressError"
        :messages="addressError ? [addressError] : undefined"
        @input="validateAddress"
      />
      <div class="flex gap-3">
        <ZkButton
          :loading="proposeGuardianInProgress"
          @click="proposeGuardian()"
        >
          Continue
        </ZkButton>
        <ZkButton
          type="secondary"
          @click="currentStep = 'info'"
        >
          Back
        </ZkButton>
      </div>
    </div>

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
      />
      <ZkButton
        type="primary"
        :disabled="!guardianAddress"
        class="w-full max-w-xs mt-2"
        @click="currentStep = 3"
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
          href="http://localhost:3002/recovery/guardian/confirm-guardian?accountAddress=0x71e6dDfE9074786Fd8e986C53f78D25450d614D5&guardianAddress=0x71e6dDfE9074786Fd8e986C53f78D25450d614D5"
          class="max-w-md truncate underline"
          target="_blank"
        >
          http://localhost:3002/recovery/guardian/confirm-guardian?accountAddress=0x71e6dDfE9074786Fd8e986C53f78D25450d614D5&guardianAddress=0x71e6dDfE9074786Fd8e986C53f78D25450d614D5
        </a>
        <common-copy-to-clipboard
          class=""
          text="http://localhost:3002/recovery/guardian/confirm-guardian?accountAddress=0x71e6dDfE9074786Fd8e986C53f78D25450d614D5&guardianAddress=0x71e6dDfE9074786Fd8e986C53f78D25450d614D5"
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

import Button from "~/components/zk/button.vue";
import Link from "~/components/zk/link.vue";

type GuardianStep = "info" | "add-guardian" | "confirm";
const currentStep = ref<GuardianStep>("info");
const { proposeGuardian: proposeGuardianAction, proposeGuardianInProgress } = useRecoveryGuardian();

const address = ref("" as Address);
const addressError = ref("");
const isValidAddress = ref(false);

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
  await proposeGuardianAction(address.value);
  currentStep.value = "confirm";
};

const validateAddress = () => {
  const result = AddressSchema.safeParse(address.value);
  if (result.success) {
    addressError.value = "";
    isValidAddress.value = true;
  } else {
    addressError.value = "Not a valid address";
    isValidAddress.value = false;
  }
};
</script>
