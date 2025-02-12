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

      <div class="rounded-2xl bg-neutral-100/50 backdrop-blur-sm dark:bg-gray-800/50 p-6">
        <label class="block text-sm font-medium text-gray-600 dark:text-gray-400 mb-2">
          Account Address
        </label>
        <div class="text-gray-900 dark:text-gray-100 break-all">
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
          <span class="mr-2 font-mono text-lg">{{ guardianAddress }}</span>
          <common-copy-to-clipboard
            class="!inline-flex"
            :text="guardianAddress"
          />
        </div>
        <p class="mt-2 text-gray-600 dark:text-gray-400 font-mono text-xs">
          {{ isSsoAccount === null ? "Checking account type..." : (isSsoAccount ? "ZKsync SSO Account" : "Standard Account") }}
        </p>
      </div>

      <div
        v-if="isSsoAccountLoading"
        class="rounded-2xl flex gap-4 bg-neutral-100/50 backdrop-blur-sm dark:bg-gray-800/50 p-6"
      >
        <div class="flex flex-col items-center w-full gap-4">
          <common-spinner class="w-8 h-8" />
          <p class="text-center text-gray-600 dark:text-gray-400">
            Checking guardian account...
          </p>
        </div>
      </div>

      <div
        v-else-if="isSsoAccountError"
        class="rounded-2xl flex gap-4 bg-error-50/80 dark:bg-error-900/30 backdrop-blur-sm p-6 border border-error-200 dark:border-error-700/50"
      >
        <ExclamationTriangleIcon class="w-6 h-6 text-error-600 dark:text-error-400 flex-shrink-0" />
        <div class="flex flex-col flex-1">
          <span class="text-lg font-medium text-error-700 dark:text-error-300">Error Checking Account</span>
          <p class="text-error-600 dark:text-error-400">
            An error occurred while checking the guardian account. Please try again.
          </p>
        </div>
      </div>

      <div
        v-else-if="isGuardianConfirmed"
        class="rounded-2xl flex gap-4 bg-green-50/80 dark:bg-green-900/30 backdrop-blur-sm p-6 border border-green-200 dark:border-green-700/50"
      >
        <CheckCircleIcon class="w-6 h-6 text-green-600 dark:text-green-400 flex-shrink-0" />
        <div class="flex flex-col flex-1">
          <span class="text-lg font-medium text-green-800 dark:text-green-200">Guardian Confirmed</span>
          <p class="text-green-700 dark:text-green-300">
            This guardian has been successfully confirmed and can now help recover your account if needed.
          </p>
        </div>
      </div>

      <div
        v-else
        class="rounded-2xl flex gap-4 backdrop-blur-sm p-6 border"
        :class="statusStyles.container"
      >
        <component
          :is="(isSsoAccount || isConnectedWalletGuardian) ? CheckCircleIcon : ExclamationTriangleIcon"
          class="w-6 h-6 flex-shrink-0"
          :class="statusStyles.icon"
        />
        <div class="flex flex-col flex-1">
          <h3
            class="text-lg font-semibold mb-2"
            :class="statusStyles.text"
          >
            {{ status.title }}
          </h3>
          <p :class="statusStyles.text">
            {{ status.message }}
          </p>
          <common-connect-button
            v-if="!isSsoAccount"
            class="w-full lg:w-fit mt-6"
            :type="accountData.isConnected ? 'secondary' : 'primary'"
          />
        </div>
      </div>

      <div
        v-if="confirmGuardianError"
        class="rounded-2xl flex gap-4 bg-error-50/80 dark:bg-error-900/30 backdrop-blur-sm p-6 border border-error-200 dark:border-error-700/50"
      >
        <ExclamationTriangleIcon class="w-6 h-6 text-error-600 dark:text-error-400 flex-shrink-0" />
        <div class="flex flex-col flex-1">
          <p class="text-error-600 dark:text-error-400">
            {{ confirmGuardianError }}
          </p>
        </div>
      </div>

      <ZkButton
        v-if="canConfirmGuardian"
        class="w-full lg:w-fit"
        :loading="confirmGuardianInProgress || getConfigurableAccountInProgress"
        @click="confirmGuardianAction"
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

import { shortenAddress } from "@/utils/formatters";
import { AddressSchema } from "@/utils/schemas";

const accountData = useAppKitAccount();
const route = useRoute();
const { confirmGuardian, confirmGuardianInProgress, getGuardians, getGuardiansData } = useRecoveryGuardian();
const { getConfigurableAccount, getConfigurableAccountInProgress } = useConfigurableAccount();
const { getWalletClient, defaultChain } = useClientStore();
const { checkIsSsoAccount, isLoading: isSsoAccountLoading, error: isSsoAccountError } = useCheckSsoAccount(defaultChain.id);

// Parse and validate URL params
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
}

const accountAddress = ref<Address>(params.data.accountAddress);
const guardianAddress = ref<Address>(params.data.guardianAddress);
const isSsoAccount = ref<null | boolean>(null);
const confirmGuardianError = ref<string | null>(null);

const isConnectedWalletGuardian = computed(() => {
  return accountData.value.isConnected && isAddressEqual(accountData.value.address as `0x${string}`, guardianAddress.value);
});

const isGuardianConfirmed = computed(() => {
  return !!(getGuardiansData.value?.find((x) => isAddressEqual(x.addr, guardianAddress.value))?.isReady);
});

const canConfirmGuardian = computed(() => {
  return !isGuardianConfirmed.value && (isSsoAccount.value || isConnectedWalletGuardian.value);
});

const statusStyles = computed(() => {
  const isSuccess = isSsoAccount.value || isConnectedWalletGuardian.value;

  return {
    container: {
      "bg-yellow-50/80 dark:bg-yellow-900/30 border-yellow-200 dark:border-yellow-700/50": !isSuccess,
      "bg-green-50/80 dark:bg-green-900/30 border-green-200 dark:border-green-700/50": isSuccess,
    },
    icon: {
      "text-yellow-600 dark:text-yellow-400": !isSuccess,
      "text-green-600 dark:text-green-400": isSuccess,
    },
    text: {
      "text-yellow-800 dark:text-yellow-200": !isSuccess,
      "text-green-800 dark:text-green-200": isSuccess,
    },
  };
});

const status = computed(() => {
  if (isSsoAccount.value) {
    return {
      title: "SSO Account Detected",
      message: "The guardian is detected as a ZKSync SSO account.",
    };
  }

  if (isConnectedWalletGuardian.value) {
    return {
      title: "Wallet Connected",
      message: "Guardian wallet successfully connected.",
    };
  }

  return {
    title: "Action Required",
    message: accountData.value.isConnected
      ? `Please connect with the guardian wallet address (${shortenAddress(guardianAddress.value)})`
      : "Connect your wallet to confirm this guardian for your account.",
  };
});

const confirmGuardianAction = async () => {
  try {
    let client;

    if (isSsoAccount.value) {
      client = (await getConfigurableAccount({ address: guardianAddress.value }))!;
    } else {
      client = await getWalletClient({ chainId: defaultChain.id });
    }

    await confirmGuardian({
      accountToGuard: accountAddress.value,
      client,
    });
    confirmGuardianError.value = null;
    await getGuardians(accountAddress.value);
  } catch (err) {
    confirmGuardianError.value = "An error occurred while confirming the guardian. Please try again.";
    console.error(err);
  }
};

onMounted(async () => {
  await getGuardians(accountAddress.value);
  const result = await checkIsSsoAccount(guardianAddress.value);
  isSsoAccount.value = result === undefined ? null : result;
});

definePageMeta({
  layout: "dashboard",
});
</script>
