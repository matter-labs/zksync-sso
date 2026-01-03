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

      <!-- DEBUG: Confirmation Flow State - Always Visible -->
      <div class="bg-yellow-100 dark:bg-yellow-900 p-4 rounded-lg border-2 border-yellow-500">
        <h3 class="font-semibold mb-2">
          ⚡ Confirmation Flow Debug (Always Visible)
        </h3>
        <div class="space-y-1 text-sm">
          <p><strong>Current State:</strong> <span class="font-mono text-lg">{{ confirmationState }}</span></p>
          <p class="text-xs text-gray-600 dark:text-gray-400">
            Expected flow: ready → started → verifying_account → getting_[sso/wallet]_client → got_[sso/wallet]_client → calling_confirm_guardian → confirm_guardian_completed → refreshing_guardians → complete
          </p>
        </div>
      </div>

      <!-- DEBUG: SSO Connection Status -->
      <div class="bg-gray-100 dark:bg-gray-800 p-4 rounded-lg">
        <h3 class="font-semibold mb-2">
          🔍 SSO Connection Debug Info
        </h3>
        <div class="space-y-1 text-sm font-mono">
          <p><strong>Current SSO Address:</strong> {{ currentSsoAddress || 'null' }}</p>
          <p><strong>Guardian Address:</strong> {{ guardianAddress }}</p>
          <p><strong>Is SSO Account:</strong> {{ isSsoAccount }}</p>
          <p><strong>Is Connected SSO Guardian:</strong> {{ isConnectedSsoGuardian }}</p>
          <p><strong>Is Connected Wallet Guardian:</strong> {{ isConnectedWalletGuardian }}</p>
          <p><strong>Is SSO or Connected Wallet Guardian:</strong> {{ isSsoOrConnectedWalletGuardian }}</p>
          <p><strong>Status Type:</strong> {{ status.type }}</p>
          <p><strong>Status Title:</strong> {{ status.title }}</p>
        </div>
      </div>

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
        <div
          v-if="!isSsoOrConnectedWalletGuardian"
          class="flex flex-col sm:flex-row gap-4 mt-6"
        >
          <!-- Show "Sign in with SSO" if guardian IS the current SSO account and user not logged in -->
          <ZkButton
            v-if="isSsoAccount && currentSsoAddress && guardianAddress.toLowerCase() === currentSsoAddress.toLowerCase() && !isConnectedSsoGuardian"
            class="w-full lg:w-fit"
            variant="secondary"
            data-testid="sign-in-sso"
            @click="navigateToLogin"
          >
            Sign in with SSO
          </ZkButton>

          <!-- Show "Switch account" if guardian IS an SSO account and logged in with wrong SSO account -->
          <ZkButton
            v-if="isSsoAccount && currentSsoAddress && guardianAddress.toLowerCase() !== currentSsoAddress.toLowerCase()"
            class="w-full lg:w-fit"
            variant="secondary"
            data-testid="switch-sso-account"
            @click="signOutAndLogin"
          >
            Sign out and sign in as guardian
          </ZkButton>

          <!-- Show Connect Wallet for non-SSO guardians OR if guardian is SSO but not the current SSO account -->
          <common-connect-button
            v-if="!isSsoAccount || (isSsoAccount && (!currentSsoAddress || guardianAddress.toLowerCase() !== currentSsoAddress.toLowerCase()))"
            class="w-full lg:w-fit"
            :type="accountData.isConnected ? 'secondary' : 'primary'"
          />

          <!-- Show "Sign out SSO" if user is logged in with SSO but needs to use a different wallet -->
          <ZkButton
            v-if="currentSsoAddress && (!isSsoAccount || guardianAddress.toLowerCase() !== currentSsoAddress.toLowerCase())"
            class="w-full lg:w-fit"
            variant="outline"
            data-testid="sign-out-sso"
            @click="signOut"
          >
            Sign out current SSO
          </ZkButton>
        </div>
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
import { storeToRefs } from "pinia";
import { type Address, isAddressEqual } from "viem";
import { z } from "zod";

