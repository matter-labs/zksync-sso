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
    <layout-header>
      <template #default>
        Private Keys
      </template>
      <template #aside>
        <transition v-bind="TransitionOpacity">
          <ZkButton
            v-if="ownerKeys?.length"
            type="danger"
          >
            <template #prefix>
              <HandRaisedIcon
                class="h-5 w-5 mr-1"
                aria-hidden="true"
              />
            </template>
            <span class="leading-tight">Remove all private keys</span>
          </ZkButton>
        </transition>
      </template>
    </layout-header>

    <CommonAlert
      v-if="ownerKeys?.length"
      class="mb-4"
    >
      <template #icon>
        <InformationCircleIcon aria-hidden="true" />
      </template>
      <template #default>
        <p class="text-sm">
          Private key information is specific to this browser session. If you open this page in another browser or incognito mode, you will not see the same private keys.
        </p>
      </template>
    </CommonAlert>

    <div class="mb-6">
      <h3 class="text-lg font-medium leading-6 text-gray-900 dark:text-gray-100">
        Add a New Private Key
      </h3>
      <form @submit.prevent="addPrivateKey">
        <div class="mt-4">
          <label
            for="privateKey"
            class="block text-sm font-medium text-gray-700 dark:text-gray-300"
          >
            Private Key
          </label>
          <input
            id="privateKey"
            v-model="newPrivateKey"
            type="text"
            class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 dark:bg-neutral-800 dark:border-neutral-700 dark:text-gray-300"
            placeholder="Enter your private key"
            required
          >
        </div>
        <div class="mt-4">
          <ZkButton
            type="primary"
            class="w-full"
          >
            Add Private Key
          </ZkButton>
        </div>
      </form>
    </div>
    <span
      v-if="!ownerKeys?.length && !ownerKeysInProgress"
      class="font-thin block text-2xl text-neutral-500 text-center"
    >
      No private keys provided...
    </span>
    <div
      v-else
      class="border rounded-3xl divide-y bg-white border-neutral-200 divide-neutral-200 dark:bg-neutral-950 dark:border-neutral-900 dark:divide-neutral-900"
    >
      <template v-if="!ownerKeys?.length && ownerKeysInProgress">
        <PasskeyRowLoader
          v-for="index in 3"
          :key="index"
        />
      </template>
      <template v-else>
        <OwnerKeyRow
          v-for="(item, index) in (ownerKeys || [])"
          :key="item"
          :index="((ownerKeys?.length || 0) - index)"
        />
      </template>
    </div>

    <CommonAlert
      v-if="defaultChain.id === zksyncInMemoryNode.id && ownerKeys?.length"
      class="mt-4"
    >
      <template #icon>
        <InformationCircleIcon aria-hidden="true" />
      </template>
      <template #default>
        <p class="text-sm">
          Timestamps on {{ zksyncInMemoryNode.name }} start from 0 and incremented by 1 with each block. Therefore session time isn't accurate.
        </p>
      </template>
    </CommonAlert>
  </div>
</template>

<script setup lang="ts">
import { InformationCircleIcon } from "@heroicons/vue/20/solid";
import { zksyncInMemoryNode } from "viem/chains";
import { SsoAccountAbi } from "zksync-sso/abi";

const { defaultChain, getPublicClient, getEcdsaClient } = useClientStore();
const { address } = storeToRefs(useAccountStore());

const {
  result: ownerKeys,
  inProgress: ownerKeysInProgress,
  execute: ownerKeysFetch,
} = useAsync(async () => {
  if (!address.value) return [];
  const publicClient = getPublicClient({ chainId: defaultChain.id });
  const k1OwnerList = await publicClient.readContract({
    address: address.value,
    abi: SsoAccountAbi,
    functionName: "listK1Owners",
    args: [],
  });
  return k1OwnerList;
});

ownerKeysFetch();

const newPrivateKey = ref("");

const addPrivateKey = async () => {
  if (!newPrivateKey.value) {
    return;
  }

  try {
    const client = getEcdsaClient({ chainId: defaultChain.id });
    const signer = await client.getSigner();
    const contract = new client.Contract(address.value, SsoAccountAbi, signer);

    await contract.write.addK1Owner([newPrivateKey]);

    newPrivateKey.value = ""; // Clear the input field
    await ownerKeysFetch(); // Refresh the owner keys list
  } catch (error) {
    console.error("Failed to add private key:", error);
  }
};
</script>
