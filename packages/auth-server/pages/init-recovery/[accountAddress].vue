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
            <label class="block text-sm font-medium text-gray-600 dark:text-gray-400 mb-2">
              Account Address
            </label>
            <div class="text-gray-900 dark:text-gray-100 break-all">
              <span class="mr-2 font-mono text-lg">{{ recoveryParams.accountAddress }}</span>
              <common-copy-to-clipboard
                class="!inline-flex"
                :text="recoveryParams.accountAddress"
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
                  {{ recoveryParams.credentialId }}
                </div>
              </div>
              <div>
                <span class="block text-sm text-gray-600 dark:text-gray-400 mb-1">Public Key:</span>
                <div class="text-gray-900 dark:text-gray-100 break-all font-mono">
                  {{ recoveryParams.credentialPublicKey }}
                </div>
              </div>
            </div>
          </div>

          <div class="rounded-2xl flex gap-4 bg-yellow-50/80 dark:bg-yellow-900/30 backdrop-blur-sm p-6 border border-yellow-200 dark:border-yellow-700/50">
            <ExclamationTriangleIcon class="w-6 h-6 text-yellow-600 dark:text-yellow-400 flex-shrink-0" />
            <div class="flex flex-col flex-1">
              <h3 class="text-lg font-semibold text-yellow-800 dark:text-yellow-200 mb-2">
                Action Required
              </h3>
              <p class="text-yellow-700 dark:text-yellow-300">
                Connect your wallet to sign the recovery transaction.
              </p>
              <common-connect-button
                class="w-full lg:w-fit mt-6"
                :type="accountData.isConnected ? 'secondary' : 'primary'"
              />
            </div>
          </div>

          <p
            v-if="accountData.address && !isAddressEqual(accountData.address as `0x${string}`, recoveryParams.accountAddress)"
            class="text-red-500 dark:text-red-400 font-medium"
          >
            The connected wallet is not the account address. Please connect the correct wallet.
          </p>

          <ZkButton
            v-if="accountData.isConnected"
            class="w-full lg:w-fit"
            :disabled="!isAddressEqual(accountData.address as `0x${string}`, recoveryParams.accountAddress)"
          >
            Sign Recovery Transaction
          </ZkButton>
        </div>
      </template>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ExclamationTriangleIcon } from "@heroicons/vue/24/solid";
import { useAppKitAccount } from "@reown/appkit/vue";
import { isAddressEqual } from "viem";
import { z } from "zod";

import { uint8ArrayToHex } from "@/utils/formatters";
import { AddressSchema } from "@/utils/schemas";

const error = ref<string | null>(null);
const accountData = useAppKitAccount();

definePageMeta({
  layout: "dashboard",
});

const route = useRoute();

const RecoveryParamsSchema = z.object({
  accountAddress: AddressSchema,
  credentialId: z.string().min(1),
  credentialPublicKey: z.string().min(1),
  checksum: z.string().min(1),
}).refine(async (data) => {
  const dataToHash = `${data.accountAddress}:${data.credentialId}:${data.credentialPublicKey}`;
  const calculatedChecksum = uint8ArrayToHex(
    new Uint8Array(
      await crypto.subtle.digest("SHA-256", new TextEncoder().encode(dataToHash)),
    ).slice(0, 8),
  );
  return calculatedChecksum === data.checksum;
}, {
  message: "Invalid recovery data checksum",
});

const recoveryParams = await RecoveryParamsSchema.parseAsync({
  accountAddress: route.params.accountAddress,
  credentialId: route.query.credentialId,
  credentialPublicKey: route.query.credentialPublicKey,
  checksum: route.query.checksum,
}).catch(() => {
  error.value = "Invalid recovery parameters. Please verify the URL and try again.";
});
</script>
