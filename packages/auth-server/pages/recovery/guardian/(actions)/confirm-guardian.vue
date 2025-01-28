<template>
  <div class="min-h-screen">
    <header class="max-w-[1920px] mx-auto mb-12">
      <app-generic-nav />
    </header>
    <main
      v-if="accountAddress && guardianAddress"
      class="max-w-[900px] mx-auto flex flex-col gap-6"
    >
      <div>
        <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-3">
          Confirm Guardian Account
        </h1>
        <p class="text-lg text-gray-600 dark:text-gray-400">
          Review and confirm the guardian details below:
        </p>
      </div>

      <div class="space-y-4">
        <div class="rounded-2xl bg-neutral-100/50 backdrop-blur-sm dark:bg-gray-800/50 p-6">
          <label class="block text-sm font-medium text-gray-600 dark:text-gray-400 mb-2">
            Account Address
          </label>
          <div class=" text-gray-900 dark:text-gray-100 break-all">
            <span class="mr-2 font-mono text-lg">{{ accountAddress }}</span>
            <common-copy-to-clipboard
              class="!inline-flex"
              :text="accountAddress"
            />
          </div>
        </div>

        <div class="rounded-2xl bg-neutral-100/50 backdrop-blur-sm dark:bg-gray-800/50 p-6">
          <label class="block text-sm font-medium text-gray-600 dark:text-gray-400 mb-2">
            Guardian Address
          </label>
          <div class="text-gray-900 dark:text-gray-100 break-all">
            <span class="mr-2 font-mono text-lg">
              {{ guardianAddress }}</span>
            <common-copy-to-clipboard
              class="!inline-flex"
              :text="guardianAddress"
            />
          </div>
        </div>
      </div>

      <div
        v-if="isGuardianConfirmed"
        class="rounded-2xl flex gap-4 bg-green-50/80 dark:bg-green-900/30 backdrop-blur-sm p-6 border border-green-200 dark:border-green-700/50"
      >
        <CheckCircleIcon class="w-6 h-6 text-green-600 dark:text-green-400 flex-shrink-0" />
        <div class="flex flex-col flex-1">
          <span class="text-lg font-medium text-green-700 dark:text-green-300">Guardian Confirmed</span>
          <p class="text-green-600 dark:text-green-400">
            This guardian has been added to your account.
          </p>
        </div>
      </div>

      <div
        v-else
        class="rounded-2xl flex gap-4 backdrop-blur-sm p-6 border"
        :class="{
          'bg-yellow-50/80 dark:bg-yellow-900/30 border-yellow-200 dark:border-yellow-700/50': !accountData.isConnected || !isAddressEqual(accountData.address as `0x${string}`, guardianAddress),
          'bg-green-50/80 dark:bg-green-900/30 border-green-200 dark:border-green-700/50': accountData.isConnected && isAddressEqual(accountData.address as `0x${string}`, guardianAddress),
        }"
      >
        <component
          :is="accountData.isConnected && isAddressEqual(accountData.address as `0x${string}`, guardianAddress) ? CheckCircleIcon : ExclamationTriangleIcon"
          class="w-6 h-6 flex-shrink-0"
          :class="{
            'text-yellow-600 dark:text-yellow-400': !accountData.isConnected || !isAddressEqual(accountData.address as `0x${string}`, guardianAddress),
            'text-green-600 dark:text-green-400': accountData.isConnected && isAddressEqual(accountData.address as `0x${string}`, guardianAddress),
          }"
        />
        <div class="flex flex-col flex-1">
          <h3
            class="text-lg font-semibold mb-2"
            :class="{
              'text-yellow-800 dark:text-yellow-200': !accountData.isConnected || !isAddressEqual(accountData.address as `0x${string}`, guardianAddress),
              'text-green-800 dark:text-green-200': accountData.isConnected && isAddressEqual(accountData.address as `0x${string}`, guardianAddress),
            }"
          >
            {{ accountData.isConnected && isAddressEqual(accountData.address as `0x${string}`, guardianAddress) ? 'Wallet Connected' : 'Action Required' }}
          </h3>
          <p
            :class="{
              'text-yellow-700 dark:text-yellow-300': !accountData.isConnected || !isAddressEqual(accountData.address as `0x${string}`, guardianAddress),
              'text-green-700 dark:text-green-300': accountData.isConnected && isAddressEqual(accountData.address as `0x${string}`, guardianAddress),
            }"
          >
            {{
              accountData.isConnected && isAddressEqual(accountData.address as `0x${string}`, guardianAddress)
                ? 'Guardian wallet successfully connected.'
                : accountData.isConnected
                  ? 'The connected wallet is not the guardian address. Please connect a guardian wallet.'
                  : 'Connect your wallet to confirm this guardian for your account.'
            }}
          </p>
          <common-connect-button
            class="w-full lg:w-fit mt-6"
            :type="accountData.isConnected ? 'secondary' : 'primary'"
          />
        </div>
      </div>

      <ZkButton
        v-if="accountData.isConnected && !isGuardianConfirmed"
        class="w-full lg:w-fit"
        :disabled="!isAddressEqual(accountData.address as `0x${string}`, guardianAddress)"
        @click="confirmGuardian"
      >
        Confirm Guardian
      </ZkButton>
    </main>
  </div>
</template>

<script setup lang="ts">
import { CheckCircleIcon, ExclamationTriangleIcon } from "@heroicons/vue/24/solid";
import { useAppKitAccount } from "@reown/appkit/vue";
import { type Address, isAddressEqual } from "viem";
import { z } from "zod";

import { AddressSchema } from "@/utils/schemas";

const accountData = useAppKitAccount();
const route = useRoute();

const accountAddress = ref<Address | null>(null);
const guardianAddress = ref<Address | null>(null);

const params = z.object({
  accountAddress: AddressSchema,
  guardianAddress: AddressSchema,
}).safeParse(route.query);

if (!params.success) {
  throw createError({
    statusCode: 404,
    statusMessage: "Page not found",
    fatal: true,
  });
} else {
  accountAddress.value = params.data.accountAddress;
  guardianAddress.value = params.data.guardianAddress;
}

const isGuardianConfirmed = ref(false);

function confirmGuardian() {
  isGuardianConfirmed.value = true;
}

definePageMeta({
  layout: "dashboard",
});
</script>
