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

    <!-- HTTP Transport Test -->
    <div class="bg-purple-50 p-4 rounded-lg mb-4 border border-purple-200">
      <h2 class="text-lg font-semibold mb-3 text-purple-800">
        Test HTTP Transport (reqwasm)
      </h2>
      <p class="text-sm text-gray-600 mb-4">
        Test that reqwasm can make HTTP calls from WASM. This makes a simple eth_chainId RPC call.
      </p>

      <div class="space-y-3">
        <div>
          <label class="block text-sm font-medium mb-1">RPC URL:</label>
          <input
            v-model="httpTestParams.rpcUrl"
            type="text"
            placeholder="https://..."
            class="w-full px-3 py-2 border border-gray-300 rounded text-sm font-mono"
          >
        </div>

        <button
          :disabled="loading || !sdkLoaded || !httpTestParams.rpcUrl"
          class="w-full px-4 py-2 bg-purple-500 text-white rounded hover:bg-purple-600 disabled:opacity-50"
          @click="testHttpTransport"
        >
          {{ loading ? 'Testing...' : 'Test HTTP Transport' }}
        </button>
      </div>

      <!-- HTTP Test Result -->
      <div
        v-if="httpTestResult"
        class="mt-4 p-3 bg-white rounded border border-purple-300"
      >
        <strong class="text-sm">Result:</strong>
        <code class="block mt-1 px-2 py-1 bg-gray-100 rounded text-xs font-mono break-all">
          {{ httpTestResult }}
        </code>
      </div>

      <div
        v-if="httpTestError"
        class="mt-4 p-3 bg-red-50 rounded border border-red-300"
      >
        <strong class="text-sm text-red-800">Error:</strong>
        <p class="text-xs text-red-600 mt-1">
          {{ httpTestError }}
        </p>
      </div>
    </div>

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
import { Wallet } from "ethers";
import { ref, onMounted, computed } from "vue";

// Reactive state
const sdkLoaded = ref(false);
const testResult = ref("");
const error = ref("");
const loading = ref(false);
const deploymentResult = ref(null);

// HTTP transport test parameters
const httpTestParams = ref({
  rpcUrl: "https://sepolia.era.zksync.dev",
});
const httpTestResult = ref("");
const httpTestError = ref("");

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

/**
 * Convert a hex string to a Uint8Array of bytes
 */
