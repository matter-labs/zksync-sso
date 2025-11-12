<template>
  <div class="bg-indigo-50 p-4 rounded-lg border border-indigo-200 mb-4">
    <h2 class="text-lg font-semibold mb-3 text-indigo-800">
      Wallet Configuration
    </h2>
    <p class="text-sm text-gray-600 mb-4">
      Configure the funding source for account deployment and transactions.
      For automated tests, Anvil accounts are used by default.
    </p>

    <div class="space-y-3">
      <!-- Network Detection -->
      <div class="p-3 bg-white rounded border border-indigo-200">
        <div class="text-sm">
          <strong>Detected Environment:</strong>
          <span
            :class="isAnvil ? 'text-green-600' : 'text-blue-600'"
            class="ml-2"
          >
            {{ isAnvil ? 'Anvil (Local Testnet)' : 'Custom Network' }}
          </span>
        </div>
        <div
          v-if="rpcUrl"
          class="text-xs text-gray-600 mt-1"
        >
          RPC: {{ rpcUrl }}
        </div>
      </div>

      <!-- Wallet Source Selection -->
      <div>
        <label class="block text-sm font-medium mb-2">Funding Source:</label>
        <div class="space-y-2">
          <label class="flex items-center">
            <input
              v-model="config.source"
              type="radio"
              value="anvil"
              class="mr-2"
              :disabled="!isAnvil"
            >
            <span :class="!isAnvil ? 'text-gray-400' : ''">
              Anvil Test Account (Automated Testing)
            </span>
          </label>
          <label class="flex items-center">
            <input
              v-model="config.source"
              type="radio"
              value="private-key"
              class="mr-2"
            >
            <span>Private Key (Manual Entry)</span>
          </label>
          <label class="flex items-center">
            <input
              v-model="config.source"
              type="radio"
              value="browser-wallet"
              class="mr-2"
            >
            <span>Browser Wallet (MetaMask, etc.)</span>
          </label>
        </div>
      </div>

      <!-- Anvil Account Selection -->
      <div
        v-if="config.source === 'anvil'"
        class="pl-6 border-l-2 border-indigo-300"
      >
        <label class="block text-sm font-medium mb-1">Anvil Account:</label>
        <select
          v-model="config.anvilAccountIndex"
          class="w-full px-3 py-2 border border-gray-300 rounded text-sm"
        >
          <option
            v-for="i in 10"
            :key="i - 1"
            :value="i - 1"
          >
            Account #{{ i - 1 }} ({{ anvilAddresses[i - 1] }})
          </option>
        </select>
        <p class="text-xs text-gray-500 mt-1">
          Select which Anvil test account to use for funding
        </p>
      </div>

      <!-- Private Key Entry -->
      <div
        v-if="config.source === 'private-key'"
        class="pl-6 border-l-2 border-indigo-300"
      >
        <label class="block text-sm font-medium mb-1">Private Key:</label>
        <input
          v-model="config.privateKey"
          type="password"
          placeholder="0x..."
          class="w-full px-3 py-2 border border-gray-300 rounded text-sm font-mono"
        >
        <p class="text-xs text-gray-500 mt-1">
          Enter your private key (will not be stored)
        </p>
        <p class="text-xs text-red-600 mt-1">
          ⚠️ Only use test accounts! Never enter mainnet private keys.
        </p>
      </div>

      <!-- Browser Wallet Connection -->
      <div
        v-if="config.source === 'browser-wallet'"
        class="pl-6 border-l-2 border-indigo-300"
      >
        <button
          v-if="!connectedAddress"
          :disabled="connecting"
          class="w-full px-4 py-2 bg-indigo-600 text-white rounded hover:bg-indigo-700 disabled:opacity-50"
          @click="connectBrowserWallet"
        >
          {{ connecting ? 'Connecting...' : 'Connect Wallet' }}
        </button>

        <div
          v-else
          class="space-y-2"
        >
          <div class="p-3 bg-green-50 rounded border border-green-300">
            <div class="text-sm font-medium text-green-800">
              Connected
            </div>
            <code class="text-xs text-green-600 block mt-1">
              {{ connectedAddress }}
            </code>
          </div>
          <button
            class="w-full px-4 py-2 bg-gray-500 text-white rounded hover:bg-gray-600"
            @click="disconnectWallet"
          >
            Disconnect
          </button>
        </div>

        <p
          v-if="walletError"
          class="text-xs text-red-600 mt-2"
        >
          {{ walletError }}
        </p>
      </div>

      <!-- Current Configuration Summary -->
      <div class="p-3 bg-white rounded border border-indigo-200">
        <div class="text-sm font-medium mb-2">
          Current Configuration:
        </div>
        <div class="text-xs space-y-1">
          <div>
            <strong>Source:</strong>
            {{
              config.source === 'anvil'
                ? 'Anvil Test Account'
                : config.source === 'private-key'
                  ? 'Private Key'
                  : 'Browser Wallet'
            }}
          </div>
          <div v-if="config.source === 'anvil'">
            <strong>Account:</strong> #{{ config.anvilAccountIndex }}
          </div>
          <div v-if="config.source === 'private-key' && config.privateKey">
            <strong>Private Key:</strong> {{ config.privateKey.substring(0, 10) }}...
          </div>
          <div v-if="config.source === 'browser-wallet' && connectedAddress">
            <strong>Address:</strong> {{ connectedAddress.substring(0, 10) }}...
          </div>
          <div
            v-if="isReady"
            class="text-green-600 mt-2"
          >
            ✓ Ready to use
          </div>
          <div
            v-else
            class="text-orange-600 mt-2"
          >
            ⚠ Configuration incomplete
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, ref, watch, onMounted } from "vue";
import { loadContracts } from "~/utils/contracts";

