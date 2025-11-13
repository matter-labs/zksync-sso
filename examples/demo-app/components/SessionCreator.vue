<template>
  <div class="bg-teal-50 p-4 rounded-lg mb-4 border border-teal-200">
    <h2 class="text-lg font-semibold mb-3 text-teal-800">
      Step 2.5: Create Session
    </h2>
    <p class="text-sm text-gray-600 mb-4">
      Register the session key on-chain with its permissions. This must be signed by an authorized signer (EOA or Passkey).
    </p>

    <div class="space-y-3">
      <!-- Session Summary -->
      <div class="bg-white p-3 rounded border border-teal-200 text-sm">
        <p class="font-medium mb-2">
          Session Configuration:
        </p>
        <div class="space-y-1 text-xs text-gray-700">
          <div>
            <strong>Session Signer:</strong> {{ sessionConfig.sessionSigner.substring(0, 10) }}...{{ sessionConfig.sessionSigner.substring(38) }}
          </div>
          <div>
            <strong>Expires:</strong> {{ formatTimestamp(sessionConfig.expiresAt) }}
          </div>
          <div>
            <strong>Fee Limit:</strong> {{ formatWei(sessionConfig.feeLimit) }} ETH
          </div>
        </div>
      </div>

      <!-- Create Session Button -->
      <button
        :disabled="loading || sessionCreated"
        class="w-full px-4 py-2 bg-teal-600 text-white rounded hover:bg-teal-700 disabled:bg-gray-400 disabled:cursor-not-allowed"
        @click="createSessionOnChain"
      >
        {{ loading ? "Creating Session..." : sessionCreated ? "Session Created ✓" : "Create Session" }}
      </button>
    </div>

    <!-- Creation Result -->
    <div
      v-if="result"
      class="mt-4 p-3 bg-green-50 border border-green-200 rounded"
    >
      <p class="font-medium text-green-800">
        Session Created Successfully!
      </p>
      <div class="mt-2">
        <strong class="text-sm">UserOp Hash:</strong>
        <code class="block mt-1 px-2 py-1 bg-white rounded text-xs font-mono break-all">
          {{ result.userOpHash }}
        </code>
      </div>
    </div>

    <div
      v-if="error"
      class="mt-4 p-3 bg-red-50 border border-red-200 rounded"
    >
      <p class="font-medium text-red-600">
        Error Creating Session
      </p>
      <p class="text-sm mt-1">
        {{ error }}
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { createPublicClient, http, parseEther, type Chain, type Address } from "viem";
import { createBundlerClient } from "viem/account-abstraction";
import { createSession as createSessionAction, toEcdsaSmartAccount, LimitType } from "zksync-sso-4337/client";

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
  eoaValidatorAddress: string;
  eoaPrivateKey: string;
}>();

// Emit event when session is created
const emit = defineEmits<{
  sessionCreated: [];
}>();

// Local state
const loading = ref(false);
const sessionCreated = ref(false);
const result = ref<{ userOpHash: string } | null>(null);
const error = ref("");

