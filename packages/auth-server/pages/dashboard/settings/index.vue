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
      <AddRecoveryMethodModal
        @closed="refreshGuardians"
      >
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
          v-for="method in guardianMethods"
          :key="method.address"
          :loading="getGuardiansInProgress && removeGuardianInProgress"
          class="p-6"
          :class="{ 'border-yellow-400 bg-yellow-50 dark:bg-yellow-950 dark:border-yellow-600': method.pendingUrl }"
        >
          <div class="flex justify-between lg:flex-row flex-col items-start gap-4">
            <div class="space-y-4">
              <div class="flex items-center gap-2">
                <h3 class="font-semibold text-lg">
                  Guardian
                </h3>
                <span
                  v-if="method.pendingUrl"
                  class="text-yellow-600 dark:text-yellow-400 text-sm font-medium"
                >
                  (Pending)
                </span>
              </div>
              <div
                class="flex items-center gap-3 text-gray-600 dark:text-gray-400"
              >
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
              v-if="method.method === 'Guardian'"
              type="danger"
              class="text-sm lg:w-auto w-full"
              @click="removeGuardian(method.address)"
            >
              Remove
            </Button>
          </div>
        </Card>

        <Card
          v-for="method in activeOidc"
          :key="method.digest"
          :loading="getOidcAccountsInProgress"
          class="p-6"
        >
          <div class="flex justify-between lg:flex-row flex-col items-start gap-4">
            <div class="space-y-4">
              <div class="flex items-center gap-2">
                <h3 class="font-semibold text-lg">
                  Google Recovery
                </h3>
              </div>
              <div
                class="flex flex-col items-start gap-3 text-gray-600 dark:text-gray-400"
              >
                <div class="flex items-center gap-3">
                  <ShieldCheckIcon class="w-5 h-5 flex-shrink-0" />
                  <span class="font-mono text-sm">{method.iss}</span>
                </div>
                <div class="flex items-center gap-3">
                  <SparklesIcon class="w-5 h-5 flex-shrink-0" />
                  <span class="font-mono text-sm">{{ shortenAddress(method.digest, 6) }}</span>
                </div>
              </div>
              <p class="text-sm text-gray-500 dark:text-gray-500">
                Added on {{ method.addedOn.toLocaleDateString() }} {{ method.addedOn.toLocaleTimeString() }}
              </p>
            </div>
            <Button
              type="danger"
              class="text-sm lg:w-auto w-full"
              @click="removeOidc"
            >
              Remove
            </Button>
          </div>
        </Card>
        <AddRecoveryMethodModal
          @closed="refreshGuardians"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ShieldCheckIcon, SparklesIcon, WalletIcon } from "@heroicons/vue/24/solid";
import { breakpointsTailwind, useBreakpoints } from "@vueuse/core";

import AddRecoveryMethodModal from "~/components/account-recovery/AddRecoveryMethodModal.vue";
import CopyToClipboard from "~/components/common/CopyToClipboard.vue";
import Button from "~/components/zk/button.vue";
import Card from "~/components/zk/panel/card.vue";
import { shortenAddress } from "~/utils/formatters";

const breakpoints = useBreakpoints(breakpointsTailwind);
const isMobile = breakpoints.smaller("lg");
const { address: accountAddress } = useAccountStore();
const {
  getGuardiansInProgress,
  getGuardians,
  getGuardiansData,
  removeGuardian,
  removeGuardianInProgress,
} = useRecoveryGuardian();
const {
  getOidcAccounts,
  oidcAccounts,
  getOidcAccountsInProgress,
  removeOidcAccount,
} = useRecoveryOidc();

const config = useRuntimeConfig();

const appUrl = config.public.appUrl;

const activeOidc = computed(() => oidcAccounts.value.map((oidcData) => {
  return {
    method: "OIDC",
    iss: oidcData.iss,
    digest: oidcData.oidcDigest,
    addedOn: new Date(),
  };
}));

const guardianMethods = computed(() => (getGuardiansData.value ?? []).map((x) => ({
  method: "Guardian",
  address: x.addr,
  addedOn: new Date(),
  ...(!x.isReady && { pendingUrl: `${appUrl}/recovery/guardian/confirm-guardian?accountAddress=${accountAddress}&guardianAddress=${x.addr}` }),
})));

const recoveryMethods = computed(() => [
  ...guardianMethods.value,
  ...activeOidc.value,
]);

const refreshGuardians = () => {
  if (accountAddress) {
    getGuardians(accountAddress);
    getOidcAccounts(accountAddress);
  }
};

async function removeOidc() {
  await removeOidcAccount();
  refreshGuardians();
}

watchEffect(async () => {
  if (accountAddress) {
    await getGuardians(accountAddress);
    await getOidcAccounts(accountAddress);
  }
});
</script>
