<template>
  <div class="min-h-screen">
    <header class="max-w-[1920px] mx-auto mb-12">
      <app-generic-nav />
    </header>
    <main class="max-w-[900px] mx-auto flex flex-col gap-6">
      <div
        v-if="error"
        class="rounded-2xl flex gap-4 bg-red-50/80 dark:bg-red-900/30 backdrop-blur-sm p-6 border border-red-200 dark:border-red-700/50"
      >
        <ExclamationTriangleIcon class="w-6 h-6 text-red-600 dark:text-red-400 flex-shrink-0" />
        <span class="text-red-700 dark:text-red-300">{{ error }}</span>
      </div>

      <template v-else>
        <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-3">
          Recovery Request Review
        </h1>
        <p class="text-lg text-gray-600 dark:text-gray-400">
          Review the recovery details below:
        </p>

        <div class="space-y-4">
          <div class="rounded-2xl bg-neutral-100/50 backdrop-blur-sm dark:bg-gray-800/50 p-6">
            <label class="block text-sm font-medium text-gray-600 dark:text-gray-400 mb-2"> Account Address </label>
            <div class="text-gray-900 dark:text-gray-100 break-all">
              <span class="mr-2 font-mono text-lg">{{ recoveryParams?.accountAddress }}</span>
              <common-copy-to-clipboard
                class="!inline-flex"
                :text="recoveryParams?.accountAddress ?? ''"
              />
            </div>
          </div>

          <div class="rounded-2xl bg-neutral-100/50 backdrop-blur-sm dark:bg-gray-800/50 p-6">
            <label class="block text-sm font-medium text-gray-600 dark:text-gray-400 mb-2">
              Recovery Credentials
            </label>
            <div class="space-y-4">
              <div>
                <span class="block text-sm text-gray-600 dark:text-gray-400 mb-1">ID:</span>
                <div class="text-gray-900 dark:text-gray-100 break-all font-mono">
                  {{ recoveryParams?.credentialId }}
                </div>
              </div>
              <div>
                <span class="block text-sm text-gray-600 dark:text-gray-400 mb-1">Public Key:</span>
                <div class="text-gray-900 dark:text-gray-100 break-all font-mono">
                  {{ recoveryParams?.credentialPublicKey }}
                </div>
              </div>
            </div>
          </div>

          <div
            class="rounded-2xl flex gap-4 backdrop-blur-sm p-6 border"
            :class="{
              'bg-yellow-50/80 dark:bg-yellow-900/30 border-yellow-200 dark:border-yellow-700/50':
                !accountData.isConnected || !isAccountGuardian,
              'bg-green-50/80 dark:bg-green-900/30 border-green-200 dark:border-green-700/50':
                accountData.isConnected && isAccountGuardian,
            }"
          >
            <component
              :is="accountData.isConnected && isAccountGuardian ? CheckCircleIcon : ExclamationTriangleIcon"
              class="w-6 h-6 flex-shrink-0"
              :class="{
                'text-yellow-600 dark:text-yellow-400': !accountData.isConnected || !isAccountGuardian,
                'text-green-600 dark:text-green-400': accountData.isConnected && isAccountGuardian,
              }"
            />
            <div class="flex flex-col flex-1">
              <h3
                class="text-lg font-semibold mb-2"
                :class="{
                  'text-yellow-800 dark:text-yellow-200': !accountData.isConnected || !isAccountGuardian,
                  'text-green-800 dark:text-green-200': accountData.isConnected && isAccountGuardian,
                }"
              >
                {{ accountData.isConnected && isAccountGuardian ? "Wallet Connected" : "Action Required" }}
              </h3>
              <p
                :class="{
                  'text-yellow-700 dark:text-yellow-300': !accountData.isConnected || !isAccountGuardian,
                  'text-green-700 dark:text-green-300': accountData.isConnected && isAccountGuardian,
                }"
              >
                {{
                  accountData.isConnected && isAccountGuardian
                    ? "Guardian wallet successfully connected."
                    : accountData.isConnected
                      ? "The connected wallet is not a guardian. Please connect a guardian wallet."
                      : "Connect your wallet to sign the recovery transaction."
                }}
              </p>
              <common-connect-button
                class="w-full lg:w-fit mt-6"
                :type="accountData.isConnected ? 'secondary' : 'primary'"
              />
            </div>
          </div>

          <ZkButton
            v-if="accountData.isConnected && !isSuccess"
            class="w-full lg:w-fit"
            :disabled="!isAccountGuardian"
            :loading="initRecoveryInProgress"
            @click="confirmRecoveryAction"
          >
            Sign Recovery Transaction
          </ZkButton>

          <div
            v-if="isSuccess"
            class="rounded-2xl flex gap-4 bg-green-50/80 dark:bg-green-900/30 backdrop-blur-sm p-6 border border-green-200 dark:border-green-700/50"
          >
            <CheckCircleIcon class="w-6 h-6 text-green-600 dark:text-green-400 flex-shrink-0" />
            <div class="flex flex-col">
              <h3 class="text-lg font-semibold text-green-800 dark:text-green-200 mb-2">
                Done!
              </h3>
              <p class="text-green-700 dark:text-green-300">
                The account will be ready to use with the new credentials in 24hrs.
              </p>
            </div>
          </div>
        </div>
      </template>
    </main>
  </div>
