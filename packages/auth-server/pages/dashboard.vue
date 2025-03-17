<template>
  <div>
    <header class="max-w-[1920px] mx-auto mb-8">
      <AppNav />
    </header>
    <main class="max-w-[900px] m-auto">
      <NuxtPage />
    </main>
    <!-- <WalletConnectSessionRequestModal /> -->
    <Teleport to="body">
      <div
        v-if="pendingRecovery"
        class="fixed bottom-6 left-1/2 -translate-x-1/2 rounded-2xl flex gap-4 backdrop-blur-sm p-6 border bg-orange-100 dark:bg-orange-900/30 border-orange-200 dark:border-orange-700/50 w-[calc(100vw-32px)] md:max-w-lg"
      >
        <ExclamationTriangleIcon class="w-6 h-6 text-yellow-600 dark:text-yellow-400 flex-shrink-0" />
        <div class="flex flex-col flex-1">
          <h3 class="text-lg font-semibold mb-2 text-yellow-800 dark:text-yellow-200">
            Recovery Process Initiated
          </h3>
          <p class="text-yellow-700 dark:text-yellow-300">
            A recovery process has been initiated for your account. Was this done by you?
          </p>
          <div class="flex justify-end gap-3 mt-4">
            <ZkButton
              type="danger"
              @click="cancelRecovery"
            >
              Cancel Recovery
            </ZkButton>
            <ZkButton
              type="secondary"
              @click="dismissWarning"
            >
              Dismiss
            </ZkButton>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ExclamationTriangleIcon } from "@heroicons/vue/24/solid";

const accountStore = useAccountStore();
const walletConnectStore = useWalletConnectStore();
const { checkRecoveryRequest, discardRecovery } = useRecoveryGuardian();

const pendingRecovery = ref(false);

onMounted(async () => {
  await walletConnectStore.initialize();
});

watchEffect(async () => {
  if (!accountStore.address) return;
  const recoveryRequest = await checkRecoveryRequest({ address: accountStore.address });
  pendingRecovery.value = recoveryRequest?.pendingRecovery ?? false;
});

const cancelRecovery = async () => {
  await discardRecovery();
  pendingRecovery.value = false;
};

const dismissWarning = () => {
  pendingRecovery.value = false;
};

definePageMeta({
  middleware: ["logged-in"],
  layout: "dashboard",
});
</script>
