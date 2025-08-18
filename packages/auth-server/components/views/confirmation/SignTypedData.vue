<template>
  <SessionTemplate>
    <template #header>
      <SessionAccountHeader
        message="Signing typed data from"
      />
    </template>

    <h2 class="flex items-center justify-center text-white text-center text-3xl mt-6 font-semibold">
      Sign Typed Data Request
    </h2>

    <div class="mt-8 space-y-4">
      <div class="text-lg flex justify-between">
        <div class="text-neutral-400">
          Domain
        </div>
      </div>
      <div class="bg-neutral-800 rounded-lg p-4 border border-neutral-700">
        <div class="space-y-2 text-sm">
          <div v-if="typedData?.domain?.name">
            <span class="text-neutral-400">Name:</span>
            <span class="text-white">{{ typedData.domain.name }}</span>
          </div>
          <div v-if="typedData?.domain?.version">
            <span class="text-neutral-400">Version:</span>
            <span class="text-white">{{ typedData.domain.version }}</span>
          </div>
          <div v-if="typedData?.domain?.chainId">
            <span class="text-neutral-400">Chain ID:</span>
            <span class="text-white">{{ typedData.domain.chainId }}</span>
          </div>
          <div v-if="typedData?.domain?.verifyingContract">
            <span class="text-neutral-400">Contract:</span>
            <span class="text-white font-mono text-xs break-all">{{ typedData.domain.verifyingContract }}</span>
          </div>
        </div>
      </div>

      <div class="text-lg flex justify-between">
        <div class="text-neutral-400">
          Message ({{ typedData?.primaryType }})
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
      <span>{{ advancedInfoOpened ? 'Hide' : 'Show' }} typed data details</span>
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
            <strong>Original parameters:</strong>
          </div>
          <pre class="overflow-auto bg-neutral-900 p-2 rounded">{{ originalParams }}</pre>
          <div>
            <strong>Parsed typed data:</strong>
          </div>
          <pre class="overflow-auto bg-neutral-900 p-2 rounded break-all">{{ JSON.stringify(typedData, null, 2) }}</pre>
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
// import { signTypedData } from "viem/experimental/erc7739";
import type { ExtractParams } from "zksync-sso/client-auth-server";

const { appMeta } = useAppMeta();
const { respond, deny } = useRequestsStore();
const { responseInProgress, responseError, requestParams, requestChain } = storeToRefs(useRequestsStore());
const { getClient } = useClientStore();

const typedDataParams = computed(() => {
  const params = requestParams.value as ExtractParams<"eth_signTypedData_v4">;
  if (!params) return null;
  return params;
});

const originalParams = computed(() => {
  if (!typedDataParams.value) return "";
  return typedDataParams.value[1];
});

const typedData = computed(() => {
  if (!typedDataParams.value) return null;
  try {
    const data = JSON.parse(typedDataParams.value[1]);
    return data;
  } catch {
    return null;
  }
});

const messageToSign = computed(() => {
  if (!typedData.value?.message) return "";
  return JSON.stringify(typedData.value.message, null, 2);
});

const advancedInfoOpened = ref(false);

const confirmSign = async () => {
  respond(async () => {
    if (!typedDataParams.value || !typedData.value) {
      throw new Error("Typed data parameters are not available");
    }
    const client = getClient({ chainId: requestChain.value!.id });
    const signature = await client.signTypedData(typedData.value);
    return {
      result: signature,
    };
  });
};
</script>
