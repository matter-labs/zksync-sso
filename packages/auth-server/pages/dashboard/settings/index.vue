<template>
  <div class="flex flex-col flex-1">
    <Card
      v-if="recoveryMethods.length === 0"
      class="border-yellow-400 bg-yellow-50 dark:bg-yellow-950 dark:border-yellow-600"
    >
      <h3 class="font-semibold text-yellow-800 mb-2 dark:text-yellow-200">
        You don't have any account recovery methods configured.
      </h3>
      <p class="text-yellow-700 mb-4 dark:text-yellow-300">
        Configure your account recovery methods to ensure your account is secure.
      </p>
      <AddRecoveryMethodModal>
        <template #trigger>
          <Button
            class="bg-yellow-500 hover:bg-yellow-600 text-white dark:bg-yellow-600 dark:hover:bg-yellow-700 focus:bg-yellow-600 active:bg-yellow-700 disabled:bg-yellow-500 disabled:text-yellow-300 disabled:dark:bg-yellow-600 disabled:dark:hover:bg-yellow-700 dark:focus:bg-yellow-700 dark:active:bg-yellow-800 focus:ring-yellow-400 dark:focus:ring-yellow-800"
          >
            Add Recovery Method
          </Button>
        </template>
      </AddRecoveryMethodModal>
    </Card>
    <div v-else>
      <div class="space-y-6">
        <Card
          v-for="method in recoveryMethods"
          :key="method.address"
          class="p-6"
          :class="{ 'border-yellow-400 bg-yellow-50 dark:bg-yellow-950 dark:border-yellow-600': method.pendingUrl }"
        >
          <div class="flex justify-between lg:flex-row flex-col items-start gap-4">
            <div class="space-y-4">
              <div class="flex items-center gap-2">
                <h3 class="font-semibold text-lg">
                  {{ method.method }}
                </h3>
                <span
                  v-if="method.pendingUrl"
                  class="text-yellow-600 dark:text-yellow-400 text-sm font-medium"
                >
                  (Pending)
                </span>
              </div>
              <div class="flex items-center gap-3 text-gray-600 dark:text-gray-400">
                <WalletIcon class="w-5 h-5 flex-shrink-0" />
                <span class="font-mono text-sm">{{ isMobile ? shortenAddress(method.address) : method.address }}</span>
                <CopyToClipboard
                  class="-ml-2"
                  :text="method.address"
                />
              </div>
              <p class="text-sm text-gray-500 dark:text-gray-500">
                Added on {{ method.addedOn.toLocaleDateString() }} {{ method.addedOn.toLocaleTimeString() }}
              </p>
              <div
                v-if="method.pendingUrl"
                class="text-sm text-yellow-700 dark:text-yellow-300"
              >
                <p class="mb-2">
                  This recovery method needs to be confirmed.
                </p>
                <div class="space-x-1 flex items-center max-w-md">
                  <a
                    :href="method.pendingUrl"
                    target="_blank"
                    class="text-yellow-600 dark:text-yellow-400 hover:underline truncate"
                  >
                    {{ method.pendingUrl }}
                  </a>
                  <CopyToClipboard
                    class="text-yellow-600 dark:text-yellow-400 hover:text-yellow-700"
                    :text="method.pendingUrl"
                  />
                </div>
              </div>
            </div>
            <Button
              type="danger"
              class="text-sm lg:w-auto w-full"
              @click="removeRecoveryMethod(method.address)"
            >
              Remove
            </Button>
          </div>
        </Card>
        <AddRecoveryMethodModal />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { WalletIcon } from "@heroicons/vue/24/solid";
import { breakpointsTailwind, useBreakpoints } from "@vueuse/core";

import AddRecoveryMethodModal from "~/components/account-recovery/AddRecoveryMethodModal.vue";
import CopyToClipboard from "~/components/common/CopyToClipboard.vue";
import Button from "~/components/zk/button.vue";
import Card from "~/components/zk/panel/card.vue";
import { shortenAddress } from "~/utils/formatters";

const breakpoints = useBreakpoints(breakpointsTailwind);
const isMobile = breakpoints.smaller("lg");
const recoveryMethods = ref([] as { method: string; address: string; addedOn: Date; pendingUrl?: string }[]);
const config = useRuntimeConfig();

recoveryMethods.value = [
  { method: "Guardian", address: "0x72D8dd6EE7ce73D545B229127E72c8AA013F4a9e", addedOn: new Date() },
  { method: "Guardian", address: "0x72D8dd6EE7ce73D545B229127E72c8AA013F4a9e", addedOn: new Date(), pendingUrl: `${config.public.appUrl}/recovery/guardian/confirm-guardian?accountAddress=0x72D8dd6EE7ce73D545B229127E72c8AA013F4a9e&guardianAddress=0x72D8dd6EE7ce73D545B229127E72c8AA013F4a9e` },
];

const removeRecoveryMethod = (address: string) => {
  // TODO: Implement removal logic
  recoveryMethods.value = recoveryMethods.value.filter((m) => m.address !== address);
};
</script>
