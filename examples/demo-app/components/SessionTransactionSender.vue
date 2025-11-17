<template>
  <div class="bg-teal-50 p-4 rounded-lg mb-4 border border-teal-200">
    <h2 class="text-lg font-semibold mb-3 text-teal-800">
      Step 3: Send Session Transaction
    </h2>
    <p class="text-sm text-gray-600 mb-4">
      Send a transaction using the session key (no EOA or Passkey required).
    </p>

    <div class="space-y-3">
      <!-- Transaction Parameters -->
      <div>
        <label class="block text-sm font-medium mb-1">Recipient Address:</label>
        <input
          v-model="target"
          type="text"
          placeholder="0x..."
          class="w-full px-3 py-2 border rounded"
        >
      </div>

      <div>
        <label class="block text-sm font-medium mb-1">Amount (ETH):</label>
        <input
          v-model="value"
          type="text"
          placeholder="0.001"
          class="w-full px-3 py-2 border rounded"
        >
      </div>

      <!-- Send Button -->
      <button
        :disabled="loading"
        class="w-full px-4 py-2 bg-teal-600 text-white rounded hover:bg-teal-700 disabled:bg-gray-400 disabled:cursor-not-allowed"
        @click="sendTransaction"
      >
        {{ loading ? "Sending..." : "Send Session Transaction" }}
      </button>
    </div>

    <!-- Transaction Result -->
    <div
      v-if="result"
      class="mt-4 p-3 bg-green-50 border border-green-200 rounded"
    >
      <p class="font-medium">
        Success!
      </p>
      <div class="mt-2 space-y-1">
        <div>
          <strong class="text-sm">UserOp Hash:</strong>
          <code class="block mt-1 px-2 py-1 bg-white rounded text-xs font-mono break-all">
            {{ result.userOpHash }}
          </code>
        </div>
      </div>
    </div>

    <div
      v-if="error"
      class="mt-4 p-3 bg-red-50 border border-red-200 rounded"
    >
      <p class="font-medium text-red-600">
        Error
      </p>
      <p class="text-sm mt-1">
        {{ error }}
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { createPublicClient, http, parseEther, type Chain, type Address, type Abi } from "viem";
import { createBundlerClient } from "viem/account-abstraction";
import { createSessionClient, LimitType } from "zksync-sso-4337/client";
// WASM helpers for diagnostics and nonce calculation
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore
import { keyed_nonce_decimal } from "zksync-sso-web-sdk/bundler";

interface SessionConfig {
  enabled: boolean;
  validatorAddress: string;
  sessionPrivateKey: string;
  sessionSigner: string;
  expiresAt: number;
  feeLimit: string;
  allowedRecipient?: string;
}

// Props
const props = defineProps<{
  accountAddress: string;
  sessionConfig: SessionConfig;
}>();

// Local state
// Default recipient is shared across SessionCreator & web-sdk-test page to keep sessionSpec stable.
const DEFAULT_RECIPIENT = "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC";
const target = ref(props.sessionConfig.allowedRecipient || DEFAULT_RECIPIENT); // default or allowed recipient
const value = ref("0.001");
const loading = ref(false);
const result = ref<{ userOpHash: string } | null>(null);
const error = ref("");
const sessionStatus = computed(() => props.sessionConfig && props.sessionConfig.sessionSigner ? "Active" : "NotInitialized");

