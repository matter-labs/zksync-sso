<template>
  <Button
    :type="type"
    :loading="stateData.open && !accountData.isConnected"
    @click="onClick"
  >
    <div class="flex items-center gap-2">
      <UserCircleIcon
        v-if="accountData.isConnected"
        class="w-4 h-4"
      />
      <WalletIcon
        v-else
        class="w-4 h-4"
      />
      <span>{{ text }}</span>
    </div>
  </Button>
</template>

<script setup lang="ts">
import { UserCircleIcon, WalletIcon } from "@heroicons/vue/24/solid";
import { useAppKit, useAppKitAccount, useAppKitState } from "@reown/appkit/vue";

import Button, { type ButtonTypes } from "~/components/zk/button.vue";

const { open } = useAppKit();
const accountData = useAppKitAccount();
const stateData = useAppKitState();

const onClick = async () => {
  if (accountData.value.isConnected) {
    await open({ view: "Account" });
  } else {
    await open({ view: "Connect" });
  }
};

const text = computed(() => {
  if (accountData.value.isConnected && accountData.value.address) {
    return shortenAddress(accountData.value.address);
  }
  return "Connect Wallet";
});

defineProps<{
  type?: ButtonTypes;
}>();
</script>
