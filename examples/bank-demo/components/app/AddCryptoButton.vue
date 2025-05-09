<template>
  <ZkButton
    type="primary"
    class="w-52"
    :ui="{base: 'py-0'}"
    :disabled="isLoading"
    @click="onClickAddCrypto"
  >
    <span v-if="!isLoading">Add Crypto account</span>
    <CommonSpinner v-else class="h-6"/>
  </ZkButton>
  <div>
    <NoPasskeyDialog v-if="showModal" />
  </div>
</template>

<script setup lang="ts">
import NoPasskeyDialog from "~/components/app/NoPasskeyDialog.vue";
import type { Hex } from "viem";
import { deployAccount } from "zksync-sso/client";
import { registerNewPasskey } from "zksync-sso/client/passkey";
import { getDeployerClient } from "../common/CryptoDeployer";

const { appMeta, userDisplay, userId, contracts, deployerKey } = useAppMeta();
const isLoading = ref(false);
const showModal = ref(false);

// Convert Uin8Array to string
const u8ToString = (input: Uint8Array): string => {
  const str = JSON.stringify(Array.from ? Array.from(input) : [].map.call(input, (v => v)));
  return str;
};

const onClickAddCrypto = async () => {
  isLoading.value = true;
  await createCryptoAccount();
  isLoading.value = false;
};

const getPublicPasskey = async () => {
  // Create new Passkey
  if (!appMeta.value || !appMeta.value.credentialPublicKey || !appMeta.value.credentialId) {
    try {
      const newPasskey = await registerNewPasskey({
        userDisplayName: userDisplay, // Display name of the user
        userName: userId, // User's unique ID
      });
      appMeta.value = {
        ...appMeta.value,
        credentialPublicKey: u8ToString(newPasskey.credentialPublicKey),
        credentialId: newPasskey.credentialId,
      };
      return newPasskey;
    } catch (error) {
      console.error("Passkey registration failed:", error);
      return false;
    }
  } else {
    return { 
      credentialPublicKey: new Uint8Array(JSON.parse(appMeta.value.credentialPublicKey)),
      credentialId: appMeta.value.credentialId,
    };
  }
};

const createAccountWithPasskey = async () => {
  const publicPassKey = await getPublicPasskey();
  if (!publicPassKey) {
    return false;
  }

  // Configure deployer account to pay for Account creation
  const deployerClient = await getDeployerClient(deployerKey as Hex);

  try {
    const { address, transactionReceipt } = await deployAccount(deployerClient, {
      passkeyModule: {
        location: contracts.passkey,
        credentialPublicKey: publicPassKey.credentialPublicKey,
        credentialId: publicPassKey.credentialId,
      },
      accountFactory: contracts.accountFactory,
      installNoDataModules: [],
      // installing creator as an account owner!
      owners: [deployerClient.account.address],
    });

    appMeta.value = {
      ...appMeta.value,
      cryptoAccountAddress: address,
    };
    console.log(`Successfully created account: ${address}`);
    console.log(`Transaction receipt: ${transactionReceipt.transactionHash}`);
    return true;
  } catch (error) {
    console.error(error);
    return false;
  }
};

const createCryptoAccount = async () => {
  if (!await createAccountWithPasskey()) {
    showModal.value = true;
  } else {
    navigateTo("/crypto-account");
  };
};
</script>
