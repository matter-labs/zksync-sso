<template>
  <Dialog
    ref="modalRef"
    title="Connect Wallet"
    content-class="min-w-[500px] min-h-[500px]"
    @close="onModalClosed()"
  >
    <template #header>
      <div />
    </template>

    <template #trigger>
      <div />
    </template>

    <div class="flex flex-col gap-2 p-6">
      <img
        v-if="walletConnectStore.sessionProposal?.params.proposer.metadata.icons?.[0]"
        :src="walletConnectStore.sessionProposal.params.proposer.metadata.icons[0]"
        :alt="walletConnectStore.sessionProposal.params.proposer.metadata.name"
        class="w-16 h-16 rounded-full mx-auto"
      >

      <div class="flex flex-col gap-1">
        <h2 class="text-2xl font-semibold text-center">
          {{ walletConnectStore.sessionProposal?.params.proposer.metadata.name }}
        </h2>
        <p class="text-gray-600 text-center">
          wants to connect
        </p>
        <a
          :href="walletConnectStore.sessionProposal?.params.proposer.metadata.url"
          target="_blank"
          rel="noopener noreferrer"
          class="text-blue-600 hover:underline text-center"
        >
          {{ walletConnectStore.sessionProposal?.params.proposer.metadata.url }}
        </a>
      </div>

      <div class="flex items-center justify-center gap-1" />

      <hr class="my-2 border-gray-200">

      <h3 class="text-base font-medium text-left">
        Requested permissions
      </h3>

      <div class="flex flex-col gap-3">
        <div class="flex items-center gap-3">
          <Icon icon="alternate_email" />
          <span class="text-sm text-left">Account address
            for {{ resolveChainNames(walletConnectStore.sessionProposal?.params.optionalNamespaces?.eip155?.chains) || 'supported chains' }}
          </span>
        </div>

        <div class="flex items-center gap-3">
          <Icon icon="account_balance_wallet" />
          <span class="text-sm text-left">View your balances and activity</span>
        </div>

        <div class="flex items-center gap-3">
          <Icon icon="lock" />
          <span class="text-sm text-left">Request approval for transactions</span>
        </div>
      </div>
    </div>

    <template #submit>
      <div class="flex flex-col-reverse md:flex-row gap-3">
        <ZkButton
          type="secondary"
          class="w-full"
          @click="walletConnectStore.rejectSessionProposal"
        >
          Reject
        </ZkButton>
        <ZkButton
          type="primary"
          class="w-full"
          @click="walletConnectStore.approveSessionProposal"
        >
          Approve
        </ZkButton>
      </div>
    </template>

    <template #cancel>
      <div />
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, watchEffect } from "vue";

import Dialog from "~/components/zk/dialog.vue";
import Icon from "~/components/zk/icon.vue";
import { useWalletConnectStore } from "~/stores/wallet-connect";

const modalRef = ref<InstanceType<typeof Dialog>>();
const walletConnectStore = useWalletConnectStore();

const chainIdToName: Record<number, string> = {
  324: "zkSync Era",
  300: "zkSync Era Sepolia",
};

function resolveChainNames(chains?: string[]): string {
  if (!chains) return "";
  return chains
    .map((chain) => {
      const chainId = parseInt(chain.split(":")[1]);
      return chainIdToName[chainId];
    })
    .filter(Boolean)
    .join(", ");
}

watchEffect(() => {
  if (walletConnectStore.sessionProposal !== null) {
    modalRef.value?.open();
  } else {
    modalRef.value?.close();
  }
});

async function onModalClosed() {
  await walletConnectStore.rejectSessionProposal();
}
</script>
