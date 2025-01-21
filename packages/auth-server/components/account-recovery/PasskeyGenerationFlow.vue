<template>
  <div class="w-full max-w-md flex flex-col gap-6">
    <!-- Generate Passkeys Step -->
    <div
      v-if="currentStep === generatePasskeysStep"
      class="w-full max-w-md flex flex-col gap-6"
    >
      <p class="text-center text-neutral-700 dark:text-neutral-300">
        Generate new passkeys to secure your account
      </p>

      <ZkButton
        class="w-full"
        :loading="registerInProgress"
        @click="handleGeneratePasskeys"
      >
        Generate Passkeys
      </ZkButton>

      <ZkButton
        type="secondary"
        class="w-full"
        @click="$emit('back')"
      >
        Back
      </ZkButton>
    </div>

    <!-- Confirmation Step -->
    <div
      v-if="currentStep === confirmationStep"
      class="w-full max-w-md flex flex-col gap-6"
    >
      <div class="flex flex-col gap-4 text-center text-neutral-700 dark:text-neutral-300">
        <p>
          Your passkeys have been generated successfully.
        </p>
        <p>
          Please share the following url with your guardian to complete the recovery process:
        </p>
      </div>

      <div class="w-full  items-center gap-2 p-4 bg-neutral-100 dark:bg-neutral-900 rounded-zk">
        <a
          :href="recoveryUrl"
          target="_blank"
          class="text-sm text-neutral-800 dark:text-neutral-100 break-all hover:text-neutral-900 dark:hover:text-neutral-400 leading-relaxed underline underline-offset-4 decoration-neutral-400 hover:decoration-neutral-900 dark:decoration-neutral-600 dark:hover:decoration-neutral-400"
        >
          {{ recoveryUrl }}
        </a>
        <common-copy-to-clipboard
          :text="recoveryUrl ?? ''"
          class="!inline-flex ml-1"
        />
      </div>

      <p class="text-sm text-center text-neutral-600 dark:text-neutral-400">
        You'll be able to access your account once your guardian confirms the recovery.
      </p>

      <ZkLink
        type="primary"
        href="/"
        class="w-full"
      >
        Back to Home
      </ZkLink>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { RegisterNewPasskeyReturnType } from "zksync-sso/client/passkey";

const props = defineProps<{
  currentStep: number;
  generatePasskeysStep: number;
  confirmationStep: number;
  address: string;
  newPasskey: RegisterNewPasskeyReturnType | null;
  registerInProgress: boolean;
}>();

const emit = defineEmits<{
  (e: "back"): void;
  (e: "update:newPasskey", value: RegisterNewPasskeyReturnType): void;
  (e: "update:currentStep", value: number): void;
}>();

const runtimeConfig = useRuntimeConfig();
const appUrl = runtimeConfig.public.appUrl;

const { registerPasskey } = usePasskeyRegister();

const recoveryUrl = computedAsync(async () => {
  const queryParams = new URLSearchParams();

  const credentialId = props.newPasskey?.credentialId ?? "";
  const credentialPublicKey = uint8ArrayToHex(props.newPasskey?.credentialPublicKey ?? new Uint8Array()) ?? "";

  queryParams.set("credentialId", credentialId);
  queryParams.set("credentialPublicKey", credentialPublicKey);
  queryParams.set("accountAddress", props.address);

  // Create checksum from concatenated credential data
  const dataToHash = `${props.address}:${credentialId}:${credentialPublicKey}`;
  const fullHash = new Uint8Array(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(dataToHash)));
  const shortHash = fullHash.slice(0, 8); // Take first 8 bytes of the hash
  const checksum = uint8ArrayToHex(shortHash);

  queryParams.set("checksum", checksum);

  return `${appUrl}/recovery/guardian/confirm-recovery?${queryParams.toString()}`;
});

const handleGeneratePasskeys = async () => {
  const result = await registerPasskey();
  if (!result) {
    throw new Error("Failed to register passkey");
  }
  emit("update:newPasskey", result);
  emit("update:currentStep", props.confirmationStep);
};
</script>
