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
  <div class="container mx-auto p-4 font-sans">
    <header class="mb-8 text-center">
      <h1 class="text-4xl font-bold text-blue-600">
        EIP-712 Listing Signer
      </h1>
      <p class="text-gray-600">
        Generate an EIP-712 signature for a marketplace ListRequest.
      </p>
    </header>

    <div class="grid grid-cols-1 gap-8 md:grid-cols-2">
      <div class="rounded-lg border border-gray-200 bg-white p-6 shadow-lg">
        <h2 class="mb-6 border-b pb-2 text-2xl font-semibold text-gray-700">
          Listing Details
        </h2>

        <div class="space-y-6">
          <h3 class="mt-4 border-t pt-4 text-lg font-medium text-gray-800">
            Marketplace Info
          </h3>

          <h3 class="mt-4 border-t pt-4 text-lg font-medium text-gray-800">
            NFT & Seller Info
          </h3>
          <div>
            <label
              for="smartAccountAddress"
              class="mb-1 block text-sm font-medium text-gray-700"
            >Seller Smart Account Address:</label>
            <input
              id="smartAccountAddress"
              v-model="account.address"
              type="text"
              placeholder="0x..."
              disabled
              class="block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 sm:text-sm"
            >
          </div>
          <div>
            <label
              for="priceInEth"
              class="mb-1 block text-sm font-medium text-gray-700"
            >Price (ETH):</label>
            <input
              id="priceInEth"
              v-model="config.priceInEth"
              type="text"
              placeholder="e.g., 0.1"
              class="block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 sm:text-sm"
            >
          </div>
          <div>
            <label
              for="sellerNonce"
              class="mb-1 block text-sm font-medium text-gray-700"
            >Seller Nonce (from Marketplace):</label>
            <input
              id="sellerNonce"
              v-model="config.sellerNonce"
              type="text"
              placeholder="e.g., 0"
              class="block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 sm:text-sm"
            >
            <p class="mt-1 text-xs text-gray-500">
              Fetch this from `marketplace.sellerNonces(sellerAddress)`.
            </p>
          </div>
        </div>

        <button
          :disabled="isLoading"
          class="mt-8 w-full rounded-lg bg-blue-600 px-6 py-3 text-base font-medium text-white shadow-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:opacity-50"
          @click="handleSignListRequest"
        >
          <span v-if="isLoading">Signing...</span>
          <span v-else>Sign Listing Request</span>
        </button>

        <div
          v-if="errorMsg"
          class="mt-4 rounded-md bg-red-50 p-4 text-sm text-red-700"
        >
          <p>Error: {{ errorMsg }}</p>
        </div>
      </div>

      <div
        class="overflow-hidden rounded-lg border border-gray-200 bg-gray-50 p-6 shadow-lg"
      >
        <h2 class="mb-6 border-b pb-2 text-2xl font-semibold text-gray-700">
          Signature & Data
        </h2>
        <div
          v-if="signature"
          class="space-y-4"
        >
          <div>
            <h3 class="text-sm font-medium text-gray-500">
              Signature:
            </h3>
            <pre
              class="mt-1 block w-full overflow-x-auto rounded-md bg-gray-100 p-3 text-sm text-gray-800 shadow-sm"
            >{{ signature }}</pre>
          </div>
          <div>
            <h3 class="text-sm font-medium text-gray-500">
              Domain:
            </h3>
            <pre
              class="mt-1 block w-full overflow-x-auto rounded-md bg-gray-100 p-3 text-sm text-gray-800 shadow-sm"
            >{{ JSON.stringify(signedDomain, null, 2) }}</pre>
          </div>
          <div>
            <h3 class="text-sm font-medium text-gray-500">
              Types:
            </h3>
            <pre
              class="mt-1 block w-full overflow-x-auto rounded-md bg-gray-100 p-3 text-sm text-gray-800 shadow-sm"
            >{{ JSON.stringify(signedTypes, null, 2) }}</pre>
          </div>
          <div>
            <h3 class="text-sm font-medium text-gray-500">
              Message (Value):
            </h3>
            <pre
              class="mt-1 block w-full overflow-x-auto rounded-md bg-gray-100 p-3 text-sm text-gray-800 shadow-sm"
            >{{ JSON.stringify(signedMessage, bigIntReplacer, 2) }}</pre>
          </div>
        </div>
        <div
          v-else
          class="text-center text-gray-500"
        >
          <p>Signature will appear here once generated.</p>
        </div>
      </div>
    </div>

    <footer class="mt-12 border-t pt-8 text-center text-sm text-gray-500">
      <p>
        This tool is for demonstration purposes. Ensure all inputs are correct
        and secure.
      </p>
      <p>
        Using Viem for EIP-712 signing. Requires a valid private key for this
        demo or a connected wallet in production.
      </p>
    </footer>
  </div>
