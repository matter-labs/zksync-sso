<template>
  <p class="text-center text-neutral-700 dark:text-neutral-300">
    Please provide a new passkey to associate to your account
  </p>
  <ZkButton
    class="w-full"
    @click="go"
  >
    Provide new passkey
  </ZkButton>
</template>

<script setup lang="ts">
import { bytesToHex, type Hex, toHex } from "viem";
import { base64urlToUint8Array, getPublicKeyBytesFromPasskeySignature } from "zksync-sso-4337/utils";

const emit = defineEmits<{
  (e: "done", passkey: PasskeyData): void;
}>();

const { registerPasskey } = usePasskeyRegister();

async function go() {
  const passkeyData = await getNewPasskey();
  emit("done", passkeyData);
}

type PasskeyData = {
  credentialId: Hex;
  passkeyPubKey: [Hex, Hex];
};

async function getNewPasskey(): Promise<PasskeyData> {
  const result = await registerPasskey();
  if (!result) {
    throw new Error("Failed to register passkey");
  }
  const { credentialPublicKey, credentialId } = result;

  const [buf1, buf2] = getPublicKeyBytesFromPasskeySignature(credentialPublicKey);

  if (buf1 === undefined || buf2 === undefined) {
    throw new Error("Could not recover passkey");
  }

  return {
    credentialId: toHex(base64urlToUint8Array(credentialId)),
    passkeyPubKey: [bytesToHex(buf1), bytesToHex(buf2)],
  };
}
</script>
