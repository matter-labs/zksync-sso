<template>
  <div class="container mx-auto px-4 py-8">
    <h1 class="text-3xl font-bold mb-4">
      ZKsync SSO Demo
    </h1>
    <button
      class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded mr-4"
      @click="address ? disconnectWallet() : connectWallet('regular')"
    >
      {{ address ? "Disconnect" : "Connect" }}
    </button>
    <button
      v-if="!address"
      class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded mr-4"
      @click="connectWallet('session')"
    >
      Connect with Session
    </button>
    <button
      v-if="!address"
      title="Connect with paymaster sponsoring gas (no session)"
      class="bg-green-500 hover:bg-green-700 text-white font-bold py-2 px-4 rounded mr-4"
      @click="connectWallet('paymaster')"
    >
      Connect (Paymaster)
    </button>
    <button
      v-if="!address"
      title="Connect with session and paymaster sponsorship"
      class="bg-green-500 hover:bg-green-700 text-white font-bold py-2 px-4 rounded"
      @click="connectWallet('session-paymaster')"
    >
      Connect Session (Paymaster)
    </button>
    <div
      v-if="address"
      class="mt-4"
    >
      <p>Connected Address: {{ address }}</p>
    </div>
    <div
      v-if="address"
      class="mt-4"
    >
      <p>Balance: {{ balance ? `${balance.formatted} ${balance.symbol}` : '...' }}</p>
    </div>
    <div
      v-if="address"
      class="mt-4"
    >
      <p>Connection Mode: {{ connectionMode }} {{ isPaymasterEnabled ? '(Gas Sponsored ✨)' : '' }}</p>
    </div>
    <button
      v-if="address"
      :class="isPaymasterEnabled ? 'bg-green-500 hover:bg-green-700' : 'bg-blue-500 hover:bg-blue-700'"
      class="text-white font-bold py-2 px-4 rounded mt-3 mr-4 disabled:bg-slate-300"
      :disabled="isSendingEth"
      @click="sendTokens()"
    >
      Send 0.1 ETH{{ isPaymasterEnabled ? ' (Paymaster)' : '' }}
    </button>

    <!-- <div
      v-if="address"
      class="mt-8 border-t pt-4"
    >
      <h2 class="text-xl font-bold mb-4">
        Typed Data Signature Verification
      </h2>
      <div class="mb-4">
        <pre class="bg-gray-100 p-3 rounded text-xs overflow-x-auto max-w-2xl max-h-60">{{ JSON.stringify(typedData, null, 2) }}</pre>
      </div>
      <div
        v-if="ERC1271CallerContract.deployedTo"
        class="mb-4 text-xs text-gray-600"
      >
        <p>ERC1271 Caller address: {{ ERC1271CallerContract.deployedTo }}</p>
      </div>
      <button
        class="bg-purple-500 hover:bg-purple-700 text-white font-bold py-2 px-4 rounded disabled:bg-slate-300"
        :disabled="isSigningTypedData"
        @click="signTypedDataHandler"
      >
        {{ isSigningTypedData ? 'Signing...' : 'Sign Typed Data' }}
      </button>
      <div
        v-if="typedDataSignature"
        class="mt-4"
      >
        <p class="break-all">
          <strong>Signature:</strong> <span class="text-xs line-clamp-2">{{ typedDataSignature }}</span>
        </p>
      </div>
      <div
        v-if="isVerifyingTypedDataSignature"
        class="mt-4"
      >
        <p class="text-gray-600">
          Verifying typed data signature...
        </p>
      </div>
      <div
        v-else-if="isValidTypedDataSignature !== null"
        class="mt-4"
      >
        <p :class="isValidTypedDataSignature ? 'text-green-600' : 'text-red-600'">
          <strong>Typed Data Verification Result:</strong> {{ isValidTypedDataSignature ? 'Valid ✓' : 'Invalid ✗' }}
        </p>
      </div>
    </div> -->

    <div
      v-if="errorMessage"
      class="p-4 mt-4 mb-4 max-w-96 text-sm text-red-800 rounded-lg bg-red-50 dark:bg-gray-800 dark:text-red-400"
    >
      <span class="font-medium">{{ errorMessage }}</span>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { disconnect, getBalance, watchAccount, createConfig, connect, waitForTransactionReceipt, type GetBalanceReturnType, signTypedData, readContract, getConnectorClient } from "@wagmi/core";
import { createWalletClient, createPublicClient, http, parseEther, toHex, type Address, type Hash } from "viem";
import { zksyncSsoConnector } from "zksync-sso-4337/connector";
import { privateKeyToAccount } from "viem/accounts";
import { localhost } from "viem/chains";
import { onMounted } from "vue";
import ERC1271CallerContract from "../forge-output-erc1271.json";

// Load contracts from public JSON at runtime to avoid ESM JSON import issues
const contractsUrl = "/contracts.json";
let contractsAnvil: Record<string, unknown> = {};
let testPaymasterAddress: Address | undefined;

