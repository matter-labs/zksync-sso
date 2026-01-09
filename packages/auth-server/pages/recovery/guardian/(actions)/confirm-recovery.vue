<template>
  <div class="min-h-screen">
    <header class="max-w-[1920px] mx-auto mb-12">
      <app-generic-nav />
    </header>
    <main class="max-w-[900px] mx-auto flex flex-col gap-6">
      <account-recovery-confirm-action-card
        v-if="generalError"
        title="Error"
        type="error"
      >
        {{ generalError }}
      </account-recovery-confirm-action-card>

      <template v-else>
        <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-3">
          Recovery Request Review
        </h1>
        <p class="text-lg text-gray-600 dark:text-gray-400">
          Review the recovery details below:
        </p>

        <div class="flex flex-col gap-4">
          <account-recovery-confirm-info-card title="Account Address">
            <span class="mr-2 font-mono text-lg">{{ recoveryParams?.accountAddress }}</span>
            <common-copy-to-clipboard
              class="!inline-flex"
              :text="recoveryParams?.accountAddress ?? ''"
            />
          </account-recovery-confirm-info-card>

          <account-recovery-confirm-info-card title="Recovery Credentials">
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
          </account-recovery-confirm-info-card>

          <account-recovery-confirm-action-card
            v-if="!recoveryCompleted"
            :title="selectedGuardian ? 'Guardian Selected' : 'Select Guardian'"
            :type="selectedGuardian ? 'success' : 'warning'"
          >
            <account-recovery-account-select
              v-model="selectedGuardian"
              :accounts="guardians?.map((x) => x.address) ?? []"
              class="max-w-fit w-full"
            />
            <div
              v-if="selectedGuardianInfo"
              class="text-xs font-mono mt-2"
            >
              {{ selectedGuardianInfo.isSsoAccount ? "ZKsync SSO Account" : "Standard Account" }}
            </div>
          </account-recovery-confirm-action-card>

          <template v-if="selectedGuardian && !recoveryCompleted">
            <div class="flex gap-4 mt-3">
              <ZkButton
                v-if="selectedGuardianInfo?.isSsoAccount || isConnectedWalletGuardian"
                type="primary"
                class="w-full max-w-56"
                :loading="initRecoveryInProgress || getConfigurableAccountInProgress"
                @click="handleConfirmRecovery"
              >
                Confirm Recovery
              </ZkButton>
              <CommonConnectButton
                v-if="!selectedGuardianInfo?.isSsoAccount"
                type="primary"
                class="w-full max-w-56"
                :disabled="initRecoveryInProgress || getConfigurableAccountInProgress"
              />
            </div>
            <p
              v-if="!selectedGuardianInfo?.isSsoAccount && accountData.isConnected && !isConnectedWalletGuardian"
              class="text-error-500 dark:text-error-400"
            >
              Please connect with the guardian wallet address ({{ shortenAddress(selectedGuardian) }})
            </p>
          </template>

          <account-recovery-confirm-action-card
            v-if="recoveryCompleted"
            title="Done!"
            type="success"
          >
            The account will be ready to use with the new credentials in 24hrs.
          </account-recovery-confirm-action-card>
        </div>
      </template>
    </main>
  </div>
</template>

<script setup lang="ts">
import { useAppKitAccount } from "@reown/appkit/vue";
import { type Address, encodeAbiParameters, isAddressEqual, keccak256, pad, parseAbiParameters, toHex } from "viem";
import { base64urlToUint8Array, getPasskeySignatureFromPublicKeyBytes, getPublicKeyBytesFromPasskeySignature } from "zksync-sso-4337/utils";
import { z } from "zod";

import { uint8ArrayToHex } from "@/utils/formatters";
import { AddressSchema } from "@/utils/schemas";

const accountData = useAppKitAccount();
const { getRecovery, initRecovery, initRecoveryInProgress, getGuardians } = useRecoveryGuardian();
const { getWalletClient, defaultChain } = useClientStore();
const { isSsoAccount: checkIsSsoAccount } = useIsSsoAccount();
const route = useRoute();
const { getConfigurableAccount, getConfigurableAccountInProgress } = useConfigurableAccount();

definePageMeta({
  layout: "dashboard",
});

