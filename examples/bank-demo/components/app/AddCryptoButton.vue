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
    <div v-if="showModal" class="modal-overlay">
      <div class="modal-content">
        <h2>Enter PIN/Password</h2>
        <input v-model="enteredPin" type="password" >
        <button @click="checkPin">Confirm</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { PrivateKeyAccount, Address, Chain, Hex } from "viem";
import { createWalletClient, http } from "viem";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";
import { deployAccount } from "zksync-sso/client";
import { registerNewPasskey } from "zksync-sso/client/passkey";

const { appMeta, userDisplay, userId, contracts, deployerKey } = useAppMeta();
const isLoading = ref(false);

const showModal = ref(false);
const enteredPin = ref("");
const account = ref<PrivateKeyAccount>();
const privateKey = ref<Hex>();

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

async function encryptMessage(cryptoPrivateKey: string) {
  if (!enteredPin.value) {
    console.warn("Pin not set");
    return;
  }
  const keyEncryptionKey = await window.crypto.subtle.generateKey(
    {
      name: "AES-CBC",
      length: 256,
    },
    true,
    ["encrypt", "decrypt"],
  );
  const encoded = new TextEncoder().encode(cryptoPrivateKey);
  // TODO: get a fixed salt from the backend here
  const hashedPin = await window.crypto.subtle.digest("SHA-512", new TextEncoder().encode(enteredPin.value));
  const iv = new Uint8Array(hashedPin.slice(0, 16));
  const encryptedKey = await window.crypto.subtle.encrypt(
    {
      name: "AES-CBC",
      iv: iv,
    },
    keyEncryptionKey,
    encoded,
  );

  return { encryptedKey, keyEncryptionKey, iv: iv };
}

async function decryptMessage(encryptedKey: ArrayBuffer, key: CryptoKey, iv: Uint8Array) {
  const decryptedBytes = await window.crypto.subtle.decrypt({ name: "AES-CBC", iv }, key, encryptedKey);
  return new TextDecoder().decode(decryptedBytes);
}

async function checkPin() {
  if (!privateKey.value) {
    console.warn("Private key not set");
    return;
  }
  const encrypted = await encryptMessage(privateKey.value);
  if (!encrypted) {
    console.error("Encryption failed", privateKey.value);
    return;
  }
  const testDecrypted = await decryptMessage(encrypted.encryptedKey, encrypted.keyEncryptionKey, encrypted.iv);
  if (privateKey.value != testDecrypted) {
    console.error("Decryption failed", privateKey.value, testDecrypted);
    return;
  }
  console.log("Decryption success", encrypted);
  // export the encrypted key
    
  const deployerClient = await getDeployerClient();
  const { address } = await deployAccount(deployerClient, {
    credentialPublicKey: new Uint8Array(),
    ownerPublicKeys: [account.value?.address],
    contracts,
  });

  appMeta.value = {
    ...appMeta.value,
    cryptoAccountAddress: address,
  };
  showModal.value = true;
  navigateTo("/crypto-account");
}

const createCryptoAccount = async () => {
  
  if (!await createAccountWithPasskey()) {
    // create account with EOA owner
    privateKey.value = generatePrivateKey();
    account.value = privateKeyToAccount(privateKey.value);
    showModal.value = true;
  } else {
    navigateTo("/crypto-account");
  };

};
</script>
