<template>
  <div
    v-if="currentStep === 'info'"
    class="gap-4 flex-1 flex flex-col justify-center items-center"
  >
    <p class="text-gray-600 px-4">
      Guardian recovery allows you to designate trusted contacts who can help you
      recover your account if you lose access.
    </p>
    <div class="flex space-x-3">
      <Button
        type="primary"
        class="w-fit"
        @click="currentStep = 'add-guardian'"
      >
        Continue
      </Button>
      <Button
        type="secondary"
        class="w-fit"
        @click="$emit('back')"
      >
        Back
      </Button>
    </div>
  </div>

  <div
    v-else-if="currentStep === 'add-guardian'"
    class="flex flex-col gap-4 flex-1 text-left justify-center px-6"
  >
    <p>Insert address</p>
    <Input />
    <div class="flex gap-3">
      <Button @click="currentStep = 'confirm'">
        Continue
      </Button>
      <Button
        type="secondary"
        @click="currentStep = 'info'"
      >
        Back
      </Button>
    </div>
  </div>

  <div
    v-else-if="currentStep === 'confirm'"
    class="flex flex-col justify-between flex-1 text-left px-6"
  >
    <p>Your recovery address was saved. Please use this url to confirm the recovery method:</p>
    <Link
      href="https://auth-test.zksync.dev/dashboard/0x1234567890"
      class="w-fit mx-auto"
      target="_blank"
    >
      https://auth-test.zksync.dev/dashboard/0x1234567890
    </Link>
    <Button @click="completeSetup">
      Close
    </Button>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";

import Button from "~/components/zk/button.vue";
import Input from "~/components/zk/input.vue";
import Link from "~/components/zk/link.vue";

type GuardianStep = "info" | "add-guardian" | "confirm";
const currentStep = ref<GuardianStep>("info");

const props = defineProps<{
  closeModal: () => void;
}>();

defineEmits<{
  (e: "back"): void;
}>();

function completeSetup() {
  props.closeModal();
}
</script>