const RecoveryParamsSchema = z
  .object({
    accountAddress: AddressSchema,
    credentialId: z.string().min(1),
    credentialPublicKey: z.string().min(1),
    checksum: z.string().min(1),
  })
  .refine(
    async (data) => {
      // Calculate checksum
      // Normalize accountAddress to lowercase for consistent hashing
      const normalizedAddress = data.accountAddress.toLowerCase();
      const dataToHash = `${normalizedAddress}:${data.credentialId}:${data.credentialPublicKey}`;

      const calculatedChecksum = uint8ArrayToHex(
        new Uint8Array(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(dataToHash))).slice(0, 8),
      );

      return calculatedChecksum === data.checksum;
    },
    {
      message: "Invalid recovery data checksum",
    },
  );

const generalError = ref<string | null>(null);
const recoveryCheckTrigger = ref(0);

const isLoadingGuardians = ref(false);
const loadingGuardiansError = ref<string | null>(null);

const isConnectedWalletGuardian = computed(() => (
  accountData.value.isConnected && isAddressEqual(selectedGuardian.value, accountData.value.address as Address)
));

const confirmGuardianErrorMessage = ref<string | null>(null);

const recoveryParams = computedAsync(async () => RecoveryParamsSchema.parseAsync({
  accountAddress: route.query.accountAddress,
  credentialId: route.query.credentialId,
  credentialPublicKey: route.query.credentialPublicKey,
  checksum: route.query.checksum,
}).catch((err) => {
  // eslint-disable-next-line no-console
  console.error(err);
  generalError.value = "Invalid recovery parameters. Please verify the URL and try again.";
}));

const recoveryCompleted = computedAsync(async () => {
  // Force re-evaluation when trigger changes
  const _triggerValue = recoveryCheckTrigger.value;

  if (!recoveryParams.value?.accountAddress || !recoveryParams.value?.credentialId || !recoveryParams.value?.credentialPublicKey) {
    return false;
  }

  const result = await getRecovery(recoveryParams.value.accountAddress);

  // The smart contract stores keccak256(data) where data is the encoded recovery payload
  // We need to reconstruct the same data structure that was passed to initializeRecovery
  const parsedPublicKey = JSON.parse(recoveryParams.value.credentialPublicKey);
  const credentialPublicKeyBytes = getPasskeySignatureFromPublicKeyBytes([parsedPublicKey.x, parsedPublicKey.y]);
  const publicKeyBytes = getPublicKeyBytesFromPasskeySignature(credentialPublicKeyBytes);
  const publicKeyHex = [
    pad(`0x${publicKeyBytes[0].toString("hex")}`),
    pad(`0x${publicKeyBytes[1].toString("hex")}`),
  ] as const;

  const recoveryData = encodeAbiParameters(
    parseAbiParameters("bytes32 credentialIdHash, bytes32[2] publicKey"),
    [
      keccak256(toHex(base64urlToUint8Array(recoveryParams.value.credentialId))),
      publicKeyHex,
    ],
  );

  const expectedHashedData = keccak256(recoveryData);

  const isComplete = result?.hashedData === expectedHashedData;

  return isComplete;
});

const guardians = computedAsync(async () => {
  isLoadingGuardians.value = true;
  loadingGuardiansError.value = null;

  if (!recoveryParams.value?.accountAddress) return [];

  try {
    const result = await getGuardians(recoveryParams.value?.accountAddress);
    return await Promise.all(
      result
        .filter((guardian) => guardian.isReady)
        .map(async (guardian) => ({
          address: guardian.addr,
          isSsoAccount: !!(await checkIsSsoAccount(guardian.addr)),
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

const selectedGuardian = ref<Address>("" as Address);
const selectedGuardianInfo = computed(() => selectedGuardian.value && guardians.value?.find((guardian) => isAddressEqual(guardian.address, selectedGuardian.value)));

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

    if (!recoveryParams.value) return;

    // Parse the credentialPublicKey JSON string to get x,y coordinates
    const parsedPublicKey = JSON.parse(recoveryParams.value.credentialPublicKey);
    // Convert coordinates to proper COSE format expected by initRecovery
    const credentialPublicKeyBytes = getPasskeySignatureFromPublicKeyBytes([parsedPublicKey.x, parsedPublicKey.y]);

    await initRecovery({
      client,
      accountToRecover: recoveryParams.value.accountAddress,
      credentialPublicKey: credentialPublicKeyBytes,
      credentialId: recoveryParams.value.credentialId,
    });
    confirmGuardianErrorMessage.value = null;

    // Wait a moment for the transaction to be processed
    await new Promise((resolve) => setTimeout(resolve, 2000));

    // Trigger re-check of recoveryCompleted
    recoveryCheckTrigger.value++;
  } catch (err) {
    confirmGuardianErrorMessage.value = "An error occurred while confirming the guardian. Please try again.";
    // eslint-disable-next-line no-console
    console.error(err);
  }
};
</script>