async function sendTransaction() {
  loading.value = true;
  error.value = "";
  result.value = null;

  try {
    // eslint-disable-next-line no-console
    console.log("Sending session transaction...");
    // Check session status before sending
    if (sessionStatus.value !== "Active") {
      throw new Error(`Session status is not Active. Current status: ${sessionStatus.value}`);
    }

    // Load contracts
    const response = await fetch("/contracts.json");
    const contracts = await response.json();

    const chain = {
      id: contracts.chainId,
      name: "Anvil",
      nativeCurrency: { name: "Ether", symbol: "ETH", decimals: 18 },
      rpcUrls: { default: { http: [contracts.rpcUrl] } },
    } satisfies Chain;

    // Create clients
    const publicClient = createPublicClient({
      chain,
      transport: http(),
    });

    const bundlerClient = createBundlerClient({
      transport: http(contracts.bundlerUrl || "http://localhost:4337"),
      chain,
      // Provide execution client so fees & non-bundler RPCs are fetched from the node, not the bundler.
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      client: publicClient as any,
      userOperation: {
        async estimateFeesPerGas() {
          const feesPerGas = await publicClient.estimateFeesPerGas();
          return {
            callGasLimit: 2_000_000n,
            verificationGasLimit: 2_000_000n,
            preVerificationGas: 1_000_000n,
            ...feesPerGas,
          } as const;
        },
      },
    });

    // Create session client (wraps session smart account)
    const sessionClient = createSessionClient({
      address: props.accountAddress as Address,
      sessionValidatorAddress: props.sessionConfig.validatorAddress as Address,
      sessionKeyPrivateKey: props.sessionConfig.sessionPrivateKey as `0x${string}`,
      sessionSpec: {
        signer: props.sessionConfig.sessionSigner as Address,
        expiresAt: BigInt(props.sessionConfig.expiresAt),
        feeLimit: {
          limitType: LimitType.Lifetime,
          limit: parseEther("1"),
          period: 0n,
        },
        callPolicies: [],
        transferPolicies: [
          {
            target: (props.sessionConfig.allowedRecipient as Address) || (DEFAULT_RECIPIENT as Address),
            maxValuePerUse: parseEther("0.001"),
            valueLimit: {
              limitType: LimitType.Unlimited,
              limit: 0n,
              period: 0n,
            },
          },
        ],
      },
      bundlerClient,
      chain,
      transport: http(),
    });

    // If an allowed recipient is configured, enforce it matches the selected target
    if (props.sessionConfig.allowedRecipient && props.sessionConfig.allowedRecipient.toLowerCase() !== target.value.toLowerCase()) {
      throw new Error(`Selected recipient ${target.value} does not match session's allowed recipient ${props.sessionConfig.allowedRecipient}. Update the input or recreate the session.`);
    }

    // --- Diagnostic: check on-chain session status for this signer and account ---
    try {
      // Read stored session hash for signer
      const sessionSignerAbi: Abi = [
        {
          name: "sessionSigner",
          type: "function",
          stateMutability: "view",
          inputs: [{ name: "signer", type: "address" }],
          outputs: [{ name: "sessionHash", type: "bytes32" }],
        },
        {
          name: "sessionStatus",
          type: "function",
          stateMutability: "view",
          inputs: [
            { name: "account", type: "address" },
            { name: "sessionHash", type: "bytes32" },
          ],
          outputs: [{ name: "status", type: "uint8" }],
        },
      ];
      const onChainSessionHash = await publicClient.readContract({
        address: props.sessionConfig.validatorAddress as Address,
        abi: sessionSignerAbi,
        functionName: "sessionSigner",
        args: [props.sessionConfig.sessionSigner as Address],
      });
      const status = await publicClient.readContract({
        address: props.sessionConfig.validatorAddress as Address,
        abi: sessionSignerAbi,
        functionName: "sessionStatus",
        args: [props.accountAddress as Address, onChainSessionHash as `0x${string}`],
      }) as bigint;
      // eslint-disable-next-line no-console
      console.log("On-chain session status:", status === 1n ? "Active" : status === 2n ? "Closed" : "NotInitialized", onChainSessionHash);
      if (status !== 1n) {
        throw new Error("Session is not Active on-chain for this account/signer. Recreate the session or ensure the spec matches.");
      }
    } catch (statusErr) {
      // eslint-disable-next-line no-console
      console.warn("Session status diagnostic failed:", statusErr);
    }

    // Log session keyed nonce (decimal) for visibility and to match Rust behavior
    try {
      const nonceKey = keyed_nonce_decimal(props.sessionConfig.sessionSigner);
      // eslint-disable-next-line no-console
      console.log("Session keyed nonce (decimal):", nonceKey);
    } catch {
      // ignore if helper not available
    }

    // eslint-disable-next-line no-console
    console.log("Session client ready, sending transaction via bundler...");

    // Prepare calls
    const calls = [
      {
        to: target.value as Address,
        value: parseEther(value.value),
        data: "0x" as `0x${string}`,
      },
    ];

    // Send transaction (viem will automatically use the bundler's userOperation overrides above)
    const txHash = await sessionClient.sendTransaction(calls[0]);
    // eslint-disable-next-line no-console
    console.log("Session transaction sent (tx hash):", txHash);
    result.value = { userOpHash: txHash };
  } catch (err: unknown) {
    // eslint-disable-next-line no-console
    console.error("Error sending session transaction:", err);
    error.value = err instanceof Error ? err.message : String(err);
  } finally {
    loading.value = false;
  }
}
</script>
