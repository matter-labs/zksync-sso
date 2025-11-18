<template>
  <div
    v-if="isLoadingGuardians"
    class="flex flex-col items-center gap-4"
  >
    <common-spinner class="w-8 h-8" />
    <p class="text-center text-gray-600 dark:text-gray-400">
      Checking guardian account...
    </p>
  </div>

  <div
    v-else-if="loadingGuardiansError"
    class="flex flex-col items-center gap-4"
  >
    <p class="text-center text-error-500 dark:text-error-400">
      An error occurred while checking the guardian account. Please try again later.
    </p>
  </div>

  <div
    v-else
    class="flex flex-col items-center gap-4"
  >
    <p class="text-center text-gray-600 dark:text-gray-400">
      Please select and confirm with your guardian account to continue.
    </p>

    <p
      v-if="confirmGuardianErrorMessage"
      class="text-center text-error-500 dark:text-error-400 mt-4"
    >
      {{ confirmGuardianErrorMessage }}
    </p>

    <account-recovery-account-select
      v-model="selectedGuardian"
      :accounts="guardians?.map((guardian) => guardian.address) ?? []"
      :disabled="isLoadingGuardians"
      placeholder="Select a guardian"
    />

    <template v-if="selectedGuardian">
      <p
        v-if="!selectedGuardianInfo?.isSsoAccount && accountData.isConnected && !isConnectedWalletGuardian"
        class="text-center text-error-500 dark:text-error-400 mt-4"
      >
        Please connect with the guardian wallet address ({{ shortenAddress(selectedGuardian) }})
      </p>
      <ZkButton
        v-if="selectedGuardianInfo?.isSsoAccount || isConnectedWalletGuardian"
        type="primary"
        class="w-full mt-4"
        :loading="initRecoveryInProgress || getConfigurableAccountInProgress"
        @click="handleConfirmRecovery"
      >
        Confirm Recovery
      </ZkButton>
      <CommonConnectButton
        v-if="!selectedGuardianInfo?.isSsoAccount"
        type="primary"
        class="w-full mt-4"
        :disabled="initRecoveryInProgress || getConfigurableAccountInProgress"
      />
    </template>

    <ZkButton
      type="secondary"
      class="w-full"
      :disabled="initRecoveryInProgress || getConfigurableAccountInProgress"
      @click="emit('back')"
    >
      Back
    </ZkButton>
  </div>
</template>

<script setup lang="ts">
import { useAppKitAccount } from "@reown/appkit/vue";
import { type Address, isAddressEqual } from "viem";

import type { RegisterNewPasskeyReturnType } from "~/composables/usePasskeyRegister";

const { getWalletClient, defaultChain } = useClientStore();
const { getGuardians, initRecovery, initRecoveryInProgress } = useRecoveryGuardian();
const { isSsoAccount } = useIsSsoAccount();
const { getConfigurableAccount, getConfigurableAccountInProgress } = useConfigurableAccount();
const accountData = useAppKitAccount();

const props = defineProps<{
  accountAddress: Address;
  newPasskey: RegisterNewPasskeyReturnType;
}>();

const emit = defineEmits<{
  (e: "back" | "next"): void;
}>();

const confirmGuardianErrorMessage = ref<string | null>(null);
const isLoadingGuardians = ref(false);
const loadingGuardiansError = ref<string | null>(null);
const selectedGuardian = ref<Address>("" as Address);
const selectedGuardianInfo = computed(() => guardians.value?.find((guardian) => isAddressEqual(guardian.address, selectedGuardian.value)));
const isConnectedWalletGuardian = computed(() => (
  accountData.value.isConnected && isAddressEqual(selectedGuardian.value, accountData.value.address as Address)
));

const guardians = computedAsync(async () => {
  isLoadingGuardians.value = true;
  loadingGuardiansError.value = null;
  try {
    const result = await getGuardians(props.accountAddress as Address);
    return await Promise.all(
      result
        .filter((guardian) => guardian.isReady)
        .map(async (guardian) => ({
          address: guardian.addr,
          isSsoAccount: (await isSsoAccount(guardian.addr)),
        })),
    );
  } catch (err) {
    loadingGuardiansError.value = "An error occurred while loading the guardians. Please try again.";
    // eslint-disable-next-line no-console
    console.error(err);
  } finally {
    isLoadingGuardians.value = false;
  }
});

const handleConfirmRecovery = async () => {
  try {
    let client: Parameters<typeof initRecovery>[0]["client"];

    if (selectedGuardianInfo.value?.isSsoAccount) {
      const configurableAccount = await getConfigurableAccount({ address: selectedGuardian.value });
      if (!configurableAccount) {
        throw new Error("No configurable account found");
      }
      client = configurableAccount;
    } else {
      client = await getWalletClient({ chainId: defaultChain.id });
    }

    await initRecovery({
      client,
      accountToRecover: props.accountAddress,
      credentialPublicKey: props.newPasskey.credentialPublicKey,
      accountId: props.newPasskey.credentialId,
    });
    confirmGuardianErrorMessage.value = null;
    emit("next");
  } catch (err) {
    confirmGuardianErrorMessage.value = "An error occurred while confirming the guardian. Please try again.";
    // eslint-disable-next-line no-console
    console.error(err);
  }
};
</script>