// Format timestamp to readable date
function formatTimestamp(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleString("en-US", {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

// Format wei to ETH
function formatWei(wei: string): string {
  try {
    return parseEther(wei).toString();
  } catch {
    return "0.001";
  }
}

async function createSessionOnChain() {
  // Guard against multiple simultaneous calls
  if (loading.value || sessionCreated.value) {
    // eslint-disable-next-line no-console
    console.log("⚠️ Session creation already in progress or completed, ignoring duplicate call");
    return;
  }

  loading.value = true;
  error.value = "";
  result.value = null;

  try {
    // eslint-disable-next-line no-console
    console.log("=== Starting session creation ===");
    // eslint-disable-next-line no-console
    console.log("Props:", {
      accountAddress: props.accountAddress,
      eoaPrivateKey: props.eoaPrivateKey?.slice(0, 10) + "...",
      eoaValidatorAddress: props.eoaValidatorAddress,
      sessionConfig: props.sessionConfig,
    });

    // Load contracts
    // eslint-disable-next-line no-console
    console.log("Fetching contracts.json...");
    const response = await fetch("/contracts.json");
    const contracts = await response.json();
    // eslint-disable-next-line no-console
    console.log("Contracts loaded:", contracts);

    const chain = {
      id: contracts.chainId,
      name: "Anvil",
      nativeCurrency: { name: "Ether", symbol: "ETH", decimals: 18 },
      rpcUrls: { default: { http: [contracts.rpcUrl] } },
    } satisfies Chain;

    // Create clients
    // eslint-disable-next-line no-console
    console.log("Creating public client...");
    const publicClient = createPublicClient({
      chain,
      transport: http(),
    });

    // Create EOA smart account (authorized to create sessions)
    // eslint-disable-next-line no-console
    console.log("Creating EOA smart account...");
    const eoaAccount = await toEcdsaSmartAccount({
      client: publicClient,
      address: props.accountAddress as Address,
      signerPrivateKey: props.eoaPrivateKey as `0x${string}`,
      eoaValidatorAddress: props.eoaValidatorAddress as Address,
    });

    // eslint-disable-next-line no-console
    console.log("EOA smart account created:", eoaAccount.address);

    // eslint-disable-next-line no-console
    console.log("Creating bundler client with URL:", contracts.bundlerUrl);
    const bundlerClient = createBundlerClient({
      account: eoaAccount,
      transport: http(contracts.bundlerUrl || "http://localhost:4337"),
      chain,
      // Provide execution client so fees & non-bundler RPCs are fetched from the node, not the bundler.
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      client: publicClient as any,
    });

    // Define session spec (must match what will be used in SessionTransactionSender)
    const sessionSpec = {
      signer: props.sessionConfig.sessionSigner as Address,
      expiresAt: BigInt(props.sessionConfig.expiresAt),
      feeLimit: {
        limitType: LimitType.Lifetime,
        limit: BigInt(props.sessionConfig.feeLimit),
        period: 0n,
      },
      callPolicies: [],
      transferPolicies: [
        {
          // Explicit recipient required by validator (no wildcard support)
          target: (props.sessionConfig.allowedRecipient || "0x0000000000000000000000000000000000000000") as Address,
          maxValuePerUse: parseEther("0.1"), // Max 0.1 ETH per transaction
          valueLimit: {
            limitType: LimitType.Unlimited,
            limit: 0n,
            period: 0n,
          },
        },
      ],
    };

    // eslint-disable-next-line no-console
    console.log("Session spec created:", sessionSpec);
    // eslint-disable-next-line no-console
    console.log("Calling createSession with:", {
      bundlerClient: !!bundlerClient,
      sessionSpec: !!sessionSpec,
      sessionValidator: props.sessionConfig.validatorAddress,
    });

    // Create the session on-chain
    let sessionResult;
    try {
      sessionResult = await createSessionAction(bundlerClient, {
        sessionSpec,
        contracts: {
          sessionValidator: props.sessionConfig.validatorAddress as Address,
        },
      });
    } catch (createSessionError) {
      // eslint-disable-next-line no-console
      console.error("createSession threw error:", createSessionError);
      throw createSessionError;
    }

    // eslint-disable-next-line no-console
    console.log("createSession returned:", sessionResult);

    if (!sessionResult || !sessionResult.userOpHash) {
      throw new Error(`createSession returned invalid result: ${JSON.stringify(sessionResult)}`);
    }

    const { userOpHash } = sessionResult;

    // eslint-disable-next-line no-console
    console.log("Session created successfully! UserOp hash:", userOpHash);

    result.value = { userOpHash };
    sessionCreated.value = true;
    emit("sessionCreated");
  } catch (err: unknown) {
    // eslint-disable-next-line no-console
    console.error("Error creating session:", err);
    error.value = err instanceof Error ? err.message : String(err);
  } finally {
    loading.value = false;
  }
}
</script>
