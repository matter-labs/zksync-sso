<template>
  <div class="flex flex-col flex-1">
    <div class="space-y-6">
      <Card class="border-yellow-400 bg-yellow-50 dark:bg-yellow-950 dark:border-yellow-600">
        <h3 class="font-semibold text-yellow-800 mb-2 dark:text-yellow-200">
          Pair WalletConnect Account
        </h3>
        <p class="text-yellow-700 mb-4 dark:text-yellow-300">
          Pair your WalletConnect dApp with your account to start using it.
        </p>
        <ZkInput
          v-model="pairingUrl"
          placeholder="0x..."
          class="w-full text-left"
        />
        <Button
          class="bg-yellow-500 hover:bg-yellow-600 text-white dark:bg-yellow-600 dark:hover:bg-yellow-700 focus:bg-yellow-600 active:bg-yellow-700 disabled:bg-yellow-500 disabled:text-yellow-300 disabled:dark:bg-yellow-600 disabled:dark:hover:bg-yellow-700 dark:focus:bg-yellow-700 dark:active:bg-yellow-800 focus:ring-yellow-400 dark:focus:ring-yellow-800"
          @click="pairWcAccount"
        >
          Pair
        </Button>
      </Card>
      <Card
        v-for="session in openSessions"
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
              Expires on {{ session.expiry }}
            </p>
          </div>
          <Button
            type="danger"
            class="text-sm lg:w-auto w-full"
            @click="closeSession(session.topic)"
          >
            Close
          </Button>
        </div>
      </Card>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { ModelRef } from "vue";

import Button from "~/components/zk/button.vue";
import Card from "~/components/zk/panel/card.vue";

const pairingUrl = defineModel<string>();
const { closeSession } = useWalletConnectStore();
const { openSessions } = storeToRefs(useWalletConnectStore());
const { pairAccount } = useWalletConnectStore();
const pairWcAccount = () => {
  if (pairingUrl.value) {
    pairAccount(toRef(pairingUrl as ModelRef<string>));
  }
};
</script>