const runtimeConfig = useRuntimeConfig();

// Prime with runtime config so server-side renders can still set it
testPaymasterAddress = (runtimeConfig.public?.testPaymasterAddress as Address | undefined)
?? (runtimeConfig.public?.NUXT_PUBLIC_TEST_PAYMASTER as Address | undefined);

if (typeof window !== "undefined") {
  fetch(contractsUrl)
    .then((r) => r.json())
    .then((json) => {
      contractsAnvil = json;
      // Prefer contract JSON values when available
      testPaymasterAddress = (contractsAnvil as { testPaymaster?: Address; TestPaymaster?: Address }).testPaymaster
      ?? (contractsAnvil as { TestPaymaster?: Address }).TestPaymaster
      ?? testPaymasterAddress;
    })
    .catch(() => undefined);
}

const chain = localhost;

const testTransferTarget = "0x55bE1B079b53962746B2e86d12f158a41DF294A6";

const sessionConfig = {
  feeLimit: parseEther("0.1"),
  transfers: [
    {
      to: testTransferTarget,
      valueLimit: parseEther("0.1"),
    },
  ],
};

const buildConnector = (mode: "regular" | "session" | "paymaster" | "session-paymaster") => {
  const baseConfig: Parameters<typeof zksyncSsoConnector>[0] = {
    authServerUrl: "http://localhost:3002/confirm",
  };

  if (mode === "session" || mode === "session-paymaster") {
    baseConfig.session = sessionConfig;
  }

  if (mode === "paymaster" || mode === "session-paymaster") {
    baseConfig.paymaster = testPaymasterAddress;
  }

  return zksyncSsoConnector(baseConfig);
};

const publicClient = createPublicClient({
  chain: chain,
  transport: http(),
});
const wagmiConfig = createConfig({
  chains: [chain],
  connectors: [buildConnector("regular")],
  transports: {
    [chain.id]: http(),
  },
});

const address = ref<Address | null>(null);
const balance = ref<GetBalanceReturnType | null>(null);
const errorMessage = ref<string | null>(null);
const connectionMode = ref<string>("Not connected");
const isPaymasterEnabled = computed(() => connectionMode.value === "paymaster" || connectionMode.value === "session-paymaster");
const isInitializing = ref(true);

// Ensure fresh, unauthenticated state on page load so the connect buttons render
onMounted(async () => {
  await disconnect(wagmiConfig).catch(() => undefined);
  address.value = null;
  balance.value = null;
  isInitializing.value = false;
});

const fundAccount = async () => {
  if (!address.value) throw new Error("Not connected");

  const richClient = createWalletClient({
    account: privateKeyToAccount("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"), // Rich anvil account
    chain: chain,
    transport: http(),
  });

  let transactionHash = await richClient.sendTransaction({
    to: address.value,
    value: parseEther("1"),
  });
  // FIXME: When not using sessions, sendTransaction returns a map and not a string
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  if ((transactionHash as any).value !== undefined) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    transactionHash = (transactionHash as any).value;
  }

  await waitForTransactionReceipt(wagmiConfig, {
    hash: transactionHash,
  });
};

watchAccount(wagmiConfig, {
  async onChange(data) {
    // Don't update address during initialization to avoid race with disconnect
    if (!isInitializing.value) {
      address.value = data.address || null;
    }
  },
});

watch(address, async () => {
  if (!address.value) {
    balance.value = null;
    return;
  }

  let currentBalance = await getBalance(wagmiConfig, {
    address: address.value,
  });

  // Skip auto-funding if using paymaster (check query param)
  const urlParams = new URLSearchParams(window.location.search);
  const skipFunding = urlParams.get("skipFunding") === "true";

  if (!skipFunding && currentBalance && currentBalance.value < parseEther("0.2")) {
    await fundAccount().catch((error) => {
      // eslint-disable-next-line no-console
      console.error("Funding failed:", error);
    });
    currentBalance = await getBalance(wagmiConfig, {
      address: address.value,
    });
  }

  balance.value = currentBalance;
}, { immediate: true });

const connectWallet = async (mode: "regular" | "session" | "paymaster" | "session-paymaster") => {
  try {
    errorMessage.value = "";
    const connector = buildConnector(mode);

    if ((mode === "paymaster" || mode === "session-paymaster") && !testPaymasterAddress) {
      errorMessage.value = "Paymaster address is not configured.";
      return;
    }

    // Track which mode was used for connection
    connectionMode.value = mode;

    connect(wagmiConfig, {
      connector,
      chainId: chain.id,
    });
  } catch (error) {
    errorMessage.value = "Connect failed, see console for more info.";
    // eslint-disable-next-line no-console
    console.error("Connection failed:", error);
  }
};

