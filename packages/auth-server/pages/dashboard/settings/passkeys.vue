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
        Passkeys
      </template>
      <template #aside>
        <transition v-bind="TransitionOpacity">
          <ZkButton
            v-if="passkeys?.length"
            type="danger"
          >
            <template #prefix>
              <HandRaisedIcon
                class="h-5 w-5 mr-1"
                aria-hidden="true"
              />
            </template>
            <span class="leading-tight">Revoke all passkeys</span>
          </ZkButton>
        </transition>
      </template>
    </layout-header>

    <CommonAlert
      v-if="passkeys?.length"
      class="mb-4"
    >
      <template #icon>
        <InformationCircleIcon aria-hidden="true" />
      </template>
      <template #default>
        <p class="text-sm">
          The passkeys information here is only for the key used to open this page.
        </p>
      </template>
    </CommonAlert>

    <span
      v-if="!passkeys?.length && !passkeysInProgress"
      class="font-thin block text-2xl text-neutral-500 text-center"
    >No passkeys provided...</span>
    <div
      v-else
      class="border rounded-3xl divide-y bg-white border-neutral-200 divide-neutral-200 dark:bg-neutral-950 dark:border-neutral-900 dark:divide-neutral-900"
    >
      <template v-if="!passkeys?.length && passkeysInProgress">
        <PasskeyRowLoader
          v-for="index in 3"
          :key="index"
        />
      </template>
      <template v-else>
        <PasskeyRow
          v-for="(item, index) in (passkeys || [])"
          :key="item.passkeyId"
          :index="((passkeys?.length || 0) - index)"
          :passkey-id="item.passkeyId"
          :transaction-hash="item.transactionHash"
          :block-number="item.blockNumber"
        />
      </template>
    </div>

    <CommonAlert
      v-if="defaultChain.id === zksyncInMemoryNode.id && passkeys?.length"
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
import { WebAuthValidatorAbi } from "zksync-sso/abi";

const { defaultChain, getPublicClient } = useClientStore();
const { address } = storeToRefs(useAccountStore());

const {
  result: passkeys,
  inProgress: passkeysInProgress,
  execute: passkeysFetch,
} = useAsync(async () => {
  const contracts = contractsByChain[defaultChain.id];
  const publicClient = getPublicClient({ chainId: defaultChain.id });
  const logs = await publicClient.getContractEvents({
    abi: WebAuthValidatorAbi,
    address: contracts.passkey,
    eventName: "PasskeyCreated",
    args: {
      keyOwner: address.value,
    },
    fromBlock: 0n,
  });
  const data = logs
    .map((log) => ({
      passkeyId: log.args.credentialId,
      transactionHash: log.transactionHash,
      blockNumber: log.blockNumber,
    })).sort((a, b) => {
      if (a.blockNumber < b.blockNumber) return 1;
      if (a.blockNumber > b.blockNumber) return -1;
      return 0;
    });
  return data;
});

passkeysFetch();
</script>
