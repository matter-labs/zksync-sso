<!-- Simple test page to verify web SDK integration -->
<template>
  <div class="container mx-auto p-6">
    <h1 class="text-2xl font-bold mb-4">
      ZKSync SSO Web SDK Test
    </h1>

    <div class="bg-gray-100 p-4 rounded-lg mb-4">
      <h2 class="text-lg font-semibold mb-2">
        Web SDK Status
      </h2>
      <p class="text-sm text-gray-600 mb-2">
        Testing WASM-based ZKSync SSO ERC-4337 integration
      </p>

      <div class="space-y-2">
        <div>
          <strong>SDK Loaded:</strong>
          <span :class="sdkLoaded ? 'text-green-600' : 'text-red-600'">
            {{ sdkLoaded ? 'Yes' : 'No' }}
          </span>
        </div>

        <div v-if="sdkLoaded">
          <strong>Test Result:</strong>
          <span class="text-blue-600">{{ testResult }}</span>
        </div>

        <div v-if="error">
          <strong>Error:</strong>
          <span class="text-red-600">{{ error }}</span>
        </div>
      </div>

      <div class="flex gap-2 mt-4">
        <button
          :disabled="loading"
          class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:opacity-50"
          @click="testWebSDK"
        >
          {{ loading ? 'Testing...' : 'Test Web SDK' }}
        </button>

        <button
          :disabled="loading || !sdkLoaded"
          class="px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600 disabled:opacity-50"
          @click="deployAccount"
        >
          {{ loading ? 'Deploying...' : 'Deploy Account' }}
        </button>
      </div>
    </div>

    <!-- Passkey Configuration (Optional) -->
    <PasskeyConfig v-model="passkeyConfig" />

    <!-- Session Configuration (Optional) -->
    <div class="bg-teal-50 p-4 rounded-lg mb-4 border border-teal-200">
      <h2 class="text-lg font-semibold mb-3 text-teal-800">
        Session Configuration (Optional)
      </h2>
      <div class="space-y-3">
        <label class="flex items-center gap-2 text-sm">
          <input
            v-model="sessionConfig.enabled"
            type="checkbox"
          >
          Enable session at deploy
        </label>

        <div
          v-if="sessionConfig.enabled"
          class="grid grid-cols-1 md:grid-cols-2 gap-3"
        >
          <div>
            <label class="block text-sm font-medium mb-1">Session Signer Address</label>
            <input
              v-model="sessionConfig.signer"
              type="text"
              placeholder="0x..."
              class="w-full px-3 py-2 border border-gray-300 rounded text-sm font-mono"
            >
          </div>

          <div>
            <label class="block text-sm font-medium mb-1">Expires In (days)</label>
            <input
              v-model.number="sessionConfig.expiresInDays"
              type="number"
              min="1"
              class="w-full px-3 py-2 border border-gray-300 rounded text-sm"
            >
          </div>

          <div>
            <label class="block text-sm font-medium mb-1">Fee Limit (ETH)</label>
            <input
              v-model="sessionConfig.feeLimitEth"
              type="text"
              placeholder="0.1"
              class="w-full px-3 py-2 border border-gray-300 rounded text-sm"
            >
          </div>

          <div>
            <label class="block text-sm font-medium mb-1">Transfer To</label>
            <input
              v-model="sessionConfig.transfers[0].to"
              type="text"
              placeholder="0x..."
              class="w-full px-3 py-2 border border-gray-300 rounded text-sm font-mono"
            >
          </div>

          <div>
            <label class="block text-sm font-medium mb-1">Value Limit (ETH)</label>
            <input
              v-model="sessionConfig.transfers[0].valueLimitEth"
              type="text"
              placeholder="0.1"
              class="w-full px-3 py-2 border border-gray-300 rounded text-sm"
            >
          </div>
        </div>
      </div>
    </div>

    <!-- Wallet Configuration -->
    <WalletConfig v-model="walletConfig" />

    <!-- Account Deployment Result -->
    <div
      v-if="deploymentResult"
      class="bg-green-50 p-4 rounded-lg mb-4 border border-green-200"
    >
      <h2 class="text-lg font-semibold mb-2 text-green-800">
        Account Deployed Successfully!
      </h2>
      <div class="space-y-2 text-sm">
        <div v-if="deploymentResult.userId">
          <strong>User ID:</strong>
          <code class="bg-white px-2 py-1 rounded text-xs ml-2">{{ deploymentResult.userId }}</code>
        </div>
        <div v-if="deploymentResult.accountId">
          <strong>Account ID (computed):</strong>
          <code class="bg-white px-2 py-1 rounded text-xs ml-2 block mt-1">{{ deploymentResult.accountId }}</code>
        </div>
        <div>
          <strong>Account Address:</strong>
          <code class="bg-white px-2 py-1 rounded text-xs ml-2">{{ deploymentResult.address }}</code>
        </div>
        <div v-if="deploymentResult.eoaSigner">
          <strong>EOA Signer:</strong>
          <code class="bg-white px-2 py-1 rounded text-xs ml-2">{{ deploymentResult.eoaSigner }}</code>
          <span class="text-xs text-gray-600 ml-2">(Anvil Rich Wallet #1)</span>
        </div>
        <div v-if="deploymentResult.passkeyEnabled">
          <span><strong>Passkey Enabled:</strong> Yes</span>
        </div>
      </div>
    </div>

    <!-- Register Passkey (if passkey-enabled deployment) -->
    <div
      v-if="deploymentResult && deploymentResult.passkeyEnabled && !passkeyRegistered"
      class="bg-purple-50 p-4 rounded-lg mb-4 border border-purple-200"
    >
      <h2 class="text-lg font-semibold mb-3 text-purple-800">
        Step 1: Register Passkey with Validator
      </h2>
      <p class="text-sm text-gray-600 mb-4">
        The account was deployed with the WebAuthn validator installed, but the specific passkey needs to be registered.
        This requires sending a transaction signed by the EOA signer.
      </p>

      <button
        :disabled="loading"
        class="w-full px-4 py-2 bg-purple-500 text-white rounded hover:bg-purple-600 disabled:opacity-50"
        @click="registerPasskey"
      >
        {{ loading ? 'Registering Passkey...' : 'Register Passkey' }}
      </button>

      <!-- Registration Result -->
      <div
        v-if="passkeyRegisterResult"
        class="mt-4 p-3 bg-white rounded border border-purple-300"
      >
        <strong class="text-sm">Result:</strong>
        <p class="text-xs text-green-600 mt-1">
          {{ passkeyRegisterResult }}
        </p>
      </div>

      <div
        v-if="passkeyRegisterError"
        class="mt-4 p-3 bg-red-50 rounded border border-red-300"
      >
        <strong class="text-sm text-red-800">Error:</strong>
        <p class="text-xs text-red-600 mt-1">
          {{ passkeyRegisterError }}
        </p>
      </div>
    </div>

    <!-- Fund Smart Account -->
    <div
      v-if="deploymentResult && (!deploymentResult.passkeyEnabled || passkeyRegistered)"
      class="bg-orange-50 p-4 rounded-lg mb-4 border border-orange-200"
    >
      <h2 class="text-lg font-semibold mb-3 text-orange-800">
        {{ deploymentResult.passkeyEnabled ? 'Step 2: Fund Smart Account' : 'Step 1: Fund Smart Account' }}
      </h2>
      <p class="text-sm text-gray-600 mb-4">
        Send ETH from the EOA wallet to fund the smart account.
      </p>

      <div class="space-y-3">
        <div>
          <label class="block text-sm font-medium mb-1">Amount to Fund (ETH):</label>
          <input
            v-model="fundParams.amount"
            type="text"
            placeholder="0.1"
            class="w-full px-3 py-2 border border-gray-300 rounded text-sm"
          >
        </div>

        <button
          :disabled="loading || !fundParams.amount"
          class="w-full px-4 py-2 bg-orange-500 text-white rounded hover:bg-orange-600 disabled:opacity-50"
          @click="fundSmartAccount"
        >
          {{ loading ? 'Funding...' : 'Fund Smart Account' }}
        </button>
      </div>

      <!-- Funding Result -->
      <div
        v-if="fundResult"
        class="mt-4 p-3 bg-white rounded border border-orange-300"
      >
        <strong class="text-sm">Transaction Hash:</strong>
        <code class="block mt-1 px-2 py-1 bg-gray-100 rounded text-xs font-mono break-all">
          {{ fundResult }}
        </code>
      </div>

      <div
        v-if="fundError"
        class="mt-4 p-3 bg-red-50 rounded border border-red-300"
      >
        <strong class="text-sm text-red-800">Error:</strong>
        <p class="text-xs text-red-600 mt-1">
          {{ fundError }}
        </p>
      </div>
    </div>

    <!-- Send Transaction from Smart Account -->
    <TransactionSender
      v-if="deploymentResult && fundResult"
      :deployment-result="deploymentResult"
      :passkey-config="passkeyConfig"
    />

    <!-- Address Computation Testing -->
    <div class="bg-blue-50 p-4 rounded-lg mb-4 border border-blue-200">
      <h2 class="text-lg font-semibold mb-3 text-blue-800">
        Test Smart Account Address Computation
      </h2>
      <p class="text-sm text-gray-600 mb-4">
        Test the offline CREATE2 address computation. You'll need to provide the contract parameters from your deployed contracts.
      </p>

      <div class="space-y-3">
        <div>
          <label class="block text-sm font-medium mb-1">User ID:</label>
          <input
            v-model="addressParams.userId"
            type="text"
            placeholder="e.g., unique-id"
            class="w-full px-3 py-2 border border-gray-300 rounded text-sm"
          >
        </div>

        <div>
          <label class="block text-sm font-medium mb-1">Deploy Wallet Address:</label>
          <input
            v-model="addressParams.deployWallet"
            type="text"
            placeholder="0x..."
            class="w-full px-3 py-2 border border-gray-300 rounded text-sm font-mono"
          >
        </div>

        <div>
          <label class="block text-sm font-medium mb-1">Account Factory Address:</label>
          <input
            v-model="addressParams.factory"
            type="text"
            placeholder="0x..."
            class="w-full px-3 py-2 border border-gray-300 rounded text-sm font-mono"
          >
        </div>

        <div>
          <label class="block text-sm font-medium mb-1">Bytecode Hash (32 bytes hex with 0x):</label>
          <input
            v-model="addressParams.bytecodeHash"
            type="text"
            placeholder="0x..."
            class="w-full px-3 py-2 border border-gray-300 rounded text-sm font-mono"
          >
        </div>

        <div>
          <label class="block text-sm font-medium mb-1">Proxy Address (encoded beacon, hex with 0x):</label>
          <input
            v-model="addressParams.proxyAddress"
            type="text"
            placeholder="0x..."
            class="w-full px-3 py-2 border border-gray-300 rounded text-sm font-mono"
          >
        </div>

        <button
          :disabled="loading || !sdkLoaded || !isAddressParamsValid"
          class="w-full px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:opacity-50"
          @click="computeAddress"
        >
          {{ loading ? 'Computing...' : 'Compute Smart Account Address' }}
        </button>
      </div>

      <!-- Computed Address Result -->
      <div
        v-if="computedAddress"
        class="mt-4 p-3 bg-white rounded border border-blue-300"
      >
        <strong class="text-sm">Computed Address:</strong>
        <code class="block mt-1 px-2 py-1 bg-gray-100 rounded text-xs font-mono break-all">
          {{ computedAddress }}
        </code>
      </div>

      <div
        v-if="addressComputeError"
        class="mt-4 p-3 bg-red-50 rounded border border-red-300"
      >
        <strong class="text-sm text-red-800">Error:</strong>
        <p class="text-xs text-red-600 mt-1">
          {{ addressComputeError }}
        </p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, computed } from "vue";
