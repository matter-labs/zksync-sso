<template>
  <div
    class="flex flex-col items-center gap-4"
  >
    <p class="text-center text-gray-600 dark:text-gray-400">
      You are about to set the account <b>{{ props.jwt.email }}</b> as
      your recovery account.
    </p>
  </div>

  <common-spinner
    v-if="addOidcAccountIsLoading"
    class="w-8 h-8"
  />
  <ZkButton
    v-else
    type="primary"
    class="w-full md:max-w-48"
    @click="confirmAccount"
  >
    Continue
  </ZkButton>
</template>

<script setup lang="ts">
import type { JWT } from "zksync-sso-circuits";

const emits = defineEmits<{
  (e: "next"): void;
}>();

const props = defineProps<{
  jwt: JWT;
}>();

const { addOidcAccount, addOidcAccountIsLoading, buildOidcDigest } = useRecoveryOidc();

async function confirmAccount() {
  const oidcDigest = await buildOidcDigest(props.jwt)
    .then((digest) => digest.toHex());

  await addOidcAccount(oidcDigest, props.jwt.iss);
  emits("next");
}
</script>
