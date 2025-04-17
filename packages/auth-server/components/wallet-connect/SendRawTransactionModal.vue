<template>
  <div class="flex flex-col gap-4 flex-1 items-center">
    <div class="flex items-center gap-2">
      <img
        :src="session.peer.metadata.icons[0]"
        class="w-6 h-6 rounded-full"
      >
      <p class="font-medium">
        {{ session.peer.metadata.name }}
      </p>
    </div>
    <p class="text-sm text-gray-600">
      Requests to send the following raw transaction:
    </p>
    <pre class="text-xs text-left bg-gray-50 p-3 rounded-lg whitespace-pre-wrap break-all">{{ rawTransaction }}</pre>
    <div class="flex flex-col-reverse md:flex-row gap-3 mt-2">
      <ZkButton
        type="secondary"
        class="w-full"
        @click="handleReject"
      >
        Reject
      </ZkButton>
      <ZkButton
        type="primary"
        class="w-full"
        @click="handleSend"
      >
        Send
      </ZkButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { WalletKitTypes } from "@reown/walletkit";

const walletConnectStore = useWalletConnectStore();

const props = defineProps<{
  request: WalletKitTypes.SessionRequest;
  closeModal: () => void;
}>();

const session = computed(() => {
  return walletConnectStore.openSessions[props.request.topic];
});

const rawTransaction = computed(() => {
  return props.request.params.request.params[0];
});

const handleSend = async () => {
  await walletConnectStore.sendRawTransaction(props.request);
  props.closeModal();
};

const handleReject = async () => {
  await walletConnectStore.walletKit?.respondSessionRequest({
    topic: props.request.topic,
    response: {
      id: props.request.id,
      error: {
        code: 4001,
        message: "User rejected the request",
      },
      jsonrpc: "2.0",
    },
  });
  props.closeModal();
};
</script>
