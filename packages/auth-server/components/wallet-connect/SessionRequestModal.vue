<template>
  <Dialog
    ref="modalRef"
    content-class="min-w-[700px] min-h-[500px]"
    description-class="flex-1 mb-0 flex text-base"
    close-class="h-8 max-h-8"
    :title="title"
    @close="onModalClosed()"
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
    <SendTransactionFlow
      v-if="sessionRequest?.params.request.method === 'eth_sendTransaction'"
      :close-modal="closeModal"
      :request="sessionRequest"
    />
    <SignTypedDataFlow
      v-if="sessionRequest?.params.request.method === 'eth_signTypedData_v4'"
      :close-modal="closeModal"
      :request="sessionRequest"
    />
  </Dialog>
</template>

<script setup lang="ts">
import { ref } from "vue";

import SendTransactionFlow from "~/components/wallet-connect/send-transaction/Root.vue";
import SignTypedDataFlow from "~/components/wallet-connect/sign-typed-data/Root.vue";
import Dialog from "~/components/zk/dialog.vue";

const modalRef = ref<InstanceType<typeof Dialog>>();
const { sessionRequest } = storeToRefs(useWalletConnectStore());

watchEffect(() => {
  if (sessionRequest.value) {
    modalRef.value?.open();
  }
});

const emit = defineEmits<{
  (e: "closed"): void;
}>();

function onModalClosed() {
  emit("closed");
}

function closeModal() {
  modalRef.value?.close();
}

const title = computed(() => {
  switch (sessionRequest.value?.params.request.method) {
    case "eth_sendTransaction":
      return "Send Transaction";
    default:
      return "";
  }
});
</script>
