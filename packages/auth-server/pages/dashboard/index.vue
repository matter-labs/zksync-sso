<template>
  <div>
    <layout-header>
      <span class="font-thin">Welcome to ZKsync SSO</span>
      <template #aside>
        <div class="flex flex-col items-end">
          <!-- Primary: Account address with avatar -->
          <div class="flex items-center space-x-3 group">
            <Web3Avatar
              :address="address!"
              class="size-10"
            />
            <div class="flex flex-col items-end">
              <ZkTooltip :label="copyLabel">
                <button
                  data-testid="account-address"
                  :data-address="address"
                  class="text-lg leading-tight font-medium text-neutral-900 dark:text-neutral-100 cursor-pointer border-b border-dashed border-neutral-300 dark:border-neutral-600 hover:border-neutral-500 dark:hover:border-neutral-400 hover:scale-105 transition-all duration-200"
                  @click="copyContent"
                >
                  {{ shortenAddress(address!) }}
                </button>
              </ZkTooltip>
              <!-- Secondary: Authentication info -->
              <div
                v-if="isPrividiumMode && isAuthenticated"
                class="flex items-center space-x-1 mt-1"
              >
                <div class="w-1 h-1 bg-green-500 rounded-full" />
                <span class="text-xs text-neutral-500 dark:text-neutral-400">
                  {{ profile?.displayName || profile?.userId || "Unknown" }}
                </span>
              </div>
            </div>
          </div>
        </div>
      </template>
    </layout-header>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <dashboard-assets />

      <!-- <dashboard-nfts /> -->
    </div>
  </div>
</template>

<script setup lang="ts">
import Web3Avatar from "web3-avatar-vue";

const { address } = storeToRefs(useAccountStore());
const { $config } = useNuxtApp();
const { isAuthenticated, profile } = storeToRefs(usePrividiumAuthStore());

const isPrividiumMode = computed(() => $config.public.prividiumMode);

const { copy } = useClipboard({
  source: computed(() => address.value || ""),
});

const copyLabel = ref("Copy");

function copyContent() {
  copy(address.value!);
  copyLabel.value = "Copied!";
  setTimeout(() => {
    copyLabel.value = "Copy";
  }, 2500);
}
</script>