const disconnectWallet = async () => {
  errorMessage.value = "";
  try {
    await disconnect(wagmiConfig);
  } catch (error) {
    // If connector doesn't have disconnect method, manually reset state
    // eslint-disable-next-line no-console
    console.warn("Disconnect failed, manually resetting state:", error);
    address.value = null;
    balance.value = null;
  }
  connectionMode.value = "Not connected";
};

/* Send ETH */
const isSendingEth = ref<boolean>(false);

const sendTokens = async () => {
  if (!address.value) return;

  errorMessage.value = "";
  isSendingEth.value = true;
  try {
    // Get the connector client which will have paymaster config if connected with paymaster mode
    const connectorClient = await getConnectorClient(wagmiConfig);

    // Use the provider's request method which routes through our custom client
    const transactionHash = await connectorClient.request({
      method: "eth_sendTransaction",
      params: [{
        from: address.value,
        to: testTransferTarget,
        value: toHex(parseEther("0.1")),
      }],
    }) as Hash;

    const receipt = await waitForTransactionReceipt(wagmiConfig, {
      hash: transactionHash,
    });
    balance.value = await getBalance(wagmiConfig, {
      address: address.value,
    });
    if (receipt.status === "reverted") throw new Error("Transaction reverted");
  } catch (error) {
    // eslint-disable-next-line no-console
    console.error("Transaction failed:", error);
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    let transactionFailureDetails = (error as any).cause?.cause?.cause?.data?.originalError?.cause?.details;
    if (!transactionFailureDetails) {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      transactionFailureDetails = (error as any).cause?.cause?.data?.originalError?.cause?.details;
    }
    if (!transactionFailureDetails) {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      transactionFailureDetails = (error as any).cause?.details;
    }

    if (transactionFailureDetails) {
      errorMessage.value = transactionFailureDetails;
    } else {
      errorMessage.value = "Transaction failed, see console for more info.";
    }
  } finally {
    isSendingEth.value = false;
  }
};

/* Typed data */
const typedDataSignature = ref<Hash | null>(null);
const isValidTypedDataSignature = ref<boolean | null>(null);
const isSigningTypedData = ref<boolean>(false);
const isVerifyingTypedDataSignature = ref<boolean>(false);

const typedData = {
  types: {
    TestStruct: [
      { name: "message", type: "string" },
      { name: "value", type: "uint256" },
    ],
  },
  primaryType: "TestStruct",
  message: {
    message: "test",
    value: 42n,
  },
} as const;

const _signTypedDataHandler = async () => {
  if (!address.value) return;

  errorMessage.value = "";
  isSigningTypedData.value = true;
  isValidTypedDataSignature.value = null;
  try {
    const erc1271CallerAddress = ERC1271CallerContract.deployedTo as Address;
    const { domain: callerDomain } = await publicClient.getEip712Domain({
      address: erc1271CallerAddress,
    });

    const signature = await signTypedData(wagmiConfig, {
      domain: {
        ...callerDomain,
        salt: undefined, // Otherwise the signature verification fails (todo: figure out why)
      },
      ...typedData,
    });
    typedDataSignature.value = signature;
  } catch (error) {
    // eslint-disable-next-line no-console
    console.error("Typed data signing failed:", error);
    errorMessage.value = "Typed data signing failed, see console for more info.";
  } finally {
    isSigningTypedData.value = false;
  }
};

const verifyTypedDataSignatureAutomatically = async () => {
  if (!address.value || !typedDataSignature.value) {
    isValidTypedDataSignature.value = null;
    return;
  }

  isVerifyingTypedDataSignature.value = true;
  try {
    const contractAddress = ERC1271CallerContract.deployedTo as Address;

    const isValid = await readContract(wagmiConfig, {
      address: contractAddress,
      abi: [{
        type: "function",
        name: "validateStruct",
        stateMutability: "view",
        inputs: [
          {
            name: "testStruct", type: "tuple", internalType: "struct ERC1271Caller.TestStruct",
            components: [
              { name: "message", type: "string", internalType: "string" },
              { name: "value", type: "uint256", internalType: "uint256" },
            ],
          },
          { name: "signer", type: "address", internalType: "address" },
          { name: "encodedSignature", type: "bytes", internalType: "bytes" },
        ],
        outputs: [{ name: "", type: "bool", internalType: "bool" }],
      }] as const,
      functionName: "validateStruct",
      args: [
        typedData.message,
        address.value,
        typedDataSignature.value,
      ],
    });

    isValidTypedDataSignature.value = isValid;
  } catch (error) {
    // eslint-disable-next-line no-console
    console.error("Typed data signature verification failed:", error);
    isValidTypedDataSignature.value = false;
  } finally {
    isVerifyingTypedDataSignature.value = false;
  }
};

watch(address, () => typedDataSignature.value = null);
watch(typedDataSignature, verifyTypedDataSignatureAutomatically);

// eslint-disable-next-line @typescript-eslint/no-explicit-any
(window.BigInt as any).prototype.toJSON = function () {
  return this.toString();
};
</script>