import { shortenAddress } from "@/utils/formatters";
import { AddressSchema } from "@/utils/schemas";

const accountData = useAppKitAccount();
const route = useRoute();
const { address: currentSsoAddress } = storeToRefs(useAccountStore());
const { confirmGuardian, confirmGuardianInProgress, getGuardians, getGuardiansData } = useRecoveryGuardian();
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
const confirmationState = ref<string>("ready");

const isConnectedWalletGuardian = computed(() => {
  return accountData.value.isConnected && isAddressEqual(accountData.value.address as `0x${string}`, guardianAddress.value);
});

const isConnectedSsoGuardian = computed(() => {
  return !!currentSsoAddress.value && isAddressEqual(currentSsoAddress.value, guardianAddress.value);
});

const isSsoOrConnectedWalletGuardian = computed(() => {
  return isConnectedSsoGuardian.value || isConnectedWalletGuardian.value;
});

const isGuardianConfirmed = computed(() => {
  return !!(getGuardiansData.value?.find((x) => isAddressEqual(x.addr, guardianAddress.value))?.isReady);
});

const canConfirmGuardian = computed(() => {
  return !isGuardianConfirmed.value && isSsoOrConnectedWalletGuardian.value;
});

const status = computed(() => {
  if (isConnectedSsoGuardian.value) {
    return {
      title: "SSO Account Connected",
      message: "You are signed in as the guardian SSO account.",
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

  if (isSsoAccount.value && currentSsoAddress.value) {
    // Signed in with wrong SSO account
    return {
      title: "Wrong SSO Account",
      message: `You are signed in with a different account (${shortenAddress(currentSsoAddress.value)}). Please sign out and sign in with the guardian account.`,
      type: "warning",
    } as const;
  }

  if (isSsoAccount.value) {
    return {
      title: "SSO Account Detected",
      message: "This is a ZKsync SSO account. Please sign in with this account to confirm.",
      type: "warning",
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
  console.error("[DIAGNOSTIC] BUTTON CLICKED - Click event received by confirmGuardianAction");
  console.error("[DIAGNOSTIC] confirmGuardianAction CALLED - function entry point");
  console.error("[DIAGNOSTIC] Current confirmationState before starting:", confirmationState.value);
  console.error("[DIAGNOSTIC] Loading states - confirmGuardianInProgress:", confirmGuardianInProgress.value, "getConfigurableAccountInProgress:", getConfigurableAccountInProgress.value);

  try {
    console.error("[DIAGNOSTIC] Inside try block, about to reset error");
    confirmGuardianError.value = null;

    console.error("[DIAGNOSTIC] About to set confirmationState to 'started'");
    confirmationState.value = "started";
    console.error("[DIAGNOSTIC] confirmationState set to:", confirmationState.value);
    console.error("[DIAGNOSTIC] Waiting 100ms to verify state persists...");
    await new Promise((resolve) => setTimeout(resolve, 100));
    console.error("[DIAGNOSTIC] After 100ms, confirmationState is:", confirmationState.value);

    // Debug: Verify account setup before attempting confirmation
    console.log("=== Starting Guardian Confirmation Debug ===");
    console.log("Account to guard:", accountAddress.value);
    console.log("Guardian address:", guardianAddress.value);
    console.log("Is connected SSO guardian:", isConnectedSsoGuardian.value);

    console.log("🔵 Setting state to verifying_account");
    confirmationState.value = "verifying_account";
    console.error("[DIAGNOSTIC] State changed to verifying_account:", confirmationState.value);
    console.log("🔵 About to call verifyAccountSetup");

    let client;

    // Use the computed property to check if user is logged in as the guardian SSO account
    if (isConnectedSsoGuardian.value) {
      // User is logged in as the guardian SSO account - use SSO client with paymaster
      confirmationState.value = "getting_sso_client";
      console.log("Getting configurable account with paymaster for guardian:", guardianAddress.value);
      client = (await getConfigurableAccount({ address: guardianAddress.value, usePaymaster: true }))!;
      console.log("Got configurable client with address:", client.account.address);
      confirmationState.value = "got_sso_client";
    } else {
      // User needs to connect with wallet (either not logged in or using different account)
      confirmationState.value = "getting_wallet_client";
      console.log("Using wallet client for guardian confirmation");
      if (!accountData.value.isConnected) {
        throw new Error("Please connect your wallet first");
      }
      client = await getWalletClient({ chainId: defaultChain.id });
      console.log("Got wallet client with address:", client.account.address);
      confirmationState.value = "got_wallet_client";
    }

    confirmationState.value = "calling_confirm_guardian";
    console.log("Calling confirmGuardian with client address:", client.account.address);
    const result = await confirmGuardian({
      accountToGuard: accountAddress.value,
      client,
    });
    confirmationState.value = "confirm_guardian_completed";
    console.log("confirmGuardian completed successfully, result:", result);

    // Only refresh guardian list if transaction succeeded
    if (result) {
      confirmationState.value = "refreshing_guardians";
      await getGuardians(accountAddress.value);
      confirmationState.value = "complete";
      console.log("Guardian list refreshed");
    }
  } catch (err) {
    console.error("[DIAGNOSTIC] =============== ERROR CAUGHT ===============");
    console.error("[DIAGNOSTIC] Error object:", err);
    console.error("[DIAGNOSTIC] Error message:", err instanceof Error ? err.message : String(err));
    console.error("[DIAGNOSTIC] Error stack:", err instanceof Error ? err.stack : "No stack");
    console.error("[DIAGNOSTIC] confirmationState before error handling:", confirmationState.value);

    confirmationState.value = "error: " + (err instanceof Error ? err.message : "Unknown error");
    console.error("[DIAGNOSTIC] confirmationState set to:", confirmationState.value);

    const errorMessage = err instanceof Error ? err.message : "An error occurred while confirming the guardian.";
    confirmGuardianError.value = errorMessage;
    console.error("[DIAGNOSTIC] confirmGuardianError set to:", confirmGuardianError.value);
    console.error("[DIAGNOSTIC] =============== END ERROR HANDLING ===============");
    // eslint-disable-next-line no-console
    console.error("Guardian confirmation error:", err);
  }
};

const navigateToLogin = () => {
  // Redirect to login page with return URL to come back to this confirmation page
  const returnUrl = encodeURIComponent(route.fullPath);
  navigateTo(`/?redirect=${returnUrl}`);
};

const signOutAndLogin = async () => {
  // Sign out of current SSO session
  await navigateTo("/logout");
  // After logout completes, redirect to login with return URL
  const returnUrl = encodeURIComponent(route.fullPath);
  await navigateTo(`/?redirect=${returnUrl}`);
};

onMounted(async () => {
  console.error("[DIAGNOSTIC] onMounted - Initial loading states:");
  console.error("  confirmGuardianInProgress:", confirmGuardianInProgress.value);
  console.error("  getConfigurableAccountInProgress:", getConfigurableAccountInProgress.value);
  console.error("  isSsoAccountLoading:", isSsoAccountLoading.value);

  await getGuardians(accountAddress.value);
  console.log("[confirm-guardian] Checking if guardian is SSO account:", guardianAddress.value);
  const result = await checkIsSsoAccount(guardianAddress.value);
  console.log("[confirm-guardian] checkIsSsoAccount result:", result);
  isSsoAccount.value = result === undefined ? null : result;
  console.log("[confirm-guardian] isSsoAccount.value set to:", isSsoAccount.value);

  console.error("[DIAGNOSTIC] onMounted complete - Final loading states:");
  console.error("  confirmGuardianInProgress:", confirmGuardianInProgress.value);
  console.error("  getConfigurableAccountInProgress:", getConfigurableAccountInProgress.value);
  console.error("  isSsoAccountLoading:", isSsoAccountLoading.value);
});

definePageMeta({
  layout: "dashboard",
});
</script>
