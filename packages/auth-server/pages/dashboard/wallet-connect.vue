<template>
  <div class="flex flex-col flex-1">
    <div class="space-y-6">
      <Card class="bg-gray-100">
        <h3 class="font-semibold mb-2">
          Connect with WalletConnect
        </h3>
        <p class="mb-4">
          Enter the WalletConnect URI from the application you want to connect to below.
        </p>
        <ZkInput
          v-model="pairingUri"
          placeholder="e.g. wc:a281567bb3e4..."
          class="w-full text-left mb-4"
        />
        <Button @click="pairAccount">
          Connect
        </Button>
      </Card>
      <Card
        v-for="session in walletConnectStore.openSessions"
        :key="session.topic"
        class="p-6"
      >
        <div class="flex justify-between lg:flex-row flex-col items-start gap-4">
          <div class="space-y-4">
            <div class="flex items-center gap-2">
              <h3 class="font-semibold text-lg">
                {{ session.peer.metadata.name }}
              </h3>
            </div>
            <div class="flex items-center gap-3 text-gray-600 dark:text-gray-400">
              <WalletIcon class="w-5 h-5 flex-shrink-0" />
              <span class="font-mono text-sm">{{ session.peer.metadata.url }}</span>
            </div>
            <p class="text-sm text-gray-500 dark:text-gray-500">
              Expires on {{ fromUnixTime(session.expiry).toDateString() }} - {{ fromUnixTime(session.expiry).toLocaleTimeString() }}
            </p>
          </div>
          <Button
            type="danger"
            class="text-sm lg:w-auto w-full"
            @click="walletConnectStore.closeSession(session.topic)"
          >
            Delete
          </Button>
        </div>
      </Card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { WalletIcon } from "@heroicons/vue/24/solid";
import { fromUnixTime } from "date-fns";

import Button from "~/components/zk/button.vue";
import Card from "~/components/zk/panel/card.vue";

const pairingUri = defineModel<string>();
const walletConnectStore = useWalletConnectStore();

const pairAccount = () => {
  if (!pairingUri.value) return;
  walletConnectStore.pairAccount(pairingUri.value);
};
</script>