import { hexToBytes } from "viem";

// Reactive state
const sdkLoaded = ref(false);
const testResult = ref("");
const error = ref("");
const loading = ref(false);
const deploymentResult = ref(null);

// Address computation parameters
const addressParams = ref({
  userId: "unique-id",
  deployWallet: "",
  factory: "",
  bytecodeHash: "",
  proxyAddress: "",
});

const computedAddress = ref("");
const addressComputeError = ref("");

// Passkey registration state
const passkeyRegistered = ref(false);
const passkeyRegisterResult = ref("");
const passkeyRegisterError = ref("");

// Fund smart account parameters
const fundParams = ref({
  amount: "0.1",
});
const fundResult = ref("");
const fundError = ref("");

// Passkey configuration state
const passkeyConfig = ref({
  enabled: false,
  credentialId: "0x2868baa08431052f6c7541392a458f64",
  passkeyX: "0xe0a43b9c64a2357ea7f66a0551f57442fbd32031162d9be762800864168fae40",
  passkeyY: "0x450875e2c28222e81eb25ae58d095a3e7ca295faa3fc26fb0e558a0b571da501",
  originDomain: window.location.origin,
  validatorAddress: "",
});

// Session configuration state
const sessionConfig = ref({
  enabled: false,
  signer: "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC", // Default Anvil/Foundry #2
  expiresInDays: 1,
  feeLimitEth: "0.1",
  transfers: [
    {
      to: "0x70997970C51812dc3A010C7d01b50e0d17dc79C8", // Example target
      valueLimitEth: "0.1",
    },
  ],
});

