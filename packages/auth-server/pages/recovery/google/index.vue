<template>
  <div class="min-h-screen">
    <header class="max-w-[1920px] mx-auto mb-12">
      <app-generic-nav />
    </header>
    <main class="max-w-[900px] mx-auto flex flex-col gap-6">
      <CommonStepper
        :current-step="currentStep"
        :total-steps="3"
      />

      <div class="flex flex-col items-center gap-8 mt-4">
        <h1 class="text-3xl font-medium text-neutral-900 dark:text-neutral-100">
          {{ stepTitle }}
        </h1>
        <div
          v-if="currentStep === 1"
          class="w-full max-w-md flex flex-col gap-6"
        >
          <ZkButton
            class="w-full"
            @click="generateProf"
          >
            Test
          </ZkButton>
        </div>

        <account-recovery-passkey-generation-flow
          v-if="currentStep >= 2"
          v-model:current-step="currentStep"
          v-model:new-passkey="newPasskey"
          :generate-passkeys-step="2"
          :confirmation-step="3"
          :address="address"
          :register-in-progress="registerInProgress"
          @back="currentStep = 1"
        />
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";

definePageMeta({
  layout: "dashboard",
});

const currentStep = ref(1);

const stepTitle = computed(() => {
  switch (currentStep.value) {
    case 1:
      return "Test oidc recovery";
    default:
      throw new Error("Unknown error");
  }
});

async function generateProf(): Promise<void> {
  throw new Error("Not implemented");
}
</script>
