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
      </div>
    </div>

    <!-- Fund Smart Account -->
    <div
      v-if="deploymentResult"
      class="bg-orange-50 p-4 rounded-lg mb-4 border border-orange-200"
    >
      <h2 class="text-lg font-semibold mb-3 text-orange-800">
        Step 1: Fund Smart Account
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
    <div
      v-if="deploymentResult && fundResult"
      class="bg-indigo-50 p-4 rounded-lg mb-4 border border-indigo-200"
    >
      <h2 class="text-lg font-semibold mb-3 text-indigo-800">
        Step 2: Send Transaction from Smart Account
      </h2>
      <p class="text-sm text-gray-600 mb-4">
        Use the smart account's EOA validator to send a transaction. The EOA wallet will sign on behalf of the smart account.
      </p>

      <div class="space-y-3">
        <div>
          <label class="block text-sm font-medium mb-1">Recipient Address:</label>
          <input
            v-model="txParams.to"
            type="text"
            placeholder="0x..."
            class="w-full px-3 py-2 border border-gray-300 rounded text-sm font-mono"
          >
        </div>

        <div>
          <label class="block text-sm font-medium mb-1">Amount (ETH):</label>
          <input
            v-model="txParams.amount"
            type="text"
            placeholder="0.001"
            class="w-full px-3 py-2 border border-gray-300 rounded text-sm"
          >
        </div>

        <button
          :disabled="loading || !txParams.to || !txParams.amount"
          class="w-full px-4 py-2 bg-indigo-500 text-white rounded hover:bg-indigo-600 disabled:opacity-50"
          @click="sendFromSmartAccount"
        >
          {{ loading ? 'Sending...' : 'Send from Smart Account' }}
        </button>
      </div>

      <!-- Transaction Result -->
      <div
        v-if="txResult"
        class="mt-4 p-3 bg-white rounded border border-indigo-300"
      >
        <strong class="text-sm">Transaction Hash:</strong>
        <code class="block mt-1 px-2 py-1 bg-gray-100 rounded text-xs font-mono break-all">
          {{ txResult }}
        </code>
      </div>

      <div
        v-if="txError"
        class="mt-4 p-3 bg-red-50 rounded border border-red-300"
      >
        <strong class="text-sm text-red-800">Error:</strong>
        <p class="text-xs text-red-600 mt-1">
          {{ txError }}
        </p>
      </div>
    </div>

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

// Fund smart account parameters
const fundParams = ref({
  amount: "0.1",
});
const fundResult = ref("");
const fundError = ref("");

// Transaction parameters
const txParams = ref({
  to: "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC", // Anvil account #2
  amount: "0.001",
});
const txResult = ref("");
const txError = ref("");

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
    const dummyPrivateKey = "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
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

