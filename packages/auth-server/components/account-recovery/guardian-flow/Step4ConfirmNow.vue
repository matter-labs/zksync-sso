<template>
  <div
    v-if="isLoading"
    class="flex flex-col items-center gap-4"
  >
    <common-spinner class="w-8 h-8" />
    <p class="text-center text-gray-600 dark:text-gray-400">
      Checking guardian account...
    </p>
  </div>

  <div
    v-else-if="isSsoAccountError"
    class="flex flex-col items-center gap-4"
  >
    <p class="text-center text-error-500 dark:text-error-400">
      An error occurred while checking the guardian account. Please try again.
    </p>
    <ZkButton
      type="primary"
      class="w-full mt-4"
      @click="handleCheck"
    >
      Retry
    </ZkButton>
    <ZkButton
      type="secondary"
      class="w-full"
      @click="emit('back')"
    >
      Back
    </ZkButton>
  </div>

  <div
    v-else
    class="flex flex-col items-center gap-4"
  >
    <p class="text-center text-gray-600 dark:text-gray-400">
      {{
        isSsoAccount
          ? "The guardian is detected as a ZKSync SSO account."
          : "The guardian is detected as a standard account."
      }}
      <br>
      Please confirm with your guardian account to continue.
    </p>

    <p
      v-if="confirmGuardianErrorMessage"
      class="text-center text-error-500 dark:text-error-400 mt-4"
    >
      {{ confirmGuardianErrorMessage }}
    </p>

    <ZkButton
      v-if="isSsoAccount || (accountData.isConnected && isConnectedWalletGuardian)"
      type="primary"
      class="w-full md:max-w-48 mt-4"
      :loading="confirmGuardianInProgress || getConfigurableAccountInProgress"
      @click="handleConfirmGuardian"
    >
      Confirm Guardian
    </ZkButton>

    <CommonConnectButton
      v-if="!isSsoAccount"
      type="primary"
      class="w-full md:max-w-48 mt-4"
      :disabled="confirmGuardianInProgress || getConfigurableAccountInProgress"
    />

    <ZkButton
      type="secondary"
      class="w-full md:max-w-48"
      :disabled="confirmGuardianInProgress || getConfigurableAccountInProgress"
      @click="emit('back')"
    >
      Back
    </ZkButton>
  </div>
</template>

<script setup lang="ts">
import { useAppKitAccount } from "@reown/appkit/vue";
import { type Address, isAddressEqual } from "viem";

const props = defineProps<{
  guardianAddress: Address;
}>();

const emit = defineEmits<{
  (e: "next" | "back"): void;
}>();

const { getWalletClient, defaultChain } = useClientStore();
const { isSsoAccount: checkIsSsoAccount, isLoading, error: isSsoAccountError } = useIsSsoAccount();
const { confirmGuardian, confirmGuardianInProgress } = useRecoveryGuardian();
const { getConfigurableAccount, getConfigurableAccountInProgress } = useConfigurableAccount();
const { address: currentSsoAddress } = useAccountStore();
const accountData = useAppKitAccount();

const confirmGuardianErrorMessage = ref<string | null>(null);
const isSsoAccount = ref(false);

const isConnectedWalletGuardian = computed(() => {
  return (
    accountData.value.isConnected && isAddressEqual(accountData.value.address as Address, props.guardianAddress)
  );
});

const handleCheck = async () => {
  const result = await checkIsSsoAccount(props.guardianAddress);
  isSsoAccount.value = result ?? false;
};

const handleConfirmGuardian = async () => {
  try {
    if (!currentSsoAddress) {
      throw new Error("No account logged in");
    }

    console.log(`[Step4ConfirmNow] Starting confirmation. Account to guard: ${currentSsoAddress}, Guardian address: ${props.guardianAddress}, Is SSO: ${isSsoAccount.value}`);

    let client: Parameters<typeof confirmGuardian>[0]["client"];

    // Check if guardian is the currently connected SSO account
    const isCurrentSsoGuardian = isSsoAccount.value
      && currentSsoAddress
      && props.guardianAddress.toLowerCase() === currentSsoAddress.toLowerCase();

    if (isCurrentSsoGuardian) {
      // Guardian is the current SSO account - use SSO client with paymaster
      console.log("[Step4ConfirmNow] Getting configurable account for current SSO guardian");
      const configurableAccount = await getConfigurableAccount({
        address: props.guardianAddress,
        usePaymaster: true,
      });
      if (!configurableAccount) {
        console.error(`[Step4ConfirmNow] No configurable account found for ${props.guardianAddress}`);
        throw new Error("No configurable account found");
      }
      console.log("[Step4ConfirmNow] Using SSO client:", configurableAccount.account.address);
      client = configurableAccount;
    } else {
      // Guardian is a different account - use WalletConnect
      if (!accountData.value.isConnected) {
        throw new Error("Please connect your wallet first");
      }
      console.log("[Step4ConfirmNow] Getting wallet client for guardian");
      client = await getWalletClient({ chainId: defaultChain.id });
      console.log("[Step4ConfirmNow] Using wallet client:", client.account.address);
    }

    console.log(`[Step4ConfirmNow] Calling confirmGuardian with client address: ${client.account.address}`);
    await confirmGuardian({
      client,
      accountToGuard: currentSsoAddress,
    });
    console.log("[Step4ConfirmNow] Guardian confirmed successfully");
    confirmGuardianErrorMessage.value = null;
    emit("next");
  } catch (err) {
    console.error("[Step4ConfirmNow] Error confirming guardian:", err);
    confirmGuardianErrorMessage.value = "An error occurred while confirming the guardian. Please try again.";
    // eslint-disable-next-line no-console
    console.error(err);
  }
};

watchEffect(() => {
  if (props.guardianAddress) {
    handleCheck();
  }
});

watchEffect(() => {
  if (isSsoAccount.value) {
    confirmGuardianErrorMessage.value = null;
    return;
  }

  if (!isSsoAccount.value && accountData.value.isConnected && isConnectedWalletGuardian.value) {
    confirmGuardianErrorMessage.value = null;
    return;
  }

  if (!isSsoAccount.value && accountData.value.isConnected && !isConnectedWalletGuardian.value) {
    confirmGuardianErrorMessage.value = `Please connect with the guardian wallet address (${shortenAddress(props.guardianAddress)})`;
    return;
  }
});
</script>