function hexToBytes(hex) {
  // Remove 0x prefix if present
  const cleanHex = hex.startsWith("0x") ? hex.slice(2) : hex;

  // Validate hex string
  if (cleanHex.length % 2 !== 0) {
    throw new Error(`Invalid hex string length: ${cleanHex.length}`);
  }

  const bytes = new Uint8Array(cleanHex.length / 2);
  for (let i = 0; i < cleanHex.length; i += 2) {
    bytes[i / 2] = parseInt(cleanHex.substr(i, 2), 16);
  }

  return bytes;
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
    // Import the web SDK (dynamic import for client-side only)
    const { ZkSyncSsoClient } = await import("zksync-sso-web-sdk/bundler");

    // Test basic configuration
    const config = {
      rpcUrl: "https://sepolia.era.zksync.dev",
      bundlerUrl: "https://bundler.example.com",
      contracts: {
        entryPoint: "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789",
        accountFactory: "0x9406Cc6185a346906296840746125a0E44976454",
      },
    };

    // Create client instance (this will use stub implementation for now)
    // For demo purposes, use a dummy private key (DO NOT use in production)
    const dummyPrivateKey = Wallet.createRandom().privateKey;
    const client = new ZkSyncSsoClient(config, dummyPrivateKey);

    testResult.value = "Web SDK client created successfully!";
    // eslint-disable-next-line no-console
    console.log("ZKSync SSO Web SDK client:", client);
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
    const { deploy_account, compute_account_id, DeployAccountConfig } = await import("zksync-sso-web-sdk/bundler");

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
      } else {
        // eslint-disable-next-line no-console
        console.warn("contracts.json not found, using default factory address");
      }
    } catch (err) {
      // eslint-disable-next-line no-console
      console.warn("Failed to load contracts.json, using default factory address:", err);
    }

    // Add a rich Anvil wallet as an EOA signer for additional security
    // Select which Anvil account to use based on URL parameter
    // This allows parallel tests to use different accounts to avoid nonce conflicts
    const urlParams = new URLSearchParams(window.location.search);
    const accountIndex = parseInt(urlParams.get("fundingAccount") || "0", 10);

    // Anvil test accounts (first 10) - private keys
    const anvilPrivateKeys = [
      "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80", // #0
      "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d", // #1
      "0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a", // #2
      "0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6", // #3
      "0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a", // #4
      "0x8b3a350cf5c34c9194ca85829a2df0ec3153be0318b5e2d3348e872092edffba", // #5
      "0x92db14e403b83dfe3df233f83dfa3a0d7096f21ca9b0d6d6b8d88b2b4ec1564e", // #6
      "0x4bbbf85ce3377467afe5d46f804f221813b2bb87f24d81f60f1fcdbf7cbf4356", // #7
      "0xdbda1821b80551c9d65939329250298aa3472ba22feea921c0cf5d620ea67b97", // #8
      "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6", // #9
    ];

    // Corresponding public addresses
    const anvilAddresses = [
      "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266", // #0
      "0x70997970C51812dc3A010C7d01b50e0d17dc79C8", // #1
      "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC", // #2
      "0x90F79bf6EB2c4f870365E785982E1f101E93b906", // #3
      "0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65", // #4
      "0x9965507D1a55bcC2695C58ba16FB37d819B0A4dc", // #5
      "0x976EA74026E726554dB657fA54763abd0C3a0aa9", // #6
      "0x14dC79964da2C08b23698B3D3cc7Ca32193d9955", // #7
      "0x23618e81E3f5cdF7f54C3d65f7FBc0aBf5B21E8f", // #8
      "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720", // #9
    ];

    const eoaSignerAddress = anvilAddresses[accountIndex] || anvilAddresses[0];
    const eoaSignersAddresses = [eoaSignerAddress];

    // Use the appropriate private key based on the network
    const deployerPrivateKey = anvilPrivateKeys[accountIndex] || anvilPrivateKeys[0];

    // eslint-disable-next-line no-console
    console.log("  Using Anvil account #" + accountIndex + " for deployment");

    // eslint-disable-next-line no-console
    console.log("Deploying account...");
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
    const deployConfig = new DeployAccountConfig(
      rpcUrl,
      factoryAddress,
      deployerPrivateKey,
      eoaValidatorAddress,
      webauthnValidatorAddress, // webauthn validator (null if not using passkeys)
    );

    // Call the deployment function with the structured config
    const deployedAddress = await deploy_account(
      userId,
      eoaSignersAddresses,
      passkeyPayload, // passkey payload (null if not using passkeys)
      deployConfig,
    );

    // eslint-disable-next-line no-console
    console.log("Account deployed at:", deployedAddress);

    // Display the deployment result
    deploymentResult.value = {
      userId,
      accountId,
      address: deployedAddress,
      eoaSigner: eoaSignerAddress,
      passkeyEnabled: passkeyConfig.value.enabled,
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

    // Validate the address looks like an Ethereum address
    const address = deploymentResult.value.address;
    if (!address || typeof address !== "string" || !address.match(/^0x[a-fA-F0-9]{40}$/)) {
      throw new Error(`Invalid smart account address: ${address}`);
    }

    // Import ethers to interact with the blockchain
    const { ethers } = await import("ethers");

    // Load contracts.json to get RPC URL
    const response = await fetch("/contracts.json");
    const contracts = await response.json();
    const rpcUrl = contracts.rpcUrl;

    // eslint-disable-next-line no-console
    console.log("Funding smart account...");
    // eslint-disable-next-line no-console
    console.log("  From (EOA):", deploymentResult.value.eoaSigner);
    // eslint-disable-next-line no-console
    console.log("  To (Smart Account):", address);
    // eslint-disable-next-line no-console
    console.log("  Amount:", fundParams.value.amount, "ETH");

    // Create provider
    const provider = new ethers.JsonRpcProvider(rpcUrl);

    // Select which Anvil account to use for funding based on URL parameter
    // This allows parallel tests to use different accounts to avoid nonce conflicts
    // Default to account #1, but tests can specify ?fundingAccount=2 to use account #2
    const urlParams = new URLSearchParams(window.location.search);
    const fundingAccountIndex = parseInt(urlParams.get("fundingAccount") || "1", 10);

    // Anvil test accounts (first 10)
    const anvilAccounts = [
      "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80", // #0
      "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d", // #1
      "0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a", // #2
      "0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6", // #3
      "0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a", // #4
      "0x8b3a350cf5c34c9194ca85829a2df0ec3153be0318b5e2d3348e872092edffba", // #5
      "0x92db14e403b83dfe3df233f83dfa3a0d7096f21ca9b0d6d6b8d88b2b4ec1564e", // #6
      "0x4bbbf85ce3377467afe5d46f804f221813b2bb87f24d81f60f1fcdbf7cbf4356", // #7
      "0xdbda1821b80551c9d65939329250298aa3472ba22feea921c0cf5d620ea67b97", // #8
      "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6", // #9
    ];

    const eoaSignerPrivateKey = anvilAccounts[fundingAccountIndex] || anvilAccounts[1];
    const eoaSigner = new ethers.Wallet(eoaSignerPrivateKey, provider);

    // eslint-disable-next-line no-console
    console.log("  Using Anvil account #" + fundingAccountIndex + " for funding");

    // Convert amount to wei
    const amountWei = ethers.parseEther(fundParams.value.amount);

    // Ensure address is properly formatted (prevents ENS lookup on non-ENS networks)
    const toAddress = ethers.getAddress(address);

    // Send transaction to fund the smart account
    const tx = await eoaSigner.sendTransaction({
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

// Test HTTP transport with reqwasm
async function testHttpTransport() {
  loading.value = true;
  httpTestError.value = "";
  httpTestResult.value = "";

  try {
    // Import the test function
    const { test_http_transport } = await import("zksync-sso-web-sdk/bundler");

    // Call the WASM function
    const result = await test_http_transport(httpTestParams.value.rpcUrl);

    httpTestResult.value = result;
    // eslint-disable-next-line no-console
    console.log("HTTP transport test result:", result);
  } catch (err) {
    // eslint-disable-next-line no-console
    console.error("HTTP transport test failed:", err);
    httpTestError.value = `Failed to test HTTP transport: ${err.message}`;
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