// Wallet configuration state
const walletConfig = ref({
  source: "anvil", // 'anvil' | 'private-key' | 'browser-wallet'
  anvilAccountIndex: 0,
  privateKey: "",
  connectedAddress: "",
  isReady: false,
});

// Anvil private keys for accounts 0-9
const anvilPrivateKeys = [
  "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80", // Account #0
  "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d", // Account #1
  "0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a", // Account #2
  "0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6", // Account #3
  "0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a", // Account #4
  "0x8b3a350cf5c34c9194ca85829a2df0ec3153be0318b5e2d3348e872092edffba", // Account #5
  "0x92db14e403b83dfe3df233f83dfa3a0d7096f21ca9b0d6d6b8d88b2b4ec1564e", // Account #6
  "0x4bbbf85ce3377467afe5d46f804f221813b2bb87f24d81f60f1fcdbf7cbf4356", // Account #7
  "0xdbda1821b80551c9d65939329250298aa3472ba22feea921c0cf5d620ea67b97", // Account #8
  "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6", // Account #9
];

/**
 * Get the private key for funding based on wallet configuration
 */
function getFundingPrivateKey() {
  if (walletConfig.value.source === "anvil") {
    const index = walletConfig.value.anvilAccountIndex;
    return anvilPrivateKeys[index];
  } else if (walletConfig.value.source === "private-key") {
    return walletConfig.value.privateKey;
  } else if (walletConfig.value.source === "browser-wallet") {
    // For browser wallet, we'll need to use a different approach (sign with provider)
    return null;
  }
  return null;
}

