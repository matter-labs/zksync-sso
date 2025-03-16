<template>
  <CommonStepper
    :current-step="currentStep"
    :total-steps="4"
  />
  <h1 class="text-3xl font-medium text-neutral-900 dark:text-neutral-100">
    {{ stepTitle }}
  </h1>
  <div
    class="gap-4 flex-1 flex flex-col justify-center items-center max-w-md w-full"
  >
    <Step1
      v-if="currentStep === 1"
      @done="step1Done"
    />
    <Step2
      v-if="currentStep === 2"
      @done="step2Done"
    />
    <Step3
      v-if="currentStep === 3"
      @done="step3Done"
    />
    <Step4
      v-if="currentStep === 4 && digest !== null && sub !== null"
      :salt="digest.salt.toHex()"
      :sub="sub"
    />
  </div>
</template>

<script setup lang="ts">
import type { Address, Hex } from "viem";
import type { OidcDigest } from "zksync-sso-circuits";

import Step1 from "~/components/account-recovery/oidc-recovery-flow/Step1.vue";
import Step2 from "~/components/account-recovery/oidc-recovery-flow/Step2.vue";
import Step3 from "~/components/account-recovery/oidc-recovery-flow/Step3.vue";
import Step4 from "~/components/account-recovery/oidc-recovery-flow/Step4.vue";

type PasskeyData = {
  credentialId: Hex;
  passkeyPubKey: [Hex, Hex];
};

const currentStep = ref(1);
const stepTitle = ref("Start Recovery");
const userAddress = ref<Address | null>(null);
const digest = ref<OidcDigest | null>(null);
const sub = ref<string | null>(null);
const passkey = ref<PasskeyData | null>(null);

watchEffect(() => {
  switch (currentStep.value) {
    case 1:
      stepTitle.value = "Connect your wallet";
      return;
    case 2:
      stepTitle.value = "Log in with google";
      return;
    case 3:
      stepTitle.value = "Provide new passkey";
      return;
    case 4:
      stepTitle.value = "Perform tx";
      return;
    default:
      throw new Error(`Unknown step: ${currentStep.value}`);
  }
});

function step1Done() {
  currentStep.value = 2;
}

function step2Done(newAddress: Address, newDigest: OidcDigest, newSub: string) {
  userAddress.value = newAddress;
  digest.value = newDigest;
  sub.value = newSub;
  currentStep.value = 3;
}

function step3Done(newPasskey: PasskeyData) {
  passkey.value = newPasskey;
  currentStep.value = 4;
}
</script>
