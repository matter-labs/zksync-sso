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
          <div class="absolute inset-0 bg-gray-500 opacity-75" />
        </div>

        <span class="hidden sm:inline-block sm:align-middle sm:h-screen" aria-hidden="true">&#8203;</span>

        <div
          class="inline-block align-bottom bg-white rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full">
          <div class="bg-white px-4 pt-5 pb-4 sm:p-6 sm:pb-4">
            <div class="sm:flex sm:items-start">
              <div
                class="mx-auto flex-shrink-0 flex items-center justify-center h-12 w-12 rounded-full bg-blue-100 sm:mx-0 sm:h-10 sm:w-10">
                <svg
class="h-6 w-6 text-blue-600" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24"
                  stroke="currentColor" aria-hidden="true">
                  <path
stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                </svg>
              </div>
              <div class="mt-3 text-center sm:mt-0 sm:ml-4 sm:text-left">
                <h3 class="text-lg leading-6 font-medium text-gray-900">
                  Create Crypto Account
                </h3>
                <p>
                  Choose a password to protect your account
                </p>
                <div class="mt-2 relative">
                  <input
v-model="password" :type="showPassword ? 'text' : 'password'" placeholder="Password"
                    class="w-full border border-gray-300 px-3 py-2 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm">
                  <button
type="button" class="absolute inset-y-0 right-0 pr-3 flex items-center text-gray-500"
                    @click="showPassword = !showPassword">
                    <svg
v-if="showPassword" class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"
                      xmlns="http://www.w3.org/2000/svg">
                      <path
stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                        d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                      <path
stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                        d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                    </svg>
                    <svg
v-else class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"
                      xmlns="http://www.w3.org/2000/svg">
                      <path
stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                        d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
                    </svg>
                  </button>
                </div>
                <div class="mt-2 relative">
                  <input
v-model="confirmPassword" :type="showConfirmPassword ? 'text' : 'password'"
                    placeholder="Confirm Password"
                    class="w-full border border-gray-300 px-3 py-2 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm">
                  <button
type="button" class="absolute inset-y-0 right-0 pr-3 flex items-center text-gray-500"
                    @click="showConfirmPassword = !showConfirmPassword">
                    <svg
v-if="showConfirmPassword" class="h-5 w-5" fill="none" stroke="currentColor"
                      viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                      <path
stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                        d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                      <path
stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                        d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                    </svg>
                    <svg
v-else class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"
                      xmlns="http://www.w3.org/2000/svg">
                      <path
stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                        d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
                    </svg>
                  </button>
                </div>
                <div v-if="showMatchError" class="text-red-500 text-sm mt-2">Passwords do not match!</div>
              </div>
            </div>
          </div>
          <div class="bg-gray-50 px-4 py-3 sm:px-6 sm:flex sm:flex-row-reverse">
            <button
type="button"
              class="w-full inline-flex justify-center rounded-md border border-transparent shadow-sm px-4 py-2 bg-blue-600 text-base font-medium text-white hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 sm:ml-3 sm:w-auto sm:text-sm"
              @click="checkPassword">
              Confirm
            </button>
            <button
type="button"
              class="mt-3 w-full inline-flex justify-center rounded-md border border-gray-300 shadow-sm px-4 py-2 bg-white text-base font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 sm:mt-0 sm:ml-3 sm:w-auto sm:text-sm"
              @click="$emit('cancel')">
              Cancel
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { toHex, type HDAccount, type Hex } from "viem";
import { english, generateMnemonic, mnemonicToAccount } from "viem/accounts";
import { deployAccount, fetchAccount } from "zksync-sso/client/ecdsa";
import { getDeployerClient } from "../common/CryptoDeployer";

const { appMeta, contracts, deployerKey } = useAppMeta();

const password = ref("");
const showPassword = ref(false);
const confirmPassword = ref("");
const showConfirmPassword = ref(false);
const showMatchError = ref(false);
const account = ref<HDAccount>();
const mnemonic = ref<string>();

