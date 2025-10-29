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
  console.log("Sending transaction from smart account using Passkey...");

  // Import WASM functions
  const {
    prepare_passkey_user_operation_fixed_gas,
    submit_passkey_user_operation,
    SendTransactionConfig,
  } = await import("zksync-sso-web-sdk/bundler");

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

  // Verify the public key is registered on-chain
  // eslint-disable-next-line no-console
  console.log("Verifying public key registration on-chain...");
  const { ethers: ethersForValidation } = await import("ethers");
  const providerForValidation = new ethersForValidation.JsonRpcProvider(rpcUrl);

  // ABI for getAccountKey function
  const validatorAbi = [
    "function getAccountKey(string calldata originDomain, bytes calldata credentialId, address accountAddress) external view returns (bytes32[2] memory)",
  ];
  const validatorContract = new ethersForValidation.Contract(webauthnValidatorAddress, validatorAbi, providerForValidation);

  const credIdBytes = hexToBytes(props.passkeyConfig.credentialId);
  const registeredKey = await validatorContract.getAccountKey(
    props.passkeyConfig.originDomain,
    credIdBytes,
    props.deploymentResult.address,
  );

  // eslint-disable-next-line no-console
  console.log("  Registered public key on-chain:");
  // eslint-disable-next-line no-console
  console.log("    X:", registeredKey[0]);
  // eslint-disable-next-line no-console
  console.log("    Y:", registeredKey[1]);
  // eslint-disable-next-line no-console
  console.log("  Expected public key (from config):");
  // eslint-disable-next-line no-console
  console.log("    X:", props.passkeyConfig.passkeyX);
  // eslint-disable-next-line no-console
  console.log("    Y:", props.passkeyConfig.passkeyY);

  // Check if they match
  const registeredX = registeredKey[0].toLowerCase();
  const registeredY = registeredKey[1].toLowerCase();
  const expectedX = props.passkeyConfig.passkeyX.toLowerCase();
  const expectedY = props.passkeyConfig.passkeyY.toLowerCase();

  if (registeredX !== expectedX || registeredY !== expectedY) {
    throw new Error("Public key mismatch! The key registered on-chain doesn't match the config. This means you're trying to sign with a different passkey than the one that was registered.");
  }

  // eslint-disable-next-line no-console
  console.log("  ✓ Public key verification passed!");

  // Convert amount to wei (as string)
  const amountWei = (BigInt(parseFloat(amount.value) * 1e18)).toString();

  // Step 0: Build UserOperation to get hash
  // eslint-disable-next-line no-console
  console.log("Step 0: Building UserOperation to get hash...");

  // Import the SDK helper function (stub signature is created internally by Rust)
  const { signWithPasskey } = await import("zksync-sso-web-sdk/bundler");

  // Prepare UserOperation - stub signature is created internally
  const sendConfig = new SendTransactionConfig(
    rpcUrl,
    bundlerUrl,
    entryPointAddress,
  );

  // eslint-disable-next-line no-console
  console.log("  Calling prepare_fixed_gas (stub created internally)...");

  const prepareResult = await prepare_passkey_user_operation_fixed_gas(
    sendConfig,
    webauthnValidatorAddress,
    props.deploymentResult.address,
    to.value,
    amountWei,
    null, // data (null for simple transfer)
  );

  // eslint-disable-next-line no-console
  console.log("  Prepare result:", prepareResult);

  // Check if it's an error message
  if (prepareResult.startsWith("Failed to") || prepareResult.startsWith("Error")) {
    throw new Error(prepareResult);
  }

  // Parse the result JSON to get the hash
  const { hash, userOpId } = JSON.parse(prepareResult);

  // eslint-disable-next-line no-console
  console.log("  UserOp hash (from prepare):", hash);
  // eslint-disable-next-line no-console
  console.log("  UserOp ID:", userOpId);

  // Step 1: Sign the hash with passkey using SDK helper (replaces ~170 lines of manual encoding)
  // eslint-disable-next-line no-console
  console.log("Step 1: Requesting passkey signature for the real hash...");
  // eslint-disable-next-line no-console
  console.log("  Please touch your security key...");

  const { signature: signatureEncoded } = await signWithPasskey({
    hash,
    credentialId: props.passkeyConfig.credentialId,
    rpId: window.location.hostname,
    origin: window.location.origin,
  });

  // eslint-disable-next-line no-console
  console.log("  Passkey signature received and encoded");
  // eslint-disable-next-line no-console
  console.log("  ABI-encoded signature length:", signatureEncoded.length);

  // Optional: Log debug info (the SDK handles all the encoding internally)
  // eslint-disable-next-line no-console
  console.log("  Debug info:");
  // eslint-disable-next-line no-console
  console.log("    Credential ID:", props.passkeyConfig.credentialId);
  // eslint-disable-next-line no-console
  console.log("    Public key X:", props.passkeyConfig.passkeyX);
  // eslint-disable-next-line no-console
  console.log("    Public key Y:", props.passkeyConfig.passkeyY);
  // eslint-disable-next-line no-console
  console.log("    Origin:", props.passkeyConfig.originDomain);

  // Step 2: Submit the signed UserOperation
  // Note: The Rust submit function will prepend the validator address,
  // so we only pass the ABI-encoded WebAuthn signature (no validator prefix)
  // eslint-disable-next-line no-console
  console.log("Step 2: Submitting signed UserOperation...");
  // eslint-disable-next-line no-console
  console.log("  UserOp ID:", userOpId);

  // Create a new config for submit (the previous one was consumed by prepare)
  const submitConfig = new SendTransactionConfig(
    rpcUrl,
    bundlerUrl,
    entryPointAddress,
  );
  const result = await submit_passkey_user_operation(
    submitConfig,
    userOpId,
    signatureEncoded, // Pass ONLY the ABI-encoded WebAuthn signature (Rust will prepend validator)
  );

  // eslint-disable-next-line no-console
  console.log("  Transaction result:", result);

  txResult.value = result;
}
</script>
