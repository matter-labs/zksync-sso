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
import { ByteVector, type JWT, OidcDigest } from "zksync-sso-circuits";

import { useRecoveryOidc } from "~/composables/useRecoveryOidc";

defineEmits<{
  (e: "finish"): void;
}>();

const props = defineProps<{
  jwt: JWT | null;
}>();
const { addOidcAccount, addOidcAccountIsLoading } = useRecoveryOidc();
const runtimeConfig = useRuntimeConfig();

if (props.jwt !== null) {
  const response = await fetch(runtimeConfig.public.saltServiceUrl, {
    method: "GET",
    headers: {
      Authorization: `Bearer ${props.jwt.raw}`,
    },
  })
    .then((res) => res.json())
    .catch((e) => {
      console.error(e);
    });

  const salt = response.salt;
  const oidcDigest = new OidcDigest(props.jwt.iss, props.jwt.aud, props.jwt.sub, ByteVector.fromHex(salt)).serialize();

  const oidcData = {
    oidcDigest,
    iss: ByteVector.fromAsciiString(props.jwt.iss).toHex(),
    aud: ByteVector.fromAsciiString(props.jwt.aud).toHex(),
  } as OidcData;

  addOidcAccount(oidcData);
}
</script>
