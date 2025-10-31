<template>
  <div class="bg-indigo-50 p-4 rounded-lg mb-4 border border-indigo-200">
    <h2 class="text-lg font-semibold mb-3 text-indigo-800">
      {{ deploymentResult.passkeyEnabled ? 'Step 3: Send Transaction from Smart Account' : 'Step 2: Send Transaction from Smart Account' }}
    </h2>
    <p class="text-sm text-gray-600 mb-4">
      Send a transaction from your smart account using either EOA or Passkey signing.
    </p>

    <div class="space-y-3">
      <!-- Signing Method Selection -->
      <div
        v-if="deploymentResult.passkeyEnabled"
        class="mb-3 p-3 bg-white rounded border border-indigo-300"
      >
        <label class="block text-sm font-medium mb-2">Signing Method:</label>
        <div class="space-y-2">
          <label class="flex items-center">
            <input
              v-model="signingMethod"
              type="radio"
              value="eoa"
              class="mr-2"
            >
            <span class="text-sm">EOA Validator (Private Key)</span>
          </label>
          <label class="flex items-center">
            <input
              v-model="signingMethod"
              type="radio"
              value="passkey"
              class="mr-2"
            >
            <span class="text-sm">WebAuthn Passkey (Hardware Key)</span>
          </label>
        </div>
      </div>

      <!-- Transaction Parameters -->
      <div>
        <label class="block text-sm font-medium mb-1">Recipient Address:</label>
        <input
          v-model="to"
          type="text"
          placeholder="0x..."
          class="w-full px-3 py-2 border rounded"
        >
      </div>

      <div>
        <label class="block text-sm font-medium mb-1">Amount (ETH):</label>
        <input
          v-model="amount"
          type="text"
          placeholder="0.001"
          class="w-full px-3 py-2 border rounded"
        >
      </div>

      <!-- Send Button -->
      <button
        :disabled="loading || !deploymentResult"
        class="w-full px-4 py-2 bg-indigo-600 text-white rounded hover:bg-indigo-700 disabled:bg-gray-400 disabled:cursor-not-allowed"
        @click="sendTransaction"
      >
        {{ loading ? "Sending..." : (signingMethod === 'passkey' ? 'Send with Passkey' : 'Send with EOA') }}
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
</template>

<script setup>
import { ref } from "vue";

// Props
const props = defineProps({
  deploymentResult: {
    type: Object,
    required: true,
  },
  passkeyConfig: {
    type: Object,
    required: true,
  },
});

// Local state
const signingMethod = ref("eoa");
const to = ref("0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC"); // Anvil account #2
const amount = ref("0.001");
const loading = ref(false);
const txResult = ref("");
const txError = ref("");

/**
 * Convert a hex string to a Uint8Array of bytes
 */
function _hexToBytes(hex) {
  // Remove 0x prefix if present
  const cleanHex = hex.startsWith("0x") ? hex.slice(2) : hex;

  // Validate hex string
  if (cleanHex.length % 2 !== 0) {
    throw new Error(`Invalid hex string length: ${cleanHex.length}`);
  }

  const bytes = new Uint8Array(cleanHex.length / 2);
  for (let i = 0; i < cleanHex.length; i += 2) {
    bytes[i / 2] = parseInt(cleanHex.slice(i, i + 2), 16);
  }

  return bytes;
}

// Main transaction handler
async function sendTransaction() {
  loading.value = true;
  txError.value = "";
  txResult.value = "";

  try {
    // Import ethers to check balance and account deployment
    // eslint-disable-next-line no-console
    console.log("Checking smart account status...");

    // Import ethers to check balance
    const { ethers } = await import("ethers");

    // Load contracts.json to get RPC URL
    const response = await fetch("/contracts.json");
    const contracts = await response.json();
    const rpcUrl = contracts.rpcUrl;

    // Create provider
    const provider = new ethers.JsonRpcProvider(rpcUrl);

    // Check if account is deployed
    const code = await provider.getCode(props.deploymentResult.address);
    if (code === "0x") {
      throw new Error("Smart account is not deployed on-chain yet. The deployment may have failed. Please check the console for deployment errors and try deploying again.");
    }
    // eslint-disable-next-line no-console
    console.log("  Account is deployed (has code)");

    // Check balance
    const balance = await provider.getBalance(props.deploymentResult.address);
    const balanceEth = ethers.formatEther(balance);

    // eslint-disable-next-line no-console
    console.log("  Smart account balance:", balanceEth, "ETH");

    // Check if balance is zero
    if (balance === 0n) {
      throw new Error("Smart account has zero balance. Please fund the account first using the 'Fund Smart Account' button above.");
    }

    // Check if balance is too low (less than 0.001 ETH)
    const minBalance = ethers.parseEther("0.001");
    if (balance < minBalance) {
      throw new Error(`Smart account balance is too low: ${balanceEth} ETH. Please fund the account with at least 0.001 ETH for gas fees.`);
    }

    // eslint-disable-next-line no-console
    console.log("  Balance check passed, proceeding with transaction...");

    if (signingMethod.value === "passkey") {
      await sendFromSmartAccountWithPasskey();
    } else {
      await sendFromSmartAccountWithEOA();
    }
  } catch (err) {
    // eslint-disable-next-line no-console
    console.error("Transaction failed:", err);
    txError.value = `Failed to send transaction: ${err.message}`;
  } finally {
    loading.value = false;
  }
}

