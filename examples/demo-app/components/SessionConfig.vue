<template>
  <div class="bg-teal-50 p-4 rounded-lg mb-4 border border-teal-200">
    <h2 class="text-lg font-semibold mb-3 text-teal-800">
      Session Configuration (Optional)
    </h2>
    <p class="text-sm text-gray-600 mb-4">
      Configure a session key for gasless operations within defined limits. Leave disabled to deploy without session support.
    </p>

    <div class="space-y-3">
      <div>
        <label class="flex items-center gap-2 text-sm">
          <input
            v-model="config.enabled"
            type="checkbox"
          >
          Enable session at deploy
        </label>
      </div>

      <div
        v-if="config.enabled"
        class="grid grid-cols-1 md:grid-cols-2 gap-3"
      >
        <div>
          <label class="block text-sm font-medium mb-1">Session Signer Address</label>
          <input
            v-model="config.signer"
            type="text"
            placeholder="0x..."
            readonly
            class="w-full px-3 py-2 border border-gray-300 rounded text-sm font-mono bg-gray-50"
          >
          <p class="text-xs text-gray-500 mt-1">
            Auto-generated unique session signer
          </p>
        </div>

        <div>
          <label class="block text-sm font-medium mb-1">Session Private Key</label>
          <input
            v-model="config.privateKey"
            type="text"
            placeholder="0x..."
            readonly
            class="w-full px-3 py-2 border border-gray-300 rounded text-sm font-mono bg-gray-50"
          >
          <p class="text-xs text-gray-500 mt-1">
            Use this to sign session transactions
          </p>
        </div>

        <div class="md:col-span-2">
          <button
            type="button"
            class="px-4 py-2 bg-teal-600 text-white text-sm rounded hover:bg-teal-700 transition-colors"
            @click="generateSessionKey(true)"
          >
            🔄 Generate New Session Key
          </button>
          <p class="text-xs text-gray-500 mt-1">
            Creates a new random session signer address and private key
          </p>
        </div>

        <div>
          <label class="block text-sm font-medium mb-1">Expires In (days)</label>
          <input
            v-model.number="config.expiresInDays"
            type="number"
            min="1"
            class="w-full px-3 py-2 border border-gray-300 rounded text-sm"
          >
          <p class="text-xs text-gray-500 mt-1">
            How long the session remains valid
          </p>
        </div>

        <div>
          <label class="block text-sm font-medium mb-1">Fee Limit (ETH)</label>
          <input
            v-model="config.feeLimitEth"
            type="text"
            placeholder="0.1"
            class="w-full px-3 py-2 border border-gray-300 rounded text-sm"
          >
          <p class="text-xs text-gray-500 mt-1">
            Maximum fees this session can pay
          </p>
        </div>

        <div>
          <label class="block text-sm font-medium mb-1">Transfer To</label>
          <input
            v-model="config.transfers[0].to"
            type="text"
            placeholder="0x..."
            class="w-full px-3 py-2 border border-gray-300 rounded text-sm font-mono"
          >
          <p class="text-xs text-gray-500 mt-1">
            Allowed recipient address
          </p>
        </div>

        <div>
          <label class="block text-sm font-medium mb-1">Value Limit (ETH)</label>
          <input
            v-model="config.transfers[0].valueLimitEth"
            type="text"
            placeholder="0.1"
            class="w-full px-3 py-2 border border-gray-300 rounded text-sm"
          >
          <p class="text-xs text-gray-500 mt-1">
            Maximum ETH per transfer
          </p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, onMounted, watch } from "vue";

// Define props and emits for v-model
const props = defineProps({
  modelValue: {
    type: Object,
    required: true,
  },
});

const emit = defineEmits(["update:modelValue"]);

// Create a local computed property that syncs with parent
const config = computed({
  get() {
    return props.modelValue;
  },
  set(value) {
    emit("update:modelValue", value);
  },
});

// Generate a random session key (can be called manually or automatically)
async function generateSessionKey(force = false) {
  // Only auto-generate if session doesn't already have a signer/private key
  if (!force && config.value.signer && config.value.privateKey) {
    return;
  }

  try {
    // Import ethers to generate a random wallet
    const { ethers } = await import("ethers");
    const randomWallet = ethers.Wallet.createRandom();

    // Update the config with the new session signer and private key
    config.value = {
      ...config.value,
      signer: randomWallet.address,
      privateKey: randomWallet.privateKey,
    };

    // eslint-disable-next-line no-console
    console.log("Generated new session key:", randomWallet.address);
  } catch (err) {
    // eslint-disable-next-line no-console
    console.error("Failed to generate session key:", err);
  }
}

// Generate session key when component mounts
onMounted(() => {
  generateSessionKey();
});

// Generate a new session key when session is enabled
watch(() => config.value.enabled, (isEnabled) => {
  if (isEnabled && (!config.value.signer || !config.value.privateKey)) {
    generateSessionKey();
  }
});
</script>
