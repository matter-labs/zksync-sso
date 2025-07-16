<template>
  <SessionTemplate>
    <template #header>
      <SessionAccountHeader
        message="Signing message from"
      />
    </template>

    <h2 class="flex items-center justify-center text-white text-center text-3xl mt-6 font-semibold">
      Personal Sign Request
    </h2>

    <div class="mt-8 space-y-4">
      <div class="text-lg flex justify-between">
        <div class="text-neutral-400">
          Message
        </div>
      </div>
      <div class="bg-neutral-800 rounded-lg p-4 border border-neutral-700">
        <div class="text-sm text-neutral-300 break-words whitespace-pre-wrap">
          {{ messageToSign }}
        </div>
      </div>
    </div>

    <div
      v-if="responseError"
      class="text-xs text-error-500 border p-2 rounded-2xl border-error-500/30 mt-4 clip"
    >
      {{ responseError }}
    </div>

    <button
      class="mx-auto mt-4 text-center w-max px-4 py-2 flex items-center gap-1 text-sm text-neutral-700 hover:text-neutral-600 transition-colors"
      @click="advancedInfoOpened = !advancedInfoOpened"
    >
      <span>{{ advancedInfoOpened ? 'Hide' : 'Show' }} message details</span>
      <ChevronDownIcon
        class="w-4 h-4 transition-transform"
        :class="{ 'rotate-180': advancedInfoOpened }"
        aria-hidden="true"
      />
    </button>
    <CommonHeightTransition :opened="advancedInfoOpened">
      <CommonLine>
        <div class="p-3 text-xs space-y-2">
          <div>
            <strong>Original message:</strong>
          </div>
          <pre class="overflow-auto bg-neutral-900 p-2 rounded">{{ originalMessage }}</pre>
          <div>
            <strong>Hex representation:</strong>
          </div>
          <pre class="overflow-auto bg-neutral-900 p-2 rounded break-all">{{ messageHex }}</pre>
        </div>
      </CommonLine>
    </CommonHeightTransition>

    <template #footer>
      <div class="flex gap-4">
        <ZkButton
          class="w-full"
          type="secondary"
          @click="deny()"
        >
          Cancel
        </ZkButton>
        <ZkButton
          class="w-full"
          :loading="!appMeta || responseInProgress"
          data-testid="confirm"
          @click="confirmSign()"
        >
          Sign
        </ZkButton>
      </div>
    </template>
  </SessionTemplate>
</template>

<script lang="ts" setup>
import { ChevronDownIcon } from "@heroicons/vue/24/outline";
import { hexToString, isHex } from "viem";
import { signMessage } from "viem/experimental/erc7739";
import type { ExtractParams } from "zksync-sso/client-auth-server";

const { appMeta } = useAppMeta();
const { respond, deny } = useRequestsStore();
const { responseInProgress, responseError, requestParams, requestChain } = storeToRefs(useRequestsStore());
const { getClient } = useClientStore();

const messageParams = computed(() => {
  const params = requestParams.value as ExtractParams<"personal_sign">;
  if (!params) return null;
  return params;
});

const originalMessage = computed(() => {
  if (!messageParams.value) return "";
  return messageParams.value[0];
});

const messageHex = computed(() => {
  return originalMessage.value;
});

const messageToSign = computed(() => {
  if (!originalMessage.value) return "";

  try {
    if (isHex(originalMessage.value)) {
      const decoded = hexToString(originalMessage.value);
      return decoded;
    }
    return originalMessage.value;
  } catch {
    return originalMessage.value;
  }
});

const advancedInfoOpened = ref(false);

const confirmSign = async () => {
  respond(async () => {
    if (!messageParams.value) {
      throw new Error("Message parameters are not available");
    }
    const client = getClient({ chainId: requestChain.value!.id });
    const signature = await signMessage(client, {
      verifier: client.account.address,
      message: messageParams.value[0],
    });
    return {
      result: signature,
    };
  });
};
</script>