// Send transaction using EOA validator
async function sendFromSmartAccountWithEOA() {
  // Import the WASM function and SendTransactionConfig
  const { send_transaction_eoa, SendTransactionConfig } = await import("zksync-sso-web-sdk/bundler");

  // Load contracts.json
  const response = await fetch("/contracts.json");
  const contracts = await response.json();
  const rpcUrl = contracts.rpcUrl;
  const bundlerUrl = contracts.bundlerUrl || "http://localhost:4337"; // Default bundler URL
  const entryPointAddress = contracts.entryPoint || "0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108";
  const eoaValidatorAddress = contracts.eoaValidator;

  // EOA signer private key (Anvil account #1) - this will sign the UserOperation
  const eoaSignerPrivateKey = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";

  // eslint-disable-next-line no-console
  console.log("Sending transaction from smart account using ERC-4337 (EOA)...");
  // eslint-disable-next-line no-console
  console.log("  Smart Account:", props.deploymentResult.address);
  // eslint-disable-next-line no-console
  console.log("  To:", to.value);
  // eslint-disable-next-line no-console
  console.log("  Amount:", amount.value, "ETH");
  // eslint-disable-next-line no-console
  console.log("  Bundler URL:", bundlerUrl);
  // eslint-disable-next-line no-console
  console.log("  EntryPoint:", entryPointAddress);
  // eslint-disable-next-line no-console
  console.log("  EOA Validator:", eoaValidatorAddress);

  // Convert amount to wei (as string)
  const amountWei = (BigInt(parseFloat(amount.value) * 1e18)).toString();

  // Construct the SendTransactionConfig wasm object
  const sendConfig = new SendTransactionConfig(
    rpcUrl,
    bundlerUrl,
    entryPointAddress,
  );

  // Call the WASM function to send transaction via ERC-4337
  const result = await send_transaction_eoa(
    sendConfig,
    eoaValidatorAddress,
    eoaSignerPrivateKey,
    props.deploymentResult.address, // account address
    to.value, // recipient
    amountWei, // value as string
    null, // data (null for simple transfer)
  );

  // eslint-disable-next-line no-console
  console.log("  UserOperation result:", result);

  txResult.value = result;
}

