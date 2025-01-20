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
</template>

<script setup lang="ts">
import { createWalletClient, http, type Address, type Chain } from "viem";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";
import { deployAccount } from "zksync-sso/client";
import { registerNewPasskey } from "zksync-sso/client/passkey";

const { appMeta, userDisplay, userId, contracts, deployerKey } = useAppMeta();
const isLoading = ref(false);

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
  if (!appMeta.value || !appMeta.value.credentialPublicKey) {
    try {
      const { credentialPublicKey: newCredentialPublicKey } = await registerNewPasskey({
        userDisplayName: userDisplay, // Display name of the user
        userName: userId, // User's unique ID
      });
      appMeta.value = {
        ...appMeta.value,
        credentialPublicKey: u8ToString(newCredentialPublicKey),
      };
      return newCredentialPublicKey;
    } catch (error) {
      console.error("Passkey registration failed:", error);
      return;
    }
  } else {
    return new Uint8Array(JSON.parse(appMeta.value.credentialPublicKey));
  }
};

const getDeployerClient = async () => {
  const config = useRuntimeConfig();
  return createWalletClient({
    account: privateKeyToAccount(deployerKey as Address),
    chain: config.public.network as Chain,
    transport: http()
  });
};

const createAccountWithPasskey = async () => {
  const publicPassKey = await getPublicPasskey();
  if (!publicPassKey) {
    return false;
  }

  // Configure deployer account to pay for Account creation
  const deployerClient = await getDeployerClient();

  try {
    const { address, transactionReceipt } = await deployAccount(deployerClient, {
      credentialPublicKey: publicPassKey,
      contracts,
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
    // create account with EOA owner
    const privateKey = generatePrivateKey();
    const account = privateKeyToAccount(privateKey);
    
    const deployerClient = await getDeployerClient();
    const { address } = await deployAccount(deployerClient, {
      credentialPublicKey: new Uint8Array(),
      ownerPublicKeys: [account.address],
      contracts,
    });

    appMeta.value = {
      ...appMeta.value,
      cryptoAccountAddress: address,
    };
  };

  navigateTo("/crypto-account");
};
</script>
