<template>
  <div class="bg-indigo-50 p-4 rounded-lg mb-4 border border-indigo-200">
    <h2 class="text-xl font-semibold mb-3">
      Session Configuration
    </h2>

    <label class="flex items-center mb-2">
      <input
        v-model="config!.deployWithSession"
        type="checkbox"
        class="mr-2"
        :disabled="isDeployed"
      >
      <span>
        Deploy with Session Support
        <span
          v-if="isDeployed"
          class="text-xs text-gray-500 ml-1"
        >(Set before deployment)</span>
      </span>
    </label>
    <p
      v-if="config?.deployWithSession"
      class="text-xs text-gray-600 mb-3 ml-6"
    >
      Session validator will be pre-installed during account deployment
    </p>

    <label class="flex items-center mb-2">
      <input
        v-model="config!.enabled"
        type="checkbox"
        class="mr-2"
      >
      Enable Session Configuration
    </label>

    <div
      v-if="config?.enabled"
      class="space-y-2"
    >
      <div>
        <label class="block text-sm font-medium mb-1">Session Validator Address</label>
        <input
          v-model="config!.validatorAddress"
          type="text"
          placeholder="0x..."
          class="w-full p-2 border rounded"
          readonly
        >
        <p class="text-xs text-gray-500 mt-1">
          Loaded from contracts.json
        </p>
      </div>

      <div>
        <label class="block text-sm font-medium mb-1">Session Signer (auto-generated)</label>
        <input
          :value="config?.sessionSigner"
          type="text"
          readonly
          class="w-full p-2 border rounded bg-gray-100"
        >
        <p class="text-xs text-gray-500 mt-1">
          This address will be authorized to send transactions on behalf of the smart account
        </p>
      </div>

      <div>
        <label class="block text-sm font-medium mb-1">Expires At (timestamp)</label>
        <input
          v-model.number="config!.expiresAt"
          type="number"
          class="w-full p-2 border rounded"
        >
        <p class="text-xs text-gray-500 mt-1">
          Unix timestamp (default: {{ formatTimestamp(config!.expiresAt) }})
        </p>
      </div>

      <div>
        <label class="block text-sm font-medium mb-1">Max Fee (wei)</label>
        <input
          v-model="config!.feeLimit"
          type="text"
          class="w-full p-2 border rounded"
        >
        <p class="text-xs text-gray-500 mt-1">
          Maximum fee the session can spend (1 ETH = 1000000000000000000 wei)
        </p>
      </div>

      <div>
        <label class="block text-sm font-medium mb-1">Allowed Recipient (address)</label>
        <input
          v-model="config!.allowedRecipient"
          type="text"
          placeholder="0x..."
          class="w-full p-2 border rounded"
        >
        <p class="text-xs text-gray-500 mt-1">
          The session validator currently requires an explicit transfer policy target. Set this to the recipient you will send to.
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { watch } from "vue";
import { privateKeyToAccount, generatePrivateKey } from "viem/accounts";

interface SessionConfig {
  enabled: boolean;
  deployWithSession: boolean;
  validatorAddress: string;
  sessionPrivateKey: string;
  sessionSigner: string;
  expiresAt: number;
  feeLimit: string;
  // Optional: Restrict transfers to a single recipient (required by current validator)
  allowedRecipient?: string;
}

defineProps<{
  isDeployed?: boolean;
}>();

const config = defineModel<SessionConfig>();

// Format timestamp to readable date
function formatTimestamp(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleDateString("en-US", {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

// Auto-generate session signer on enable or deployWithSession
watch(
  () => [config.value?.enabled, config.value?.deployWithSession],
  ([enabled, deployWithSession]) => {
    if ((enabled || deployWithSession) && !config.value?.sessionPrivateKey) {
      const privateKey = generatePrivateKey();
      if (config.value) {
        config.value.sessionPrivateKey = privateKey;
        const account = privateKeyToAccount(privateKey);
        config.value.sessionSigner = account.address;
      }
    }
  },
);
</script>
