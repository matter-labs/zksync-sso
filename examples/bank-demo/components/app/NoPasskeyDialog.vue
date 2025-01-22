<!--
 Copyright 2025 cbe
 
 Licensed under the Apache License, Version 2.0 (the "License");
 you may not use this file except in compliance with the License.
 You may obtain a copy of the License at
 
     https://www.apache.org/licenses/LICENSE-2.0
 
 Unless required by applicable law or agreed to in writing, software
 distributed under the License is distributed on an "AS IS" BASIS,
 WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 See the License for the specific language governing permissions and
 limitations under the License.
-->

<template>
  <div>
    <div class="fixed inset-0 z-50 overflow-y-auto">
      <div class="flex items-center justify-center min-h-screen px-4 pt-4 pb-20 text-center sm:block sm:p-0">
        <div class="fixed inset-0 transition-opacity" aria-hidden="true">
          <div class="absolute inset-0 bg-gray-500 opacity-75"/>
        </div>

        <span class="hidden sm:inline-block sm:align-middle sm:h-screen" aria-hidden="true">&#8203;</span>

        <div class="inline-block align-bottom bg-white rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full">
          <div class="bg-white px-4 pt-5 pb-4 sm:p-6 sm:pb-4">
            <div class="sm:flex sm:items-start">
              <div class="mx-auto flex-shrink-0 flex items-center justify-center h-12 w-12 rounded-full bg-blue-100 sm:mx-0 sm:h-10 sm:w-10">
                <svg class="h-6 w-6 text-blue-600" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                </svg>
              </div>
              <div class="mt-3 text-center sm:mt-0 sm:ml-4 sm:text-left">
                <h3 class="text-lg leading-6 font-medium text-gray-900">
                  Enter Password
                </h3>
                <div class="mt-2">
                  <p class="text-sm text-gray-500">
                    Please enter your password to continue.
                  </p>
                  <input v-model="password" type="password" class="mt-2 w-full border border-gray-300 px-3 py-2 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm" >
                </div>
              </div>
            </div>
          </div>
          <div class="bg-gray-50 px-4 py-3 sm:px-6 sm:flex sm:flex-row-reverse">
            <button type="button" class="w-full inline-flex justify-center rounded-md border border-transparent shadow-sm px-4 py-2 bg-blue-600 text-base font-medium text-white hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 sm:ml-3 sm:w-auto sm:text-sm" @click="checkPassword">
              Confirm
            </button>
            <button type="button" class="mt-3 w-full inline-flex justify-center rounded-md border border-gray-300 shadow-sm px-4 py-2 bg-white text-base font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 sm:mt-0 sm:ml-3 sm:w-auto sm:text-sm" @click="$emit('cancel')">
              Cancel
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Hex, PrivateKeyAccount } from "viem";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";
import { deployAccount } from "zksync-sso/client";
import { getDeployerClient } from "../common/CrpytoDeployer";

const { appMeta, contracts, deployerKey } = useAppMeta();

const password = ref("");
const account = ref<PrivateKeyAccount>();
const privateKey = ref<Hex>();

// create account with EOA owner
onMounted(() => {
    privateKey.value = generatePrivateKey();
    account.value = privateKeyToAccount(privateKey.value);
});

async function encryptMessage(cryptoPrivateKey: string) {
  if (!password.value) {
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
  const hashedPin = await window.crypto.subtle.digest("SHA-512", new TextEncoder().encode(password.value));
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

async function checkPassword() {
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
    
  const deployerClient = await getDeployerClient(deployerKey as Hex);
  const { address } = await deployAccount(deployerClient, {
    credentialPublicKey: new Uint8Array(),
    ownerPublicKeys: [account.value?.address],
    contracts,
  });

  appMeta.value = {
    ...appMeta.value,
    cryptoAccountAddress: address,
  };
  navigateTo("/crypto-account");
  confirm();
}

defineEmits(["confirm", "cancel"]);
</script>