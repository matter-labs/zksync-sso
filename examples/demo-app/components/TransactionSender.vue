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
        v-if="deploymentResult.passkeyEnabled || deploymentResult.sessionEnabled"
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
          <label
            v-if="deploymentResult.passkeyEnabled"
            class="flex items-center"
          >
            <input
              v-model="signingMethod"
              type="radio"
              value="passkey"
              class="mr-2"
            >
            <span class="text-sm">WebAuthn Passkey (Hardware Key)</span>
          </label>
          <label
            v-if="deploymentResult.sessionEnabled"
            class="flex items-center"
          >
            <input
              v-model="signingMethod"
              type="radio"
              value="session"
              class="mr-2"
            >
            <span class="text-sm">Session Key (Gasless within limits)</span>
          </label>
        </div>
      </div>

      <!-- Session Private Key Input (only shown for session method) -->
      <div v-if="signingMethod === 'session'">
        <label class="block text-sm font-medium mb-1">Session Private Key:</label>
        <input
          v-model="sessionPrivateKey"
          type="text"
          placeholder="0x..."
          class="w-full px-3 py-2 border rounded font-mono text-sm"
        >
        <p class="text-xs text-gray-500 mt-1">
          The private key for the session signer address
        </p>
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
        {{
          loading
            ? "Sending..."
            : signingMethod === 'passkey'
              ? 'Send with Passkey'
              : signingMethod === 'session'
                ? 'Send with Session Key'
                : 'Send with EOA'
        }}
      </button>
    </div>

    <!-- Transaction Result -->
    <div
      v-if="txResult"
      class="mt-4 p-3 bg-white rounded border border-indigo-300"
    >
      <strong class="text-sm">Send Transaction Hash:</strong>
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
import { ref, watch } from "vue";
import { hexToBytes } from "viem";

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
  sessionConfig: {
    type: Object,
    default: null,
  },
});

// Local state
const signingMethod = ref("eoa");
const to = ref("0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC"); // Anvil account #2
const amount = ref("0.001");
const sessionPrivateKey = ref(""); // Session private key input
const loading = ref(false);
const txResult = ref("");
const txError = ref("");

// Pre-fill session private key from config
watch(() => props.sessionConfig?.privateKey, (newPrivateKey) => {
  if (newPrivateKey && !sessionPrivateKey.value) {
    sessionPrivateKey.value = newPrivateKey;
  }
}, { immediate: true });

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
    } else if (signingMethod.value === "session") {
      await sendFromSmartAccountWithSession();
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

  // Import simplified SDK function
  const { sendTransactionWithPasskey } = await import("zksync-sso-web-sdk/bundler");

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

  // Send transaction using simplified SDK function (handles prepare → sign → submit)
  // eslint-disable-next-line no-console
  console.log("Sending transaction with passkey (SDK handles full flow)...");

  const result = await sendTransactionWithPasskey({
    rpcUrl,
    bundlerUrl,
    entryPointAddress,
    webauthnValidatorAddress,
    accountAddress: props.deploymentResult.address,
    toAddress: to.value,
    value: amountWei,
    data: null,
    credentialId: props.passkeyConfig.credentialId,
    rpId: window.location.hostname,
    origin: window.location.origin,
  });

  // eslint-disable-next-line no-console
  console.log("  Transaction result:", result);

  txResult.value = result;
}

// Send transaction using Session Key validator
async function sendFromSmartAccountWithSession() {
  if (!props.sessionConfig || !props.sessionConfig.enabled) {
    throw new Error("Session is not configured or enabled");
  }

  if (!sessionPrivateKey.value) {
    throw new Error("Session private key is required");
  }

  // eslint-disable-next-line no-console
  console.log("Sending transaction from smart account using Session Key...");

  // Import SDK function
  const { sendTransactionWithSession, SendTransactionConfig } = await import("zksync-sso-web-sdk/bundler");

  // Load contracts.json
  const response = await fetch("/contracts.json");
  const contracts = await response.json();
  const rpcUrl = contracts.rpcUrl;
  const bundlerUrl = contracts.bundlerUrl || "http://localhost:4337";
  const entryPointAddress = contracts.entryPoint || "0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108";
  const sessionValidatorAddress = contracts.sessionValidator;

  if (!sessionValidatorAddress) {
    throw new Error("Session validator address not found in contracts.json");
  }

  // eslint-disable-next-line no-console
  console.log("  Smart Account:", props.deploymentResult.address);
  // eslint-disable-next-line no-console
  console.log("  To:", to.value);
  // eslint-disable-next-line no-console
  console.log("  Amount:", amount.value, "ETH");
  // eslint-disable-next-line no-console
  console.log("  Session Validator:", sessionValidatorAddress);

  // Convert amount to wei (as string)
  const amountWei = (BigInt(parseFloat(amount.value) * 1e18)).toString();

  // Construct the SendTransactionConfig wasm object
  const sendConfig = new SendTransactionConfig(
    rpcUrl,
    bundlerUrl,
    entryPointAddress,
  );

  // Send transaction using session key
  // eslint-disable-next-line no-console
  console.log("Sending transaction with session key...");

  // Build SessionConfig in the shape expected by the SDK helper (expiresAt, feeLimit, valueLimit)
  const { ethers } = await import("ethers");
  const nowSec = Math.floor(Date.now() / 1000);
  const expiresAt = nowSec + (props.sessionConfig.expiresInDays ?? 1) * 86400;

  const session = {
    signer: props.sessionConfig.signer,
    expiresAt,
    feeLimit: {
      limitType: "lifetime",
      limit: ethers.parseEther(String(props.sessionConfig.feeLimitEth ?? "0.1")),
    },
    transfers: [
      {
        to: props.sessionConfig.transfers?.[0]?.to ?? to.value,
        valueLimit: ethers.parseEther(String(props.sessionConfig.transfers?.[0]?.valueLimitEth ?? amount.value)),
      },
    ],
  };

  const result = await sendTransactionWithSession({
    txConfig: sendConfig,
    sessionValidatorAddress,
    accountAddress: props.deploymentResult.address,
    to: to.value,
    value: amountWei,
    data: null,
    sessionPrivateKey: sessionPrivateKey.value,
    session,
  });

  // eslint-disable-next-line no-console
  console.log("  Transaction result:", result);

  txResult.value = result;
}
</script>
