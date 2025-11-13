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

    <!-- Session Configuration -->
    <SessionConfig
      v-model="sessionConfig"
      :is-deployed="!!deploymentResult"
    />

    <!-- Create Session (must be done before sending session transactions) -->
    <SessionCreator
      v-if="deploymentResult && sessionConfig.enabled && fundResult && !sessionCreated && eoaValidatorAddress"
      :account-address="deploymentResult.address"
      :session-config="sessionConfig"
      :eoa-validator-address="eoaValidatorAddress"
      :eoa-private-key="eoaSignerPrivateKey"
      @session-created="handleSessionCreated"
    />

    <!-- Session Create Success Banner (persists after child unmount) -->
    <div
      v-if="deploymentResult && sessionConfig.enabled && fundResult && sessionCreated"
      class="mt-4 p-3 bg-green-50 border border-green-200 rounded"
    >
      <p class="font-medium text-green-800">
        Session Created Successfully!
      </p>
    </div>

    <!-- Send Session Transaction -->
    <SessionTransactionSender
      v-if="deploymentResult && sessionConfig.enabled && fundResult && sessionCreated"
      :account-address="deploymentResult.address"
      :session-config="sessionConfig"
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

<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { createPublicClient, createWalletClient, http, hexToBytes, type Chain, type Hash, type Hex, type Address } from "viem";
import { privateKeyToAccount } from "viem/accounts";
import { ethers } from "ethers";
import { deploySmartAccount, getDeployedAccountAddress } from "zksync-sso-4337/client";
import { compute_account_id, add_passkey_to_account, SendTransactionConfig, PasskeyPayload } from "zksync-sso-web-sdk/bundler";

// Types
interface DeploymentResult {
  userId: string;
  accountId: string;
  address: string;
  eoaSigner: string;
  passkeyEnabled: boolean;
}

// Reactive state
const sdkLoaded = ref(false);
const testResult = ref("");
const error = ref("");
const loading = ref(false);
const deploymentResult = ref<DeploymentResult | null>(null);

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

// Wallet configuration state
const walletConfig = ref({
  source: "anvil", // 'anvil' | 'private-key' | 'browser-wallet'
  anvilAccountIndex: 0,
  privateKey: "",
  connectedAddress: "",
  isReady: false,
});

// Session configuration state
const sessionConfig = ref({
  enabled: false,
  deployWithSession: false,
  validatorAddress: "",
  sessionPrivateKey: "",
  sessionSigner: "",
  expiresAt: Math.floor(Date.now() / 1000) + 3600, // 1 hour from now
  feeLimit: "1000000000000000", // 0.001 ETH in wei
});

// Session creation state
const sessionCreated = ref(false);
const eoaValidatorAddress = ref("");
const eoaSignerPrivateKey = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"; // Anvil account #1

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
    return anvilPrivateKeys[index] as Hash;
  } else if (walletConfig.value.source === "private-key") {
    return walletConfig.value.privateKey as Hash;
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
  } catch (err: unknown) {
    // eslint-disable-next-line no-console
    console.error("Web SDK test failed:", err);
    error.value = `Failed to test Web SDK: ${err instanceof Error ? err.message : String(err)}`;
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
      sessionConfig.value.validatorAddress = contracts.sessionValidator || "";
      eoaValidatorAddress.value = contracts.eoaValidator || "";
      // eslint-disable-next-line no-console
      console.log("Loaded WebAuthn validator address:", contracts.webauthnValidator);
      // eslint-disable-next-line no-console
      console.log("Loaded Session validator address:", contracts.sessionValidator);
      // eslint-disable-next-line no-console
      console.log("Loaded EOA validator address:", contracts.eoaValidator);
    } else {
      // eslint-disable-next-line no-console
      console.warn("contracts.json not found, cannot load validator addresses");
    }
  } catch (err: unknown) {
    // eslint-disable-next-line no-console
    console.warn("Failed to load contracts.json:", err);
  }
}

/**
 * Handler for when session is created
 */
function handleSessionCreated() {
  sessionCreated.value = true;
  // eslint-disable-next-line no-console
  console.log("Session created successfully, ready to send session transactions");
}