/**
 * Get a signer for funding transactions
 * Returns either a Wallet (for private key) or a BrowserProvider signer (for browser wallet)
 */
async function getFundingSigner() {
  const { ethers } = await import("ethers");

  // Load contracts.json to get RPC URL
  const response = await fetch("/contracts.json");
  const contracts = await response.json();
  const rpcUrl = contracts.rpcUrl;

  if (walletConfig.value.source === "browser-wallet") {
    // Use browser wallet provider
    if (!window.ethereum) {
      throw new Error("No Ethereum wallet detected");
    }
    const provider = new ethers.BrowserProvider(window.ethereum);
    return await provider.getSigner();
  } else {
    // Use private key (anvil or manual entry)
    const privateKey = getFundingPrivateKey();
    if (!privateKey) {
      throw new Error("No private key available");
    }
    const provider = new ethers.JsonRpcProvider(rpcUrl);
    return new ethers.Wallet(privateKey, provider);
  }
}

// Computed property to check if all address params are valid
const isAddressParamsValid = computed(() => {
  const params = addressParams.value;
  return (
    params.userId.length > 0
    && params.deployWallet.startsWith("0x") && params.deployWallet.length === 42
    && params.factory.startsWith("0x") && params.factory.length === 42
    && params.bytecodeHash.startsWith("0x") && params.bytecodeHash.length === 66
    && params.proxyAddress.startsWith("0x") && params.proxyAddress.length > 2
  );
});

