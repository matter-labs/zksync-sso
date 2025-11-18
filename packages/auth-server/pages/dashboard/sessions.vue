<template>
  <div>
    <layout-header>
      <template #default>
        Sessions
      </template>
      <template #aside>
        <!-- <transition v-bind="TransitionOpacity">
          <ZkButton
            v-if="sessions?.length"
            type="danger"
          >
            <template #prefix>
              <HandRaisedIcon
                class="h-5 w-5 mr-1"
                aria-hidden="true"
              />
            </template>
            <span class="leading-tight">End all sessions</span>
          </ZkButton>
        </transition> -->
      </template>
    </layout-header>

    <CommonAlert
      v-if="sessions?.length"
      class="mb-4"
    >
      <template #icon>
        <InformationCircleIcon aria-hidden="true" />
      </template>
      <template #default>
        <p class="text-sm">
          ZKsync SSO is still under development. The displayed spending amounts may not always be accurate.
        </p>
      </template>
    </CommonAlert>

    <span
      v-if="!sessions?.length && !sessionsInProgress"
      class="font-thin block text-2xl text-neutral-500 text-center"
    >No sessions yet...</span>
    <div
      v-else
      class="border rounded-3xl divide-y bg-white border-neutral-200 divide-neutral-200 dark:bg-neutral-950 dark:border-neutral-900 dark:divide-neutral-900"
    >
      <template v-if="!sessions?.length && sessionsInProgress">
        <SessionRowLoader
          v-for="index in 3"
          :key="index"
        />
      </template>
      <template v-else>
        <SessionRow
          v-for="(item, index) in (sessions || [])"
          :key="item.sessionId"
          :index="((sessions?.length || 0) - index)"
          :session-id="item.sessionId"
          :session="item.session"
          :transaction-hash="item.transactionHash"
          :block-number="item.blockNumber"
          :timestamp="item.timestamp"
        />
      </template>
    </div>

    <CommonAlert
      v-if="defaultChain.id === zksyncInMemoryNode.id && sessions?.length"
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
import type { Hex } from "viem";
import { zksyncInMemoryNode } from "viem/chains";
import { SessionKeyValidatorAbi } from "zksync-sso-4337/abi";
import type { SessionConfig } from "zksync-sso-4337/client";

const { defaultChain, getPublicClient } = useClientStore();
const { address } = storeToRefs(useAccountStore());

const {
  result: sessions,
  inProgress: sessionsInProgress,
  // error: sessionsFetchError,
  execute: sessionsFetch,
} = useAsync(async () => {
  const contracts = contractsByChain[defaultChain.id];
  const publicClient = getPublicClient({ chainId: defaultChain.id });
  const logs = await publicClient.getContractEvents({
    abi: SessionKeyValidatorAbi,
    address: contracts.session,
    eventName: "SessionCreated",
    args: {
      account: address.value,
    },
    fromBlock: 0n,
  });
  const data = logs
    .filter((log) => log.args.sessionSpec && log.args.sessionHash)
    .map((log) => ({
      session: log.args.sessionSpec! as SessionConfig,
      sessionId: log.args.sessionHash!,
      transactionHash: log.transactionHash,
      blockNumber: log.blockNumber,
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      timestamp: new Date(parseInt((log as any).blockTimestamp as Hex, 16) * 1000).getTime(),
    })).sort((a, b) => {
      if (a.blockNumber < b.blockNumber) return 1;
      if (a.blockNumber > b.blockNumber) return -1;
      return 0;
    });
  return data;
});

sessionsFetch();
</script>