// create account with EOA owner
onMounted(() => {
  mnemonic.value = generateMnemonic(english);
  account.value = mnemonicToAccount(mnemonic.value);
});

async function encryptMessage(cryptoSecretKey: string, password: string, iv: Uint8Array, salt: Uint8Array) {
  // hashed pin/password
  const keyMaterial = await window.crypto.subtle.importKey(
    "raw",
    new TextEncoder().encode(password),
    "PBKDF2",
    false,
    ["deriveBits", "deriveKey"],
  );
  const keyEncryptionKey = await window.crypto.subtle.deriveKey(
    {
      name: "PBKDF2",
      salt,
      iterations: 100000,
      hash: "SHA-256",
    },
    keyMaterial,
    { name: "AES-GCM", length: 256 },
    true,
    ["encrypt", "decrypt"],
  );
  const encryptedCryptoSecret = await window.crypto.subtle.encrypt(
    {
      name: "AES-GCM",
      iv: iv
    },
    keyEncryptionKey,
    new TextEncoder().encode(cryptoSecretKey),
  );

  return { encryptedCryptoSecret, keyEncryptionKey };
}

async function decryptMessage(encryptedKey: ArrayBuffer, key: CryptoKey, iv: Uint8Array) {
  const decryptedBytes = await window.crypto.subtle.decrypt({ name: "AES-GCM", iv }, key, encryptedKey);
  return new TextDecoder().decode(decryptedBytes);
}

async function checkPassword() {
  if (!mnemonic.value) {
    console.warn("Private key not set");
    return;
  }
  if (!account.value) {
    console.warn("account not ready");
    return;
  }
  if (password.value !== confirmPassword.value) {
    console.warn("no password match");
    showMatchError.value = true;
    return;
  }
  // TODO: get iv and salt from server!
  const iv = new Uint8Array(12);
  const salt = new Uint8Array(16);
  if (!password.value) {
    console.warn("Pin not set");
    return;
  }
  const encrypted = await encryptMessage(mnemonic.value, password.value, iv, salt);
  if (!encrypted) {
    console.error("Encryption failed", mnemonic.value);
    return;
  }
  const testDecrypted = await decryptMessage(
    encrypted.encryptedCryptoSecret, encrypted.keyEncryptionKey, iv);
  if (mnemonic.value != testDecrypted) {
    console.error("Decryption failed", mnemonic.value, testDecrypted);
    return;
  }
  console.log("Decryption success", encrypted);

  const address = await deployAddress(account.value);
  const pk = account.value.getHdKey().privateKey;
  if (!pk) {
    console.warn("account has no private key!");
    return;
  }

  appMeta.value = {
    ...appMeta.value,
    cryptoAccountAddress: address,
    privateKey: toHex(pk),
  };
  
  downloadArrayBuffer("encrypted-private-key.txt", encrypted.encryptedCryptoSecret);
  navigateTo("/crypto-account");
  confirm();
}

async function deployAddress(accountAddress: HDAccount) {
  const deployerClient = await getDeployerClient(deployerKey as Hex);
  try {
    const fetchedAccount = await fetchAccount(deployerClient, {
      contracts,
      prefix: "bank-demo",
      owner: accountAddress.address,
    });
    return fetchedAccount.address;
  } catch (err) {
    console.info("account does not exist, deploy!", err);
    const deployedAccount = await deployAccount(deployerClient, {
      contracts,
      prefix: "bank-demo",
      owner: accountAddress.address,
    });
    return deployedAccount.address;
  }
}

function downloadArrayBuffer(filename: string, buffer: ArrayBuffer, mimeType: string = "application/octet-stream") {
  const blob = new Blob([buffer], { type: mimeType });
  const url = window.URL.createObjectURL(blob);

  const a = document.createElement("a");
  a.href = url;
  a.download = filename;

  a.style.display = "none";
  document.body.appendChild(a);

  a.click();

  window.URL.revokeObjectURL(url);
  document.body.removeChild(a);
}

defineEmits(["confirm", "cancel"]);
</script>