// Deploy a new smart account
async function deployAccount() {
  loading.value = true;
  error.value = "";
  deploymentResult.value = null;

  try {
    // Generate a user ID (in real app, this would be from authentication)
    const userId = "demo-user-" + Date.now();

    // Load factory address from deployed contracts
    let factoryAddress: Hex = "0x679FFF51F11C3f6CaC9F2243f9D14Cb1255F65A3"; // Default fallback
    let rpcUrl = "http://localhost:8545"; // Default to Anvil
    let chainId = 1337; // Default to Anvil
    let eoaValidatorAddress: Hex | undefined = undefined;
    let webauthnValidatorAddress: Hex | undefined = undefined;

    try {
      // Try to load contracts.json if it exists
      const response = await fetch("/contracts.json");
      if (response.ok) {
        const contracts = await response.json();
        factoryAddress = contracts.factory;
        rpcUrl = contracts.rpcUrl;
        chainId = contracts.chainId;
        eoaValidatorAddress = contracts.eoaValidator;
        webauthnValidatorAddress = contracts.webauthnValidator;
        // eslint-disable-next-line no-console
        console.log("Loaded factory address from contracts.json:", factoryAddress);
        // eslint-disable-next-line no-console
        console.log("Loaded EOA validator address from contracts.json:", eoaValidatorAddress);
        // eslint-disable-next-line no-console
        console.log("Loaded WebAuthn validator address from contracts.json:", webauthnValidatorAddress);
      } else {
        // eslint-disable-next-line no-console
        console.warn("contracts.json not found, using default factory address");
      }
    } catch (err: unknown) {
      // eslint-disable-next-line no-console
      console.warn("Failed to load contracts.json, using default factory address:", err);
    }

    // Add a rich Anvil wallet as an EOA signer for additional security
    // Using Anvil account #1 (0x70997970C51812dc3A010C7d01b50e0d17dc79C8)
    const eoaSignerAddress: Hex = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8";
    const eoaSignersAddresses: Hex[] = [eoaSignerAddress];

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

    // Create viem chain config
    const chain = {
      id: chainId,
      name: "Anvil",
      nativeCurrency: { name: "Ether", symbol: "ETH", decimals: 18 },
      rpcUrls: { default: { http: [rpcUrl] } },
    } satisfies Chain;

    // Create public client for RPC calls
    const publicClient = createPublicClient({
      chain,
      transport: http(),
    });

    // Prepare passkey signers if enabled
    let passkeySigners: Array<{ credentialId: Hex; publicKey: { x: Hex; y: Hex }; originDomain: string }> | undefined = undefined;
    if (passkeyConfig.value.enabled) {
      // eslint-disable-next-line no-console
      console.log("Including passkey signer in deployment...");

      if (!passkeyConfig.value.validatorAddress) {
        throw new Error("WebAuthn validator address is required when using passkeys");
      }

      passkeySigners = [{
        credentialId: passkeyConfig.value.credentialId as Hex,
        publicKey: {
          x: passkeyConfig.value.passkeyX as Hex,
          y: passkeyConfig.value.passkeyY as Hex,
        },
        originDomain: passkeyConfig.value.originDomain,
      }];

      // eslint-disable-next-line no-console
      console.log("  WebAuthn Validator:", passkeyConfig.value.validatorAddress);
    }

    // Get deployment transaction using new SDK
    // eslint-disable-next-line no-console
    console.log("  Creating deployment transaction...");

    // Log session validator installation if enabled
    if (sessionConfig.value.deployWithSession) {
      // eslint-disable-next-line no-console
      console.log("  Installing Session Validator during deployment...");
      // eslint-disable-next-line no-console
      console.log("  Session Validator:", sessionConfig.value.validatorAddress);
    }

    const { transaction, accountId } = await deploySmartAccount({
      contracts: {
        factory: factoryAddress,
        eoaValidator: eoaValidatorAddress,
        webauthnValidator: passkeyConfig.value.enabled ? passkeyConfig.value.validatorAddress as Address : undefined,
        sessionValidator: sessionConfig.value.deployWithSession ? sessionConfig.value.validatorAddress as Address : undefined,
      },
      eoaSigners: eoaSignersAddresses,
      passkeySigners: passkeySigners,
      installSessionValidator: sessionConfig.value.deployWithSession,
      userId, // Pass userId for deterministic accountId generation
    });

    // eslint-disable-next-line no-console
    console.log("  Account ID:", accountId);

    // Create wallet client for sending transactions
    const account = privateKeyToAccount(deployerPrivateKey);
    const walletClient = createWalletClient({
      account,
      chain,
      transport: http(),
    });

    // eslint-disable-next-line no-console
    console.log("  Sending deployment transaction...");
    const hash = await walletClient.sendTransaction({
      to: transaction.to,
      data: transaction.data,
      value: transaction.value,
    });

    // eslint-disable-next-line no-console
    console.log("  Transaction hash:", hash);

    // Wait for transaction receipt
    // eslint-disable-next-line no-console
    console.log("  Waiting for transaction confirmation...");
    const receipt = await publicClient.waitForTransactionReceipt({ hash });

    // eslint-disable-next-line no-console
    console.log("  Transaction confirmed in block:", receipt.blockNumber);

    // Extract deployed address from AccountCreated event using utility function
    const deployedAddress = getDeployedAccountAddress(receipt);

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

    // Build success message
    let successMessage = "Account deployed successfully with EOA signer";
    if (passkeyConfig.value.enabled && sessionConfig.value.deployWithSession) {
      successMessage += ", WebAuthn passkey, and Session validator! (Passkey automatically registered, session validator pre-installed)";
    } else if (passkeyConfig.value.enabled) {
      successMessage += " and WebAuthn passkey! (Passkey automatically registered)";
    } else if (sessionConfig.value.deployWithSession) {
      successMessage += " and Session validator! (Session validator pre-installed)";
    } else {
      successMessage += "!";
    }

    testResult.value = successMessage;

    // eslint-disable-next-line no-console
    console.log("Account deployment result:", deploymentResult.value);
  } catch (err: unknown) {
    // eslint-disable-next-line no-console
    console.error("Account deployment failed:", err);
    error.value = `Failed to deploy account: ${err instanceof Error ? err.message : String(err)}`;
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
    // Check if account is deployed
    if (!deploymentResult.value) {
      throw new Error("No deployed account found. Please deploy an account first.");
    }

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
    const credentialId = hexToBytes(passkeyConfig.value.credentialId as Hex);
    const passkeyX = hexToBytes(passkeyConfig.value.passkeyX as Hex);
    const passkeyY = hexToBytes(passkeyConfig.value.passkeyY as Hex);

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
  } catch (err: unknown) {
    // eslint-disable-next-line no-console
    console.error("Passkey registration failed:", err);
    passkeyRegisterError.value = `Failed to register passkey: ${err instanceof Error ? err.message : String(err)}`;
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

    if (!receipt) {
      throw new Error("Transaction receipt is null");
    }

    // eslint-disable-next-line no-console
    console.log("  Transaction confirmed in block:", receipt.blockNumber);

    fundResult.value = tx.hash;

    // Check the balance
    const provider = signer.provider;
    if (!provider) {
      throw new Error("Provider is not available");
    }

    if (!deploymentResult.value) {
      throw new Error("Deployment result is not available");
    }

    const balance = await provider.getBalance(deploymentResult.value.address);
    // eslint-disable-next-line no-console
    console.log("  Smart account balance:", ethers.formatEther(balance), "ETH");
  } catch (err: unknown) {
    // eslint-disable-next-line no-console
    console.error("Funding failed:", err);
    fundError.value = `Failed to fund smart account: ${err instanceof Error ? err.message : String(err)}`;
  } finally {
    loading.value = false;
  }
}

