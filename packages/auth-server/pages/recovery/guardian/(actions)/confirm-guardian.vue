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
      <account-recovery-confirm-info-card title="Account Address">
        <span class="mr-2 font-mono text-lg">{{ accountAddress }}</span>
        <common-copy-to-clipboard
          class="!inline-flex"
          :text="accountAddress"
        />
      </account-recovery-confirm-info-card>

      <account-recovery-confirm-info-card title="Guardian Address">
        <span class="mr-2 font-mono text-lg">{{ guardianAddress }}</span>
        <common-copy-to-clipboard
          class="!inline-flex"
          :text="guardianAddress"
        />

        <template #footer>
          {{ isSsoAccount === null ? "Checking account type..." : (isSsoAccount ? "ZKsync SSO Account" : "Standard Account") }}
        </template>
      </account-recovery-confirm-info-card>

      <account-recovery-confirm-info-card
        v-if="isSsoAccountLoading"
        title="Checking Account Type"
      >
        <div class="flex flex-col items-center w-full gap-4">
          <common-spinner class="w-8 h-8" />
          <p class="text-center text-gray-600 dark:text-gray-400">
            Checking guardian account...
          </p>
        </div>
      </account-recovery-confirm-info-card>

      <account-recovery-confirm-action-card
        v-else-if="isSsoAccountError"
        title="Error Checking Account"
        type="error"
      >
        An error occurred while checking the guardian account. Please try again.
      </account-recovery-confirm-action-card>

      <account-recovery-confirm-action-card
        v-else-if="isGuardianConfirmed"
        title="Guardian Confirmed"
        type="success"
      >
        This guardian has been successfully confirmed and can now help recover your account if needed.
      </account-recovery-confirm-action-card>

      <account-recovery-confirm-action-card
        v-else
        :title="status.title"
        :type="status.type"
      >
        <p>
          {{ status.message }}
        </p>
        <common-connect-button
          v-if="!isSsoAccount"
          class="w-full lg:w-fit mt-6"
          :type="accountData.isConnected ? 'secondary' : 'primary'"
        />
      </account-recovery-confirm-action-card>

      <ZkButton
        v-if="canConfirmGuardian"
        class="w-full lg:w-fit"
        :loading="confirmGuardianInProgress || getConfigurableAccountInProgress"
        @click="confirmGuardianAction"
      >
        Confirm Guardian
      </ZkButton>

      <p
        v-if="confirmGuardianError"
        class="text-error-600 dark:text-error-400"
      >
        {{ confirmGuardianError }}
      </p>
    </main>
  </div>
</template>

<script setup lang="ts">
import { useAppKitAccount } from "@reown/appkit/vue";
import { type Address, isAddressEqual } from "viem";
import { z } from "zod";

import { shortenAddress } from "@/utils/formatters";
import { AddressSchema } from "@/utils/schemas";

const accountData = useAppKitAccount();
const route = useRoute();
const { confirmGuardian, confirmGuardianInProgress, getGuardians, getGuardiansData } = useRecoveryGuardian();
const { verifyAccountSetup } = useDebugGuardian();
const { getConfigurableAccount, getConfigurableAccountInProgress } = useConfigurableAccount();
const { getWalletClient, defaultChain } = useClientStore();
const { isSsoAccount: checkIsSsoAccount, isLoading: isSsoAccountLoading, error: isSsoAccountError } = useIsSsoAccount();

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

const isSsoOrConnectedWalletGuardian = computed(() => {
  return isSsoAccount.value || isConnectedWalletGuardian.value;
});

const isGuardianConfirmed = computed(() => {
  return !!(getGuardiansData.value?.find((x) => isAddressEqual(x.addr, guardianAddress.value))?.isReady);
});

const canConfirmGuardian = computed(() => {
  return !isGuardianConfirmed.value && isSsoOrConnectedWalletGuardian.value;
});

const status = computed(() => {
  if (isSsoAccount.value) {
    return {
      title: "SSO Account Detected",
      message: "The guardian is detected as a ZKSync SSO account.",
      type: "success",
    } as const;
  }

  if (isConnectedWalletGuardian.value) {
    return {
      title: "Wallet Connected",
      message: "Guardian wallet successfully connected.",
      type: "success",
    } as const;
  }

  return {
    title: "Action Required",
    message: accountData.value.isConnected
      ? `Please connect with the guardian wallet address (${shortenAddress(guardianAddress.value)})`
      : "Connect your wallet to confirm this guardian for your account.",
    type: "warning",
  } as const;
});

const confirmGuardianAction = async () => {
  try {
    confirmGuardianError.value = null;

    // Debug: Verify account setup before attempting confirmation
    console.log("=== Starting Guardian Confirmation Debug ===");
    await verifyAccountSetup(accountAddress.value);

    let client;

    if (isSsoAccount.value) {
      client = (await getConfigurableAccount({ address: guardianAddress.value }))!;
    } else {
      client = await getWalletClient({ chainId: defaultChain.id });
    }

    const result = await confirmGuardian({
      accountToGuard: accountAddress.value,
      client,
    });

    // Only refresh guardian list if transaction succeeded
    if (result) {
      await getGuardians(accountAddress.value);
    }
  } catch (err) {
    const errorMessage = err instanceof Error ? err.message : "An error occurred while confirming the guardian.";
    confirmGuardianError.value = errorMessage;
    // eslint-disable-next-line no-console
    console.error("Guardian confirmation error:", err);
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