// Send transaction using Passkey validator
async function sendFromSmartAccountWithPasskey() {
  // eslint-disable-next-line no-console
  console.log("Sending transaction from smart account using Passkey (pure JS implementation)...");

  // Import helpers
  const { signWithPasskey, createStubSignature } = await import("zksync-sso-web-sdk/bundler");
  const { ethers } = await import("ethers");

  // Load contracts.json
  const response = await fetch("/contracts.json");
  const contracts = await response.json();
  const rpcUrl = contracts.rpcUrl;
  const bundlerUrl = contracts.bundlerUrl || "http://localhost:4337";
  const entryPointAddress = contracts.entryPoint || "0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108";
  const webauthnValidatorAddress = props.passkeyConfig.validatorAddress;

  if (!webauthnValidatorAddress) {
    throw new Error("WebAuthn validator address not found");
  }

  // eslint-disable-next-line no-console
  console.log("  Smart Account:", props.deploymentResult.address);
  // eslint-disable-next-line no-console
  console.log("  To:", to.value);
  // eslint-disable-next-line no-console
  console.log("  Amount:", amount.value, "ETH");
  // eslint-disable-next-line no-console
  console.log("  WebAuthn Validator:", webauthnValidatorAddress);

  // Create provider
  const provider = new ethers.JsonRpcProvider(rpcUrl);

  // Convert amount to wei
  const amountWei = ethers.parseEther(amount.value);

  // Step 1: Build execution call data
  // eslint-disable-next-line no-console
  console.log("Step 1: Building execution call data...");

  // Using MSA's execute function: execute(uint8 mode, bytes calldata executionCalldata)
  // mode 0x00 = single execution
  // executionCalldata = abi.encode(target, value, data)
  const executionCalldata = ethers.AbiCoder.defaultAbiCoder().encode(
    ["address", "uint256", "bytes"],
    [to.value, amountWei, "0x"],
  );

  const accountInterface = new ethers.Interface([
    "function execute(uint8 mode, bytes calldata executionCalldata)",
  ]);
  const callData = accountInterface.encodeFunctionData("execute", [0x00, executionCalldata]);

  // eslint-disable-next-line no-console
  console.log("  Call data encoded");

  // Step 2: Get nonce from entry point
  // eslint-disable-next-line no-console
  console.log("Step 2: Getting nonce from entry point...");

  const entryPointInterface = new ethers.Interface([
    "function getNonce(address sender, uint192 key) view returns (uint256)",
  ]);
  const entryPointContract = new ethers.Contract(entryPointAddress, entryPointInterface, provider);
  const nonce = await entryPointContract.getNonce(props.deploymentResult.address, 0);

  // eslint-disable-next-line no-console
  console.log("  Nonce:", nonce.toString());

  // Step 3: Create stub signature for gas estimation
  // eslint-disable-next-line no-console
  console.log("Step 3: Creating stub signature...");

  const stubSignature = await createStubSignature(webauthnValidatorAddress);

  // Step 4: Build UserOperation with fixed gas
  // eslint-disable-next-line no-console
  console.log("Step 4: Building UserOperation...");

  const userOp = {
    sender: props.deploymentResult.address,
    nonce: "0x" + nonce.toString(16),
    callData,
    initCode: "0x",
    // Fixed high gas values
    callGasLimit: "0x" + (2000000).toString(16),
    verificationGasLimit: "0x" + (2000000).toString(16),
    preVerificationGas: "0x" + (2000000).toString(16),
    maxFeePerGas: "0x" + (2000000).toString(16),
    maxPriorityFeePerGas: "0x" + (2000000).toString(16),
    signature: stubSignature,
    paymasterAndData: "0x",
  };

  // eslint-disable-next-line no-console
  console.log("  UserOperation built");

  // Step 5: Get UserOp hash
  // eslint-disable-next-line no-console
  console.log("Step 5: Getting UserOp hash...");

  // Pack UserOp for hashing
  const packedUserOp = ethers.AbiCoder.defaultAbiCoder().encode(
    ["address", "uint256", "bytes32", "bytes32", "bytes32", "uint256", "bytes32", "bytes32"],
    [
      userOp.sender,
      userOp.nonce,
      ethers.keccak256(userOp.initCode),
      ethers.keccak256(userOp.callData),
      ethers.solidityPacked(
        ["uint128", "uint128"],
        [userOp.verificationGasLimit, userOp.callGasLimit],
      ),
      userOp.preVerificationGas,
      ethers.solidityPacked(
        ["uint128", "uint128"],
        [userOp.maxPriorityFeePerGas, userOp.maxFeePerGas],
      ),
      ethers.keccak256(userOp.paymasterAndData),
    ],
  );

  const packedHash = ethers.keccak256(packedUserOp);
  const chainId = await provider.getNetwork().then((n) => n.chainId);

  const userOpHash = ethers.keccak256(
    ethers.AbiCoder.defaultAbiCoder().encode(
      ["bytes32", "address", "uint256"],
      [packedHash, entryPointAddress, chainId],
    ),
  );

  // eslint-disable-next-line no-console
  console.log("  UserOp hash:", userOpHash);

  // Step 6: Sign with passkey
  // eslint-disable-next-line no-console
  console.log("Step 6: Requesting passkey signature...");
  // eslint-disable-next-line no-console
  console.log("  Please touch your security key...");

  const { signature: signatureEncoded } = await signWithPasskey({
    hash: userOpHash,
    credentialId: props.passkeyConfig.credentialId,
    rpId: window.location.hostname,
    origin: window.location.origin,
  });

  // eslint-disable-next-line no-console
  console.log("  Passkey signature received");

  // Update UserOp with real signature
  userOp.signature = signatureEncoded;

  // Step 7: Submit to bundler
  // eslint-disable-next-line no-console
  console.log("Step 7: Submitting to bundler...");

  const bundlerResponse = await fetch(bundlerUrl, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "eth_sendUserOperation",
      params: [userOp, entryPointAddress],
    }),
  });

  const bundlerResult = await bundlerResponse.json();

  // eslint-disable-next-line no-console
  console.log("  Bundler response:", bundlerResult);

  if (bundlerResult.error) {
    throw new Error(`Bundler error: ${bundlerResult.error.message}`);
  }

  const userOpHashResult = bundlerResult.result;

  // eslint-disable-next-line no-console
  console.log("  UserOp submitted, hash:", userOpHashResult);

  // Step 8: Wait for receipt
  // eslint-disable-next-line no-console
  console.log("Step 8: Waiting for receipt...");

  let receipt = null;
  for (let i = 0; i < 30; i++) {
    await new Promise((resolve) => setTimeout(resolve, 2000));

    const receiptResponse = await fetch(bundlerUrl, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        jsonrpc: "2.0",
        id: 1,
        method: "eth_getUserOperationReceipt",
        params: [userOpHashResult],
      }),
    });

    const receiptResult = await receiptResponse.json();
    if (receiptResult.result) {
      receipt = receiptResult.result;
      break;
    }
  }

  if (!receipt) {
    throw new Error("Transaction receipt not found after 60 seconds");
  }

  // eslint-disable-next-line no-console
  console.log("  Receipt:", receipt);

  if (!receipt.success) {
    throw new Error("Transaction failed");
  }

  // eslint-disable-next-line no-console
  console.log("✓ Transaction successful!");

  txResult.value = `Transaction successful! Hash: ${receipt.receipt.transactionHash}`;
}
</script>
