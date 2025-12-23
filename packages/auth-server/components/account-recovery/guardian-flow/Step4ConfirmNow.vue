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
const { address } = useAccountStore();
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
    if (!address) {
      throw new Error("No account logged in");
    }

    let client: Parameters<typeof confirmGuardian>[0]["client"];

    if (isSsoAccount.value) {
      const configurableAccount = await getConfigurableAccount({ address: props.guardianAddress });
      if (!configurableAccount) {
        throw new Error("No configurable account found");
      }
      client = configurableAccount;
    } else {
      client = await getWalletClient({ chainId: defaultChain.id });
    }

    await confirmGuardian({
      client,
      accountToGuard: address,
    });
    confirmGuardianErrorMessage.value = null;
    emit("next");
  } catch (err) {
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