// Props
const props = defineProps({
  modelValue: {
    type: Object,
    required: true,
  },
});

// Emits
const emit = defineEmits(["update:modelValue"]);

// Local state for two-way binding
const config = computed({
  get: () => props.modelValue,
  set: (value) => emit("update:modelValue", value),
});

// Wallet connection state
const connecting = ref(false);
const connectedAddress = ref("");
const walletError = ref("");
const rpcUrl = ref("");
const isAnvil = ref(true);

// Anvil test account addresses
const anvilAddresses = [
  "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266", // Account #0
  "0x70997970C51812dc3A010C7d01b50e0d17dc79C8", // Account #1
  "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC", // Account #2
  "0x90F79bf6EB2c4f870365E785982E1f101E93b906", // Account #3
  "0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65", // Account #4
  "0x9965507D1a55bcC2695C58ba16FB37d819B0A4dc", // Account #5
  "0x976EA74026E726554dB657fA54763abd0C3a0aa9", // Account #6
  "0x14dC79964da2C08b23698B3D3cc7Ca32193d9955", // Account #7
  "0x23618e81E3f5cdF7f54C3d65f7FBc0aBf5B21E8f", // Account #8
  "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720", // Account #9
];

// Check if configuration is ready to use
const isReady = computed(() => {
  if (config.value.source === "anvil") {
    return isAnvil.value && config.value.anvilAccountIndex !== null;
  } else if (config.value.source === "private-key") {
    return config.value.privateKey && config.value.privateKey.startsWith("0x") && config.value.privateKey.length === 66;
  } else if (config.value.source === "browser-wallet") {
    return connectedAddress.value !== "";
  }
  return false;
});

// Watch for changes to emit ready state
watch(isReady, (ready) => {
  config.value.isReady = ready;
  if (config.value.source === "browser-wallet") {
    config.value.connectedAddress = connectedAddress.value;
  }
}, { immediate: true, flush: "sync" });

/**
 * Connect to browser wallet (MetaMask, etc.)
 */
async function connectBrowserWallet() {
  connecting.value = true;
  walletError.value = "";

  try {
    // Check if ethereum provider is available
    if (!window.ethereum) {
      throw new Error("No Ethereum wallet detected. Please install MetaMask or another Web3 wallet.");
    }

    // Request account access
    const accounts = await window.ethereum.request({
      method: "eth_requestAccounts",
    });

    if (accounts.length === 0) {
      throw new Error("No accounts found. Please unlock your wallet.");
    }

    connectedAddress.value = accounts[0];

    // Listen for account changes
    window.ethereum.on("accountsChanged", handleAccountsChanged);

    // eslint-disable-next-line no-console
    console.log("Connected to wallet:", connectedAddress.value);
  } catch (err) {
    // eslint-disable-next-line no-console
    console.error("Failed to connect wallet:", err);
    walletError.value = err.message;
  } finally {
    connecting.value = false;
  }
}

/**
 * Disconnect wallet
 */
function disconnectWallet() {
  connectedAddress.value = "";
  if (window.ethereum) {
    window.ethereum.removeListener("accountsChanged", handleAccountsChanged);
  }
}

/**
 * Handle account changes from wallet
 */
function handleAccountsChanged(accounts) {
  if (accounts.length === 0) {
    // User disconnected wallet
    disconnectWallet();
  } else {
    connectedAddress.value = accounts[0];
    // eslint-disable-next-line no-console
    console.log("Wallet account changed:", connectedAddress.value);
  }
}

/**
 * Detect network environment and check for query parameters
 */
async function detectEnvironment() {
  // First, detect if we're on Anvil
  try {
    const contracts = await loadContracts();
    rpcUrl.value = contracts.rpcUrl || "";
    isAnvil.value = rpcUrl.value.includes("localhost:8545") || rpcUrl.value.includes("127.0.0.1:8545");

    // Auto-select appropriate source
    if (isAnvil.value && config.value.source === "browser-wallet") {
      config.value.source = "anvil";
    } else if (!isAnvil.value && config.value.source === "anvil") {
      config.value.source = "browser-wallet";
    }
  } catch (err) {
    // eslint-disable-next-line no-console
    console.warn("Could not load contracts.json:", err);
  }

  // Then, check for fundingAccount query parameter (for E2E tests)
  // This must happen after we've detected isAnvil
  if (typeof window !== "undefined") {
    const urlParams = new URLSearchParams(window.location.search);
    const fundingAccountParam = urlParams.get("fundingAccount");
    if (fundingAccountParam !== null) {
      const accountIndex = parseInt(fundingAccountParam, 10);
      if (!isNaN(accountIndex) && accountIndex >= 0 && accountIndex <= 9) {
        // eslint-disable-next-line no-console
        console.log("Setting Anvil account from query parameter:", accountIndex);
        config.value.source = "anvil";
        config.value.anvilAccountIndex = accountIndex;
        // Since isAnvil might already be true from detectEnvironment,
        // the isReady computed property should now evaluate to true
      }
    }
  }
}

// Initialize on mount
onMounted(() => {
  detectEnvironment();
});
</script>