</template>

<script setup lang="ts">
import {
  type Address,
  createWalletClient,
  type Hex,
  http,
  parseEther,
  type TypedDataDefinition } from "viem";
import { reactive, ref } from "vue";

const { account } = storeToRefs(useConnectorStore());

const runtimeConfig = useRuntimeConfig();
const chain = runtimeConfig.public.chain;

const config = reactive({
  marketplaceName: "SimpleMarketplace",
  marketplaceVersion: "v0.0.1",
  marketplaceAddress: runtimeConfig.public.contracts.marketplace,
  nftContractAddress: runtimeConfig.public.contracts.nft,
  tokenId: "0", // Will be converted to BigInt
  priceInEth: "0.01", // Will be converted to wei (BigInt)
  sellerNonce: "0", // Will be converted to BigInt
});

const isLoading = ref(false);
const errorMsg = ref<string | null>(null);
const signature = ref<Hex | null>(null);
const signedDomain = ref<object | null>(null);
const signedTypes = ref<object | null>(null);
const signedMessage = ref<object | null>(null);

// Helper to convert BigInt to string for JSON.stringify
function bigIntReplacer(key: string, value) {
  if (typeof value === "bigint") {
    return value.toString();
  }
  return value;
}

const listRequestTypes = {
  ListRequest: [
    { name: "seller", type: "address" },
    { name: "nftContract", type: "address" },
    { name: "tokenId", type: "uint256" },
    { name: "price", type: "uint256" },
    { name: "nonce", type: "uint256" },
  ],
} as const;

async function handleSignListRequest() {
  isLoading.value = true;
  errorMsg.value = null;
  signature.value = null;
  signedDomain.value = null;
  signedTypes.value = null;
  signedMessage.value = null;

  if (!config.marketplaceAddress) {
    errorMsg.value = "Marketplace address is required.";
    isLoading.value = false;
    console.error("Marketplace address is required.", runtimeConfig.public.contracts.marketplace);
    return;
  }
  if (!account.value.address) {
    errorMsg.value = "Smart Account address is required.";
    isLoading.value = false;
    return;
  }
  if (!config.nftContractAddress) {
    errorMsg.value = "nft address is required.";
    isLoading.value = false;
    return;
  }
  if (!chain.id) {
    errorMsg.value = "Chain ID is required.";
    isLoading.value = false;
    return;
  }

  try {
    const accountValue = account.value;
    if (!chain) {
      errorMsg.value = `Chain with ID ${chain.id} not found in wagmiConfig.`;
      isLoading.value = false;
      return;
    }
    if (!accountValue || !accountValue.address) {
      errorMsg.value = "Smart Account address is required.";
      isLoading.value = false;
      return;
    }
    if (!config.marketplaceAddress || !config.nftContractAddress) {
      errorMsg.value = "Marketplace and NFT contract addresses are required.";
      isLoading.value = false;
      return;
    }
    const client = createWalletClient({
      account: accountValue,
      chain: chain,
      transport: http(),
    });
    const domain = {
      name: config.marketplaceName,
      version: config.marketplaceVersion,
      chainId: BigInt(chain.id),
      verifyingContract: config.marketplaceAddress as Address,
    } as const;

    const message = {
      seller: accountValue.address,
      nftContract: config.nftContractAddress as Address,
      tokenId: BigInt(config.tokenId),
      price: parseEther(config.priceInEth),
      nonce: BigInt(config.sellerNonce),
    } as const;

    // Store for display
    signedDomain.value = domain;
    signedTypes.value = listRequestTypes;
    signedMessage.value = message;

    const typedData: TypedDataDefinition<typeof listRequestTypes, "ListRequest"> = {
      domain,
      types: listRequestTypes,
      primaryType: "ListRequest",
      message,
    };

    const sig = await client.signTypedData(typedData);
    signature.value = sig;
  } catch (e) {
    errorMsg.value = e.message || "An unknown error occurred during signing.";
    if (e.shortMessage) { // Viem often has a shortMessage
      errorMsg.value = e.shortMessage;
    }
  } finally {
    isLoading.value = false;
  }
}
</script>

<style scoped>
/* Scoped styles for the component if needed */
/* Tailwind CSS is applied globally via nuxt.config.ts or a global CSS file */
pre {
  white-space: pre-wrap; /* Allow text wrapping in pre tags */
  word-break: break-all; /* Break long strings */
}
</style>
