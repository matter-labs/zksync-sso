<template>
  <div
    class="gap-4 flex-1 flex flex-col justify-center items-center max-w-md w-full"
  >
    <Step1
      v-if="currentStep === 1"
      :request="request"
      @sign="signWcPersonal"
    />
  </div>
</template>

<script setup lang="ts">
import type { WalletKitTypes } from "@reown/walletkit";
import { ref } from "vue";

import Step1 from "./Step1.vue";

const { signPersonal } = useWalletConnectStore();
const stepTitle = defineModel<string>("stepTitle", { required: true });
const { request } = defineProps<{ request: WalletKitTypes.SessionRequest }>();

const currentStep = ref(1);
watchEffect(() => {
  switch (currentStep.value) {
    case 1:
      stepTitle.value = "Review Signature Parameters";
      break;
  }
});

const signWcPersonal = async () => {
  await signPersonal(request);
  currentStep.value++;
};
</script>