// Compute smart account address using offline CREATE2 computation
async function computeAddress() {
  loading.value = true;
  addressComputeError.value = "";
  computedAddress.value = "";

  try {
    // Validate inputs
    if (!isAddressParamsValid.value) {
      throw new Error("Please fill in all address parameters correctly");
    }

    // Use the WASM function to compute the account ID
    const accountId = compute_account_id(addressParams.value.userId);

    // eslint-disable-next-line no-console
    console.log("Computing address with parameters:");
    // eslint-disable-next-line no-console
    console.log("  Account ID:", accountId);
    // eslint-disable-next-line no-console
    console.log("  Deploy Wallet:", addressParams.value.deployWallet);
    // eslint-disable-next-line no-console
    console.log("  Factory:", addressParams.value.factory);
    // eslint-disable-next-line no-console
    console.log("  Bytecode Hash:", addressParams.value.bytecodeHash);
    // eslint-disable-next-line no-console
    console.log("  Proxy Address:", addressParams.value.proxyAddress);

    // TODO: Implement offline CREATE2 address computation
    // This would require implementing the same logic as the factory contract
    throw new Error("Address computation not yet implemented - use deployment to get address");
  } catch (err: unknown) {
    // eslint-disable-next-line no-console
    console.error("Address computation failed:", err);
    addressComputeError.value = `Failed to compute address: ${err instanceof Error ? err.message : String(err)}`;
  } finally {
    loading.value = false;
  }
}

// Check if SDK is available on mount
onMounted(async () => {
  // Only run on client side
  if (import.meta.client) {
    try {
      // SDK is already imported at the top
      sdkLoaded.value = true;

      // Load WebAuthn validator address from contracts.json
      await loadWebAuthnValidatorAddress();
    } catch (err: unknown) {
      // eslint-disable-next-line no-console
      console.error("Failed to load Web SDK:", err);
      error.value = `Failed to load Web SDK: ${err instanceof Error ? err.message : String(err)}`;
    }
  }
});
</script>
