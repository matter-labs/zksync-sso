<template>
  <p class="text-center text-gray-600 dark:text-gray-400">
    Input the address of the account you want to add as your guardian
  </p>
  <ZkInput
    v-model="guardianAddress"
    :error="!!addressError"
    :messages="addressError ? [addressError] : undefined"
    data-testid="guardian-address-input"
    placeholder="0x..."
    class="w-full text-left"
    @input="validateAddress"
  />
  <p
    v-if="proposeGuardianError"
    class="text-center text-red-500 dark:text-red-400"
  >
    An error occurred while proposing the guardian. Please try again.
  </p>
  <ZkButton
    type="primary"
    :disabled="!isValidAddress"
    class="w-full md:max-w-48"
    :loading="proposeGuardianInProgress"
    @click="proposeGuardian"
  >
    Propose Guardian
  </ZkButton>
  <ZkButton
    type="secondary"
    class="w-full md:max-w-48"
    :disabled="proposeGuardianInProgress"
    @click="emit('back')"
  >
    Back
  </ZkButton>
</template>

<script setup lang="ts">
import type { Address } from "viem";

import { AddressSchema } from "~/utils/schemas";

const addressError = ref("");
const isValidAddress = ref(false);
const { proposeGuardian: proposeGuardianAction, proposeGuardianInProgress, proposeGuardianError } = useRecoveryGuardian();

const validateAddress = () => {
  const result = AddressSchema.safeParse(guardianAddress.value);
  if (result.success) {
    addressError.value = "";
    isValidAddress.value = true;
  } else {
    addressError.value = "Not a valid address";
    isValidAddress.value = false;
  }
};

const proposeGuardian = async () => {
  if (!guardianAddress.value) {
    addressError.value = "Address is required";
    isValidAddress.value = false;
    return;
  }

  await proposeGuardianAction(guardianAddress.value);
  emit("next");
};

const guardianAddress = defineModel<Address>();
const emit = defineEmits<{
  (e: "next" | "back"): void;
}>();
</script>
