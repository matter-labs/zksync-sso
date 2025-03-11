<template>
  <div
    v-if="addOidcAccountIsLoading"
    class="flex flex-col items-center gap-4"
  >
    <common-spinner class="w-8 h-8" />
    <p class="text-center text-gray-600 dark:text-gray-400">
      Please wait...
    </p>
  </div>

  <div
    v-else
    class="flex flex-col items-center justify-center h-full"
  >
    <p class="text-center text-gray-600 dark:text-gray-400">
      Your Google account has been linked and is ready to help you recover your account.
    </p>
    <ZkButton
      type="secondary"
      class="w-full md:max-w-48"
      @click="$emit('finish')"
    >
      Finish
    </ZkButton>
  </div>
</template>

<script setup lang="ts">
import type { OidcData } from "zksync-sso/client";
import { ByteVector, type JWT } from "zksync-sso-circuits";

import { useRecoveryOidc } from "~/composables/useRecoveryOidc";

defineEmits<{
  (e: "finish"): void;
}>();

const props = defineProps<{
  jwt: JWT | null;
}>();
const { addOidcAccount, addOidcAccountIsLoading, buildOidcDigest } = useRecoveryOidc();
if (props.jwt !== null) {
  const oidcDigest = await buildOidcDigest(props.jwt)
    .then((digest) => digest.toHex());
  const oidcData = {
    oidcDigest,
    iss: ByteVector.fromAsciiString(props.jwt.iss).toHex(),
    aud: ByteVector.fromAsciiString(props.jwt.aud).toHex(),
  } as OidcData;

  addOidcAccount(oidcData);
}
</script>