// Test the web SDK
async function testWebSDK() {
  loading.value = true;
  error.value = "";
  testResult.value = "";

  try {
    // Import the web SDK utility functions
    const { compute_account_id } = await import("zksync-sso-web-sdk/bundler");

    // Test the compute_account_id function
    const testUserId = "test-user-123";
    const accountId = compute_account_id(testUserId);
    // eslint-disable-next-line no-console
    console.log("Computed account ID:", accountId);

    // Verify the account ID is a valid hex string
    if (!accountId.startsWith("0x") || accountId.length !== 66) {
      throw new Error("Invalid account ID format");
    }

    testResult.value = `✅ SDK functions working! Account ID: ${accountId.substring(0, 10)}...`;
    // eslint-disable-next-line no-console
    console.log("Web SDK utility functions tested successfully");
  } catch (err) {
    // eslint-disable-next-line no-console
    console.error("Web SDK test failed:", err);
    error.value = `Failed to test Web SDK: ${err.message}`;
  } finally {
    loading.value = false;
  }
}

/**
 * Load WebAuthn validator address from contracts.json
 */
async function loadWebAuthnValidatorAddress() {
  try {
    const response = await fetch("/contracts.json");
    if (response.ok) {
      const contracts = await response.json();
      passkeyConfig.value.validatorAddress = contracts.webauthnValidator;
      // eslint-disable-next-line no-console
      console.log("Loaded WebAuthn validator address:", contracts.webauthnValidator);
    } else {
      // eslint-disable-next-line no-console
      console.warn("contracts.json not found, cannot load WebAuthn validator address");
    }
  } catch (err) {
    // eslint-disable-next-line no-console
    console.warn("Failed to load contracts.json:", err);
  }
}

