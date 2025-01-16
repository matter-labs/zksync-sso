<template>
  <Dialog
    ref="modalRef"
    content-class="min-w-[700px] min-h-[500px]"
    description-class="flex-1 mb-0 flex"
    close-class="h-8 max-h-8"
    :title="title"
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
      class="space-y-4 text-left flex-1 flex flex-col"
    >
      <p class="text-gray-600 mb-6">
        Choose a recovery method for your account:
      </p>
      <div class="flex flex-col gap-5 items-center flex-1 justify-center">
        <Button
          class="w-64"
          @click="selectMethod('guardian')"
        >
          <div class="flex items-center justify-between gap-2">
            <UserIcon class="w-5 h-5" />
            <span>Guardian Recovery</span>
          </div>
        </Button>
        <div class="flex flex-col gap-2">
          <Button
            disabled
            class="w-64"
          >
            <div class="flex items-center justify-between gap-2">
              <EnvelopeIcon class="w-5 h-5" />
              <span>Email Recovery</span>
            </div>
          </Button>
          <span class="text-sm text-gray-500 text-center">
            Coming soon...
          </span>
        </div>
      </div>
    </div>

    <GuardianFlow
      v-if="currentStep === 'guardian'"
      :close-modal="closeModal"
      @back="currentStep = 'select-method'"
    />
  </Dialog>
</template>

<script setup lang="ts">
import { EnvelopeIcon, UserIcon } from "@heroicons/vue/24/solid";
import { ref } from "vue";

import GuardianFlow from "~/components/account-recovery/flows/GuardianFlow.vue";
import Button from "~/components/zk/button.vue";
import Dialog from "~/components/zk/dialog.vue";

type Step = "select-method" | "guardian" | "email";
const currentStep = ref<Step>("select-method");
const modalRef = ref<InstanceType<typeof Dialog>>();

function closeModal() {
  modalRef.value?.close();
}

const title = computed(() => {
  switch (currentStep.value) {
    case "select-method":
      return "Add Recovery Method";
    case "guardian":
      return "Guardian Recovery Setup";
    case "email":
      return "Email Recovery Setup";
    default:
      throw new Error("Invalid step");
  }
});

function selectMethod(method: "guardian" | "email") {
  currentStep.value = method;
}
</script>
