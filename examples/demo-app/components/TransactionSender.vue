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

<script setup lang="ts">
import { ref } from "vue";
import { createPublicClient, formatEther, http, parseEther } from "viem";
import { createBundlerClient } from "viem/account-abstraction";
import type { Address, Chain } from "viem";
import { toPasskeySmartAccount } from "zksync-sso/client-new/passkey";
import { toEcdsaSmartAccount } from "zksync-sso/client-new/ecdsa";
import { WebAuthValidatorAbi } from "zksync-sso/abi";

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

// Main transaction handler
async function sendTransaction() {
  loading.value = true;
  txError.value = "";
  txResult.value = "";

  try {
    // Check balance and account deployment using viem
    // eslint-disable-next-line no-console
    console.log("Checking smart account status...");

    // Load contracts.json to get RPC URL
    const response = await fetch("/contracts.json");
    const contracts = await response.json();

    const chain = {
      id: contracts.chainId,
      name: "Anvil",
      nativeCurrency: { name: "Ether", symbol: "ETH", decimals: 18 },
      rpcUrls: { default: { http: [contracts.rpcUrl] } },
    } satisfies Chain;

    // Create public client
    const publicClient = createPublicClient({
      chain,
      transport: http(),
    });

    // Check if account is deployed
    const code = await publicClient.getCode({
      address: props.deploymentResult.address as Address,
    });
    if (!code || code === "0x") {
      throw new Error("Smart account is not deployed on-chain yet. The deployment may have failed. Please check the console for deployment errors and try deploying again.");
    }
    // eslint-disable-next-line no-console
    console.log("  Account is deployed (has code)");

    // Check balance
    const balance = await publicClient.getBalance({
      address: props.deploymentResult.address as Address,
    });
    const balanceEth = formatEther(balance);

    // eslint-disable-next-line no-console
    console.log("  Smart account balance:", balanceEth, "ETH");

    // Check if balance is zero
    if (balance === 0n) {
      throw new Error("Smart account has zero balance. Please fund the account first using the 'Fund Smart Account' button above.");
    }

    // Check if balance is too low (less than 0.001 ETH)
    const minBalance = parseEther("0.001");
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
    txError.value = `Failed to send transaction: ${(err as Error).message}`;
  } finally {
    loading.value = false;
  }
}

// Send transaction using EOA validator (NEW SDK)
async function sendFromSmartAccountWithEOA() {
  // Load contracts.json
  const response = await fetch("/contracts.json");
  const contracts = await response.json();

  const chain = {
    id: contracts.chainId,
    name: "Anvil",
    nativeCurrency: { name: "Ether", symbol: "ETH", decimals: 18 },
    rpcUrls: { default: { http: [contracts.rpcUrl] } },
  } satisfies Chain;

  // Create public client for network calls
  const publicClient = createPublicClient({
    chain,
    transport: http(),
  });

  // Create smart account using the new SDK
  // eslint-disable-next-line no-console
  console.log("  Creating smart account instance with toEcdsaSmartAccount...");
  const account = await toEcdsaSmartAccount({
    client: publicClient,
    // EOA signer private key (Anvil account #1) - this will sign the UserOperation
    signerPrivateKey: "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d",
    address: props.deploymentResult.address,
    eoaValidatorAddress: contracts.eoaValidator,
  });

  // Create bundler client with entry point configuration
  const bundlerClient = createBundlerClient({
    client: publicClient,
    chain,
    transport: http(contracts.bundlerUrl || "http://localhost:4337"),
  });

  // Send user operation
  // eslint-disable-next-line no-console
  console.log("  Sending UserOperation...");
  const userOpHash = await bundlerClient.sendUserOperation({
    account,
    calls: [{
      to: to.value as Address,
      value: parseEther(amount.value),
      data: "0x",
    }],
  });

  // eslint-disable-next-line no-console
  console.log("  UserOperation hash:", userOpHash);

  // Wait for receipt
  // eslint-disable-next-line no-console
  console.log("  Waiting for UserOperation receipt...");
  const receipt = await bundlerClient.waitForUserOperationReceipt({
    hash: userOpHash,
  });

  // eslint-disable-next-line no-console
  console.log("  UserOperation confirmed! Receipt:", receipt);

  txResult.value = `Transaction successful! UserOp hash: ${userOpHash}`;
}

// Send transaction using Passkey validator (NEW SDK)
async function sendFromSmartAccountWithPasskey() {
  // eslint-disable-next-line no-console
  console.log("Sending transaction from smart account using Passkey (NEW SDK)...");

  // Load contracts.json
  const response = await fetch("/contracts.json");
  const contracts = await response.json();
  const webauthnValidatorAddress = props.passkeyConfig.validatorAddress;

  if (!webauthnValidatorAddress) {
    throw new Error("WebAuthn validator address not found");
  }

  const chain = {
    id: contracts.chainId,
    name: "Anvil",
    nativeCurrency: { name: "Ether", symbol: "ETH", decimals: 18 },
    rpcUrls: { default: { http: [contracts.rpcUrl] } },
  } satisfies Chain;

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

  // Create public client for network calls
  const publicClient = createPublicClient({
    chain,
    transport: http(),
  });

  const registeredKey = await publicClient.readContract({
    address: webauthnValidatorAddress as Address,
    abi: WebAuthValidatorAbi,
    functionName: "getAccountKey",
    args: [
      props.passkeyConfig.originDomain,
      props.passkeyConfig.credentialId,
      props.deploymentResult.address as Address,
    ],
  });

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

  // Create smart account using the new SDK
  // eslint-disable-next-line no-console
  console.log("  Creating passkey smart account instance with toPasskeySmartAccount...");
  const account = await toPasskeySmartAccount({
    client: publicClient,
    address: props.deploymentResult.address,
    validatorAddress: webauthnValidatorAddress,
    credentialId: props.passkeyConfig.credentialId,
    rpId: window.location.hostname,
    origin: window.location.origin,
  });

  // Create bundler client with entry point configuration
  const bundlerClient = createBundlerClient({
    client: publicClient,
    chain,
    transport: http(contracts.bundlerUrl || "http://localhost:4337"),
    userOperation: {
      // Use fixed gas values matching old Rust SDK implementation
      // (old SDK used: 2M callGas, 2M verificationGas, 1M preVerificationGas)
      async estimateFeesPerGas() {
        const feesPerGas = await publicClient.estimateFeesPerGas();
        return {
          callGasLimit: 2_000_000n,
          verificationGasLimit: 2_000_000n,
          preVerificationGas: 1_000_000n,
          ...feesPerGas,
        };
      },
    },
  });

  // Send user operation
  // eslint-disable-next-line no-console
  console.log("  Sending UserOperation...");
  const userOpHash = await bundlerClient.sendUserOperation({
    account,
    calls: [{
      to: to.value as Address,
      value: parseEther(amount.value),
      data: "0x",
    }],
  });

  // eslint-disable-next-line no-console
  console.log("  UserOperation hash:", userOpHash);

  // Wait for receipt
  // eslint-disable-next-line no-console
  console.log("  Waiting for UserOperation receipt...");
  const receipt = await bundlerClient.waitForUserOperationReceipt({
    hash: userOpHash,
  });

  // eslint-disable-next-line no-console
  console.log("  UserOperation confirmed! Receipt:", receipt);

  txResult.value = `Transaction successful! UserOp hash: ${userOpHash}`;
}
</script>
