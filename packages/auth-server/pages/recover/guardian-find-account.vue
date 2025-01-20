<template>
  <div class="min-h-screen">
    <header class="max-w-[1920px] mx-auto mb-12">
      <app-generic-nav />
    </header>
    <main class="max-w-[900px] mx-auto flex flex-col gap-6">
      <CommonStepper
        :current-step="currentStep"
        :total-steps="4"
      />

      <div class="flex flex-col items-center gap-8 mt-4">
        <h1 class="text-3xl font-medium text-neutral-900 dark:text-neutral-100">
          {{ stepTitle }}
        </h1>

        <!-- Step 1: Input Guardian Address -->
        <div
          v-if="currentStep === 1"
          class="w-full max-w-md flex flex-col gap-6"
        >
          <div class="flex flex-col gap-2">
            <label
              for="guardianAddress"
              class="text-sm text-neutral-700 dark:text-neutral-300"
            >
              Insert your guardian address
            </label>
            <ZkInput
              id="guardianAddress"
              v-model="guardianAddress"
              placeholder="0x..."
              :error="!!guardianAddressError"
              :messages="guardianAddressError ? [guardianAddressError] : undefined"
              @input="validateGuardianAddress"
            />
          </div>

          <ZkLink
            class="text-sm text-center w-fit text-neutral-700 dark:text-neutral-300 hover:text-neutral-900 dark:hover:text-neutral-100 transition-colors hover:border-b-neutral-900 dark:hover:border-b-neutral-100"
            href="/recover/guardian-unknown-account"
          >
            I don't remember my address or the guardian address
          </ZkLink>

          <ZkButton
            class="w-full"
            :disabled="!isValidGuardianAddress"
            @click="handleGuardianContinue"
          >
            Continue
          </ZkButton>
        </div>

        <!-- Step 2: Choose Account Address -->
        <div
          v-if="currentStep === 2"
          class="w-full max-w-md flex flex-col gap-6"
        >
          <div class="flex flex-col gap-2">
            <label
              for="accountSelect"
              class="text-sm text-neutral-700 dark:text-neutral-300"
            >
              Choose the account you want to recover
            </label>
            <account-recovery-account-select
              id="accountSelect"
              v-model="address"
              :accounts="accounts"
              :error="!!addressError"
              :messages="addressError ? ['Please select a valid account address'] : undefined"
              :disabled="isLoadingAccounts"
              @update:model-value="validateAddress"
            />
          </div>

          <div class="flex flex-col gap-4">
            <ZkButton
              class="w-full"
              :disabled="!isValidAddress"
              @click="handleAccountContinue"
            >
              Continue
            </ZkButton>

            <ZkButton
              type="secondary"
              class="w-full"
              @click="currentStep = 1"
            >
              Back
            </ZkButton>
          </div>
        </div>

        <account-recovery-passkey-generation-flow
          v-if="currentStep >= 3"
          v-model:current-step="currentStep"
          v-model:new-passkey="newPasskey"
          :generate-passkeys-step="3"
          :confirmation-step="4"
          :address="address"
          :register-in-progress="registerInProgress"
          @back="currentStep = 2"
        />
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { RegisterNewPasskeyReturnType } from "zksync-sso/client/passkey";

import { AddressSchema } from "~/utils/schemas";

definePageMeta({
  layout: "dashboard",
});

interface Account {
  address: string;
}

const currentStep = ref(1);
const guardianAddress = ref("");
const guardianAddressError = ref("");
const isValidGuardianAddress = ref(false);
const address = ref("");
const addressError = ref("");
const isValidAddress = ref(false);
const newPasskey = ref<RegisterNewPasskeyReturnType | null>(null);
const accounts = ref<Account[]>([]);
const isLoadingAccounts = ref(false);

const { inProgress: registerInProgress } = usePasskeyRegister();

const stepTitle = computed(() => {
  switch (currentStep.value) {
    case 1:
      return "Enter Guardian Address";
    case 2:
      return "Select Your Account";
    case 3:
      return "Generate Passkeys";
    case 4:
      return "Recovery Started";
    default:
      return "";
  }
});

const validateGuardianAddress = () => {
  const result = AddressSchema.safeParse(guardianAddress.value);
  if (result.success) {
    guardianAddressError.value = "";
    isValidGuardianAddress.value = true;
  } else {
    guardianAddressError.value = "Not a valid address";
    isValidGuardianAddress.value = false;
  }
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

const handleGuardianContinue = () => {
  if (isValidGuardianAddress.value) {
    currentStep.value = 2;
  }
};

const handleAccountContinue = () => {
  if (isValidAddress.value) {
    currentStep.value = 3;
  }
};

watch(isValidGuardianAddress, async () => {
  if (isValidGuardianAddress.value) {
    accounts.value = [
      { address: "0x71e6dDfE9074786Fd8e986C53f78D25450d614D5" },
      { address: "0x71e6dDfE9074786Fd8e986C53f78D25450d614D5" },
      { address: "0x71e6dDfE9074786Fd8e986C53f78D25450d614D5" },
    ];
  } else {
    accounts.value = [];
  }
});
</script>
