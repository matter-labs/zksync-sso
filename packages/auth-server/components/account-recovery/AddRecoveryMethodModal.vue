<template>
  <Dialog
    ref="modalRef"
    content-class="min-w-[700px] min-h-[500px]"
    description-class="flex-1 mb-0 flex text-base"
    close-class="h-8 max-h-8"
    :title="title"
    @close="onModalClosed()"
  >
    <template #trigger>
      <slot name="trigger">
        <Button
          class="w-full lg:w-auto"
          type="primary"
        >
          Add Recovery Method
        </Button>
      </slot>
    </template>

    <template #submit>
      <div />
    </template>

    <template #cancel>
      <div />
    </template>

    <!-- Method Selection Step -->
    <div
      v-if="currentStep === 'select-method'"
      class="space-y-6 text-left flex-1 flex flex-col"
    >
      <div class="flex flex-col gap-6 items-center flex-1 justify-center max-w-md mx-auto w-full">
        <div class="text-center">
          <p class="text-xl font-medium mb-2">
            Choose a Recovery Method
          </p>
          <p class="text-base text-gray-600 dark:text-gray-400">
            Select how you'd like to recover your account if you lose access
          </p>
        </div>

        <div class="flex flex-col gap-5 w-full max-w-xs">
          <Button
            class="w-full"
            @click="selectMethod('guardian')"
          >
            Recover with Guardian
          </Button>

          <Button
            class="w-full"
            @click="selectMethod('google')"
            v-if="!oidcEnabled"
          >
            Recover with Google
          </Button>
        </div>
      </div>
    </div>

    <GuardianFlow
      v-if="currentStep === 'guardian'"
      :close-modal="closeModal"
      @back="currentStep = 'select-method'"
    />
    <GoogleFlow
      v-if="currentStep === 'google'"
      :close-modal="closeModal"
      @back="currentStep = 'select-method'"
    />
  </Dialog>
</template>

<script setup lang="ts">
import { ref } from "vue";

import GoogleFlow from "~/components/account-recovery/google-flow/Root.vue";
import GuardianFlow from "~/components/account-recovery/guardian-flow/Root.vue";
import Button from "~/components/zk/button.vue";
import Dialog from "~/components/zk/dialog.vue";

const { enabled: oidcEnabled } = useOidcConfig();

type Step = "select-method" | "guardian" | "google";
const currentStep = ref<Step>("select-method");
const modalRef = ref<InstanceType<typeof Dialog>>();

const emit = defineEmits<{
  (e: "closed"): void;
}>();

function onModalClosed() {
  emit("closed");
}

function closeModal() {
  modalRef.value?.close();
}

const title = computed(() => {
  switch (currentStep.value) {
    case "select-method":
      return "Add Recovery Method";
    case "guardian":
      return "Guardian Recovery Setup";
    case "google":
      return "Google Recovery Setup";
    default:
      throw new Error("Invalid step");
  }
});

function selectMethod(method: "guardian" | "google") {
  currentStep.value = method;
}
</script>