// Deploy a new smart account
async function deployAccount() {
  loading.value = true;
  error.value = "";
  deploymentResult.value = null;

  try {
    // Import the web SDK - destructure the WASM functions we need
    const { compute_account_id, DeployAccountConfig, deployAccountWithSession } = await import("zksync-sso-web-sdk/bundler");

    // Generate a user ID (in real app, this would be from authentication)
    const userId = "demo-user-" + Date.now();

    // Compute the account ID from user ID
    const accountId = compute_account_id(userId);
    // eslint-disable-next-line no-console
    console.log("Computed account ID:", accountId);

    // Load factory address from deployed contracts
    let factoryAddress = "0x679FFF51F11C3f6CaC9F2243f9D14Cb1255F65A3"; // Default fallback
    let rpcUrl = "http://localhost:8545"; // Default to Anvil
    let eoaValidatorAddress = null;
    let sessionValidatorAddress = null;

    try {
      // Try to load contracts.json if it exists
      const response = await fetch("/contracts.json");
      if (response.ok) {
        const contracts = await response.json();
        factoryAddress = contracts.factory;
        rpcUrl = contracts.rpcUrl;
        eoaValidatorAddress = contracts.eoaValidator;
        // eslint-disable-next-line no-console
        console.log("Loaded factory address from contracts.json:", factoryAddress);
        // eslint-disable-next-line no-console
        console.log("Loaded EOA validator address from contracts.json:", eoaValidatorAddress);
        // eslint-disable-next-line no-console
        console.log("Loaded WebAuthn validator address from contracts.json:", webAuthnValidatorAddress);
        // eslint-disable-next-line no-console
        console.log("Loaded Session validator address from contracts.json:", contracts.sessionValidator);
        sessionValidatorAddress = contracts.sessionValidator || null;
      } else {
        // eslint-disable-next-line no-console
        console.warn("contracts.json not found, using default factory address");
      }
    } catch (err) {
      // eslint-disable-next-line no-console
      console.warn("Failed to load contracts.json, using default factory address:", err);
    }

    // Add a rich Anvil wallet as an EOA signer for additional security
    // Using Anvil account #1 (0x70997970C51812dc3A010C7d01b50e0d17dc79C8)
    const eoaSignerAddress = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8";
    const eoaSignersAddresses = [eoaSignerAddress];

    // Get the deployer private key from wallet configuration
    const deployerPrivateKey = getFundingPrivateKey();
    if (!deployerPrivateKey) {
      throw new Error("No private key available. Please configure wallet settings.");
    }
    if (!walletConfig.value.isReady) {
      throw new Error("Wallet configuration incomplete. Please configure wallet settings.");
    }

    // eslint-disable-next-line no-console
    console.log("Deploying account...");
    // eslint-disable-next-line no-console
    console.log("  Funding Source:", walletConfig.value.source);
    if (walletConfig.value.source === "anvil") {
      // eslint-disable-next-line no-console
      console.log("  Using Anvil account #" + walletConfig.value.anvilAccountIndex + " as deployer");
    }
    // eslint-disable-next-line no-console
    console.log("  RPC URL:", rpcUrl);
    // eslint-disable-next-line no-console
    console.log("  Factory:", factoryAddress);
    // eslint-disable-next-line no-console
    console.log("  User ID:", userId);

    // Create passkey payload if enabled
    let passkeyPayload = null;
    let webauthnValidatorAddress = null;

    if (passkeyConfig.value.enabled) {
      // eslint-disable-next-line no-console
      console.log("Creating passkey payload...");

      try {
        const credentialId = hexToBytes(passkeyConfig.value.credentialId);
        const passkeyX = hexToBytes(passkeyConfig.value.passkeyX);
        const passkeyY = hexToBytes(passkeyConfig.value.passkeyY);

        // Validate coordinate lengths (must be 32 bytes)
        if (passkeyX.length !== 32) {
          throw new Error(`Passkey X coordinate must be 32 bytes, got ${passkeyX.length}`);
        }
        if (passkeyY.length !== 32) {
          throw new Error(`Passkey Y coordinate must be 32 bytes, got ${passkeyY.length}`);
        }

        // Import PasskeyPayload from SDK
        const { PasskeyPayload } = await import("zksync-sso-web-sdk/bundler");
        if (!PasskeyPayload) {
          throw new Error("PasskeyPayload class not found in SDK");
        }
        passkeyPayload = new PasskeyPayload(
          credentialId,
          passkeyX,
          passkeyY,
          passkeyConfig.value.originDomain,
        );

        // Set the webauthn validator address if provided
        if (passkeyConfig.value.validatorAddress) {
          webauthnValidatorAddress = passkeyConfig.value.validatorAddress;
        } else {
          throw new Error("WebAuthn validator address is required when using passkeys");
        }

        // eslint-disable-next-line no-console
        console.log("  Passkey payload created successfully");
        // eslint-disable-next-line no-console
        console.log("  WebAuthn Validator:", webauthnValidatorAddress);
      } catch (err) {
        throw new Error(`Failed to create passkey payload: ${err.message}`);
      }
    }

    // Construct the DeployAccountConfig wasm object
    // Include session validator in deploy config if present (optional arg)
    const deployConfig = new DeployAccountConfig(
      rpcUrl,
      factoryAddress,
      deployerPrivateKey,
      eoaValidatorAddress,
      webauthnValidatorAddress, // webauthn validator (null if not using passkeys)
      sessionValidatorAddress, // session validator (null if not using sessions)
    );

    // Optionally build session payload if enabled
    let session = null;
    if (sessionConfig.value.enabled) {
      const { ethers } = await import("ethers");
      const nowSec = Math.floor(Date.now() / 1000);
      const expiresAt = nowSec + sessionConfig.value.expiresInDays * 86400;
      session = {
        signer: sessionConfig.value.signer,
        expiresAt,
        feeLimit: {
          limitType: "lifetime",
          limit: ethers.parseEther(sessionConfig.value.feeLimitEth),
        },
        transfers: [
          {
            to: sessionConfig.value.transfers[0].to,
            valueLimit: ethers.parseEther(sessionConfig.value.transfers[0].valueLimitEth),
          },
        ],
      };
    }

    // Call the deployment function with the structured config
    // Use session-aware deploy wrapper (passes through to WASM)
    const deployedAddress = await deployAccountWithSession({
      userId,
      eoaSigners: eoaSignersAddresses,
      passkeyPayload, // null if not using passkeys
      session, // null if not using sessions
      deployConfig,
    });

    // eslint-disable-next-line no-console
    console.log("Account deployed at:", deployedAddress);

    // Display the deployment result
    deploymentResult.value = {
      userId,
      accountId,
      address: deployedAddress,
      eoaSigner: eoaSignerAddress,
      passkeyEnabled: passkeyConfig.value.enabled,
      sessionEnabled: sessionConfig.value.enabled,
    };

    // If passkey was provided during deployment, it's automatically registered
    if (passkeyConfig.value.enabled) {
      passkeyRegistered.value = true;
    }

    testResult.value = passkeyConfig.value.enabled
      ? "Account deployed successfully with EOA signer and WebAuthn passkey! (Passkey automatically registered)"
      : "Account deployed successfully with EOA signer!";

    // eslint-disable-next-line no-console
    console.log("Account deployment result:", deploymentResult.value);
  } catch (err) {
    // eslint-disable-next-line no-console
    console.error("Account deployment failed:", err);
    error.value = `Failed to deploy account: ${err.message}`;
  } finally {
    loading.value = false;
  }
}

