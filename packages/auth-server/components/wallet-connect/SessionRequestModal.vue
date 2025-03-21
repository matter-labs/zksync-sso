<template>
  <Dialog
    ref="modalRef"
    content-class="min-w-[700px] min-h-[500px]"
    description-class="flex-1 mb-0 flex text-base"
    close-class="h-8 max-h-8"
    :title="title"
  >
    <template #trigger>
      <div />
    </template>

    <template #submit>
      <div />
    </template>

    <template #cancel>
      <div />
    </template>
    <SignTypedDataModal
      v-if="walletConnectStore.sessionRequest?.params.request.method === 'eth_signTypedData_v4'"
      :close-modal="closeModal"
      :request="walletConnectStore.sessionRequest"
    />
    <SignPersonalMessageModal
      v-if="walletConnectStore.sessionRequest?.params.request.method === 'personal_sign'"
      :close-modal="closeModal"
      :request="walletConnectStore.sessionRequest"
    />
    <SendTransactionModal
      v-if="walletConnectStore.sessionRequest?.params.request.method === 'eth_sendTransaction'"
      :close-modal="closeModal"
      :request="walletConnectStore.sessionRequest"
    />
    <SendRawTransactionModal
      v-if="walletConnectStore.sessionRequest?.params.request.method === 'eth_sendRawTransaction'"
      :close-modal="closeModal"
      :request="walletConnectStore.sessionRequest"
    />
  </Dialog>
</template>

<script setup lang="ts">
import { ref } from "vue";

import SendRawTransactionModal from "~/components/wallet-connect/SendRawTransactionModal.vue";
import SendTransactionModal from "~/components/wallet-connect/SendTransactionModal.vue";
import SignPersonalMessageModal from "~/components/wallet-connect/SignPersonalMessageModal.vue";
import SignTypedDataModal from "~/components/wallet-connect/SignTypedDataModal.vue";
import Dialog from "~/components/zk/dialog.vue";

const modalRef = ref<InstanceType<typeof Dialog>>();
const walletConnectStore = useWalletConnectStore();

watchEffect(() => {
  if (walletConnectStore.sessionRequest) {
    modalRef.value?.open();
  }
});

function closeModal() {
  modalRef.value?.close();
}

const title = computed(() => {
  switch (walletConnectStore.sessionRequest?.params.request.method) {
    case "eth_sendTransaction":
    case "eth_sendRawTransaction":
      return "Send Transaction";
    case "eth_signTypedData_v4":
    case "personal_sign":
      return "Signature Request";
    default:
      return "Unknown Request";
  }
});
</script>
