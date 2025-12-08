<template>
  <p class="text-center text-neutral-700 dark:text-neutral-300">
    Generate new passkeys to secure your account
  </p>

  <ZkButton
    class="w-full mt-4"
    :loading="inProgress"
    @click="handleGeneratePasskeys"
  >
    Generate Passkeys
  </ZkButton>

  <ZkButton
    type="secondary"
    class="w-full"
    @click="emit('back')"
  >
    Back
  </ZkButton>
</template>

<script setup lang="ts">
import type { RegisterNewPasskeyReturnType } from "~/composables/usePasskeyRegister";

const { registerPasskey, inProgress } = usePasskeyRegister();

const emit = defineEmits<{
  (e: "back"): void;
  (e: "savePasskey", passkey: RegisterNewPasskeyReturnType): void;
}>();

const handleGeneratePasskeys = async () => {
  const result = await registerPasskey();
  if (!result) {
    throw new Error("Failed to register passkey");
  }
  emit("savePasskey", result);
};
</script>