// Register the passkey with the WebAuthn validator
async function registerPasskey() {
  loading.value = true;
  passkeyRegisterError.value = "";
  passkeyRegisterResult.value = "";

  try {
    // Import the WASM function
    const { add_passkey_to_account, SendTransactionConfig, PasskeyPayload } = await import("zksync-sso-web-sdk/bundler");

    // Load contracts.json
    const response = await fetch("/contracts.json");
    const contracts = await response.json();
    const rpcUrl = contracts.rpcUrl;
    const bundlerUrl = contracts.bundlerUrl || "http://localhost:4337";
    const entryPointAddress = contracts.entryPoint || "0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108";
    const eoaValidatorAddress = contracts.eoaValidator;

    // EOA signer private key (Anvil account #1) - to authorize the passkey registration
    const eoaSignerPrivateKey = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";

    // eslint-disable-next-line no-console
    console.log("Registering passkey with WebAuthn validator...");
    // eslint-disable-next-line no-console
    console.log("  Smart Account:", deploymentResult.value.address);
    // eslint-disable-next-line no-console
    console.log("  WebAuthn Validator:", passkeyConfig.value.validatorAddress);

    // Convert passkey coordinates to Uint8Array
    const credentialId = hexToBytes(passkeyConfig.value.credentialId);
    const passkeyX = hexToBytes(passkeyConfig.value.passkeyX);
    const passkeyY = hexToBytes(passkeyConfig.value.passkeyY);

    // Create PasskeyPayload
    const passkeyPayload = new PasskeyPayload(
      credentialId,
      passkeyX,
      passkeyY,
      passkeyConfig.value.originDomain,
    );

    // Create SendTransactionConfig
    const sendConfig = new SendTransactionConfig(
      rpcUrl,
      bundlerUrl,
      entryPointAddress,
    );

    // Call the WASM function
    const result = await add_passkey_to_account(
      sendConfig,
      deploymentResult.value.address,
      passkeyPayload,
      passkeyConfig.value.validatorAddress,
      eoaValidatorAddress,
      eoaSignerPrivateKey,
    );

    // eslint-disable-next-line no-console
    console.log("  Registration result:", result);

    if (result.startsWith("Failed") || result.startsWith("Error")) {
      throw new Error(result);
    }

    passkeyRegisterResult.value = result;
    passkeyRegistered.value = true;

    // eslint-disable-next-line no-console
    console.log("  Passkey registered successfully!");
  } catch (err) {
    // eslint-disable-next-line no-console
    console.error("Passkey registration failed:", err);
    passkeyRegisterError.value = `Failed to register passkey: ${err.message}`;
  } finally {
    loading.value = false;
  }
}