// Deploy a new smart account
async function deployAccount() {
  loading.value = true;
  error.value = "";
  deploymentResult.value = null;

  try {
    // Import the web SDK - destructure the WASM functions we need
    const { deploy_account, compute_account_id } = await import("zksync-sso-web-sdk/bundler");

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

    // Use the appropriate private key based on the network
    // Standard Anvil (port 8545): 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
    const deployerPrivateKey = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"; // Anvil default account #0

    // eslint-disable-next-line no-console
    console.log("Deploying account...");
    // eslint-disable-next-line no-console
    console.log("  RPC URL:", rpcUrl);
    // eslint-disable-next-line no-console
    console.log("  Factory:", factoryAddress);
    // eslint-disable-next-line no-console
    console.log("  User ID:", userId);

    // Call the deployment function
    const deployedAddress = await deploy_account(
      rpcUrl,
      factoryAddress,
      userId,
      deployerPrivateKey,
      eoaSignersAddresses,
      eoaValidatorAddress,
    );

    // eslint-disable-next-line no-console
    console.log("Account deployed at:", deployedAddress);

    // Display the deployment result
    deploymentResult.value = {
      userId,
      accountId,
      address: deployedAddress,
      eoaSigner: eoaSignerAddress,
    };
    testResult.value = "Account deployed successfully with EOA signer!";

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

// Fund the smart account with ETH from EOA wallet
async function fundSmartAccount() {
  loading.value = true;
  fundError.value = "";
  fundResult.value = "";

  try {
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
    console.log("  To (Smart Account):", deploymentResult.value.address);
    // eslint-disable-next-line no-console
    console.log("  Amount:", fundParams.value.amount, "ETH");

    // Create provider
    const provider = new ethers.JsonRpcProvider(rpcUrl);

    // EOA signer private key (Anvil account #1)
    const eoaSignerPrivateKey = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
    const eoaSigner = new ethers.Wallet(eoaSignerPrivateKey, provider);

    // Convert amount to wei
    const amountWei = ethers.parseEther(fundParams.value.amount);

    // Send transaction to fund the smart account
    const tx = await eoaSigner.sendTransaction({
      to: deploymentResult.value.address,
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

// Send transaction from smart account using EOA validator
async function sendFromSmartAccount() {
  loading.value = true;
  txError.value = "";
  txResult.value = "";

  try {
    // Import ethers and alloy for contract interaction
    const { ethers } = await import("ethers");

    // Load contracts.json
    const response = await fetch("/contracts.json");
    const contracts = await response.json();
    const rpcUrl = contracts.rpcUrl;

    // eslint-disable-next-line no-console
    console.log("Sending transaction from smart account...");
    // eslint-disable-next-line no-console
    console.log("  From (Smart Account):", deploymentResult.value.address);
    // eslint-disable-next-line no-console
    console.log("  To:", txParams.value.to);
    // eslint-disable-next-line no-console
    console.log("  Amount:", txParams.value.amount, "ETH");

    // Create provider
    const provider = new ethers.JsonRpcProvider(rpcUrl);

    // EOA signer private key (Anvil account #1) - this will sign on behalf of the smart account
    const eoaSignerPrivateKey = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
    const eoaSigner = new ethers.Wallet(eoaSignerPrivateKey, provider);

    // eslint-disable-next-line no-console
    console.log("  EOA Signer:", eoaSigner.address);

    // Convert amount to wei
    const amountWei = ethers.parseEther(txParams.value.amount);

    // Get the ModularSmartAccount ABI for execute function (ERC7579 standard)
    // The execute function takes a mode (bytes32) and executionCalldata (bytes)
    // Mode encoding: [callType (1 byte) | execType (1 byte) | unused (4 bytes) | selector (4 bytes) | payload (22 bytes)]
    // CallType: 0x00 = SINGLE, 0x01 = BATCH
    // ExecType: 0x00 = DEFAULT (revert on failure), 0x01 = TRY (continue on failure)

    const accountAbi = [
      "function execute(bytes32 mode, bytes calldata executionCalldata) external payable",
    ];

    // Create contract instance
    const smartAccount = new ethers.Contract(
      deploymentResult.value.address,
      accountAbi,
      eoaSigner,
    );

    // eslint-disable-next-line no-console
    console.log("  Calling execute function on smart account...");

    // Encode the mode for SINGLE execution with DEFAULT exec type
    // CALLTYPE_SINGLE = 0x00, EXECTYPE_DEFAULT = 0x00
    const mode = "0x0000000000000000000000000000000000000000000000000000000000000000";

    // Encode the execution data: (target, value, callData)
    // For a simple ETH transfer, callData is empty (0x)
    const executionCalldata = ethers.AbiCoder.defaultAbiCoder().encode(
      ["address", "uint256", "bytes"],
      [txParams.value.to, amountWei, "0x"],
    );

    // Call execute function
    const tx = await smartAccount.execute(mode, executionCalldata);

    // eslint-disable-next-line no-console
    console.log("  Transaction sent:", tx.hash);

    // Wait for confirmation
    const receipt = await tx.wait();

    // eslint-disable-next-line no-console
    console.log("  Transaction confirmed in block:", receipt.blockNumber);

    txResult.value = tx.hash;
  } catch (err) {
    // eslint-disable-next-line no-console
    console.error("Transaction failed:", err);
    txError.value = `Failed to send transaction: ${err.message}`;
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
    } catch (err) {
      // eslint-disable-next-line no-console
      console.error("Failed to load Web SDK:", err);
      error.value = `Failed to load Web SDK: ${err.message}`;
    }
  }
});
</script>