</template>

<script setup lang="ts">
import { CheckCircleIcon, ExclamationTriangleIcon } from "@heroicons/vue/24/solid";
import { useAppKitAccount } from "@reown/appkit/vue";
import { hexToBytes, isAddressEqual, zeroAddress } from "viem";
import { encodePasskeyModuleParameters, getPublicKeyBytesFromPasskeySignature } from "zksync-sso/utils";
import { z } from "zod";

import { uint8ArrayToHex } from "@/utils/formatters";
import { AddressSchema } from "@/utils/schemas";

const error = ref<string | null>(null);
const accountData = useAppKitAccount();
const { getRecovery, initRecovery, initRecoveryInProgress, getGuardians, getGuardiansData } = useRecoveryGuardian();

definePageMeta({
  layout: "dashboard",
});

const route = useRoute();

const RecoveryParamsSchema = z
  .object({
    accountAddress: AddressSchema,
    credentialId: z.string().min(1),
    credentialPublicKey: z.string().min(1),
    checksum: z.string().min(1),
  })
  .refine(
    async (data) => {
      const dataToHash = `${data.accountAddress}:${data.credentialId}:${data.credentialPublicKey}`;
      const calculatedChecksum = uint8ArrayToHex(
        new Uint8Array(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(dataToHash))).slice(0, 8),
      );
      return calculatedChecksum === data.checksum;
    },
    {
      message: "Invalid recovery data checksum",
    },
  );

const recoveryParams = ref<z.infer<typeof RecoveryParamsSchema> | null>(null);
const isAccountGuardian = ref(false);
const isSuccess = ref(false);

try {
  recoveryParams.value = await RecoveryParamsSchema.parseAsync({
    accountAddress: route.query.accountAddress,
    credentialId: route.query.credentialId,
    credentialPublicKey: route.query.credentialPublicKey,
    checksum: route.query.checksum,
  });
  await getGuardians(recoveryParams.value.accountAddress);

  const recoveryStatus = await getRecovery(recoveryParams.value.accountAddress);
  isSuccess.value = recoveryStatus[2] === route.query.credentialId;
} catch {
  error.value = "Invalid recovery parameters. Please verify the URL and try again.";
}

watchEffect(() => {
  isAccountGuardian.value = !!(getGuardiansData.value?.find((x) => isAddressEqual(x.addr, (accountData.value.address as `0x${string}`) ?? zeroAddress))?.isReady);
});

const confirmRecoveryAction = async () => {
  let origin: string | undefined = undefined;
  if (!origin) {
    try {
      origin = window.location.origin;
    } catch {
      throw new Error("Can't identify expectedOrigin, please provide it manually");
    }
  }

  // eslint-disable-next-line @typescript-eslint/no-non-null-asserted-optional-chain
  const passkeyPublicKey = getPublicKeyBytesFromPasskeySignature(hexToBytes(`0x${recoveryParams.value?.credentialPublicKey!}`));
  const encodedPasskeyParameters = encodePasskeyModuleParameters({
    passkeyPublicKey,
    expectedOrigin: origin,
  });
  const accountId = recoveryParams.value?.credentialId || encodedPasskeyParameters;
  // eslint-disable-next-line @typescript-eslint/no-non-null-asserted-optional-chain
  await initRecovery(recoveryParams.value?.accountAddress!, encodedPasskeyParameters, accountId);
  isSuccess.value = true;
};
</script>