// Fund the smart account with ETH from EOA wallet
async function fundSmartAccount() {
  loading.value = true;
  fundError.value = "";
  fundResult.value = "";

  try {
    // Check if deployment was successful
    if (!deploymentResult.value || !deploymentResult.value.address) {
      throw new Error("Smart account not deployed yet");
    }

    // Check if wallet is configured
    if (!walletConfig.value.isReady) {
      throw new Error("Wallet not configured. Please configure wallet settings.");
    }

    // Validate the address looks like an Ethereum address
    const address = deploymentResult.value.address;
    if (!address || typeof address !== "string" || !address.match(/^0x[a-fA-F0-9]{40}$/)) {
      throw new Error(`Invalid smart account address: ${address}`);
    }

    // Import ethers to interact with the blockchain
    const { ethers } = await import("ethers");

    // eslint-disable-next-line no-console
    console.log("Funding smart account...");
    // eslint-disable-next-line no-console
    console.log("  Funding Source:", walletConfig.value.source);
    // eslint-disable-next-line no-console
    console.log("  To (Smart Account):", address);
    // eslint-disable-next-line no-console
    console.log("  Amount:", fundParams.value.amount, "ETH");

    // Get the signer based on wallet configuration
    const signer = await getFundingSigner();
    const signerAddress = await signer.getAddress();

    // eslint-disable-next-line no-console
    console.log("  From (Funding Wallet):", signerAddress);

    // Convert amount to wei
    const amountWei = ethers.parseEther(fundParams.value.amount);

    // Ensure address is properly formatted (prevents ENS lookup on non-ENS networks)
    const toAddress = ethers.getAddress(address);

    // Send transaction to fund the smart account
    const tx = await signer.sendTransaction({
      to: toAddress,
      value: amountWei,
    });

    // eslint-disable-next-line no-console
    console.log("  Transaction sent:", tx.hash);

    // Wait for confirmation
    const receipt = await tx.wait();

    // eslint-disable-next-line no-console
    console.log("  Transaction confirmed in block:", receipt.blockNumber);

    fundResult.value = tx.hash;

    // Check the balance
    const provider = signer.provider;
    const balance = await provider.getBalance(deploymentResult.value.address);
    // eslint-disable-next-line no-console
    console.log("  Smart account balance:", ethers.formatEther(balance), "ETH");
  } catch (err) {
    // eslint-disable-next-line no-console
    console.error("Funding failed:", err);
    fundError.value = `Failed to fund smart account: ${err.message}`;
  } finally {
    loading.value = false;
  }
}

// Check if SDK is available on mount
onMounted(async () => {
  // Only run on client side
  if (import.meta.client) {
    try {
      await import("zksync-sso-web-sdk/bundler");
      sdkLoaded.value = true;

      // Load WebAuthn validator address from contracts.json
      await loadWebAuthnValidatorAddress();
    } catch (err) {
      // eslint-disable-next-line no-console
      console.error("Failed to load Web SDK:", err);
      error.value = `Failed to load Web SDK: ${err.message}`;
    }
  }
});
</script>
