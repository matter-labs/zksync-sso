<template>
  <div class="flex flex-col flex-1">
    <Card
      v-if="passkeys.length === 0"
      class="border-yellow-400 bg-yellow-50 dark:bg-yellow-950 dark:border-yellow-600"
    >
      <h3 class="font-semibold text-yellow-800 mb-2 dark:text-yellow-200">
        You don't have any passkey configured.
      </h3>
      <p class="text-yellow-700 mb-4 dark:text-yellow-300">
        Configure your account passkeys to ensure your account is secure.
      </p>
    </Card>
    <div v-else>
      <div class="space-y-6">
        <Card
          v-for="currentPasskey in passkeys"
          :key="currentPasskey.id"
          :loading="getPasskeysInProgress && removePasskeyInProgress"
          class="p-6"
        >
          <div class="flex justify-between lg:flex-row flex-col items-start gap-4">
            <div class="space-y-4">
              <div class="flex items-center gap-2">
                <h3 class="font-semibold text-lg">
                  {{ currentPasskey.method }}
                </h3>
              </div>
              <div class="flex items-center gap-3 text-gray-600 dark:text-gray-400">
                <WalletIcon class="w-5 h-5 flex-shrink-0" />
                <span class="font-mono text-sm">{{ currentPasskey.id }}</span>
                <CopyToClipboard
                  class="-ml-2"
                  :text="currentPasskey.id"
                />
              </div>
            </div>
            <Button
              v-if="currentPasskey.id !== usernameHexed"
              type="danger"
              class="text-sm lg:w-auto w-full"
              @click="removePasskey(origin, currentPasskey.id)"
            >
              Remove
            </Button>
          </div>
        </Card>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { WalletIcon } from "@heroicons/vue/24/solid";
import { toHex } from "viem";
import { base64UrlToUint8Array } from "zksync-sso/utils";

import CopyToClipboard from "~/components/common/CopyToClipboard.vue";
import Button from "~/components/zk/button.vue";
import Card from "~/components/zk/panel/card.vue";

const { address: accountAddress, username } = useAccountStore();
const { getPasskeysInProgress, getPasskeys, getPasskeysData, removePasskey, removePasskeyInProgress } = usePasskeyModule();

const origin = window.location.origin;
const passkeys = computed(() => (getPasskeysData.value ?? []).map((x) => ({
  method: "Passkey",
  id: x,
})));

const usernameHexed = computed(() => toHex(base64UrlToUint8Array((username!))));

watchEffect(async () => {
  if (accountAddress) {
    await getPasskeys(origin, accountAddress);
  }
});
</script>
