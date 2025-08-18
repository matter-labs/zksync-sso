<template>
  <div class="container mx-auto px-4 py-8">
    <h1 class="text-3xl font-bold mb-4">
      ZKsync SSO Demo
    </h1>
    <button
      class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded mr-4"
      @click="address ? disconnectWallet() : connectWallet(false)"
    >
      {{ address ? "Disconnect" : "Connect" }}
    </button>
    <button
      v-if="!address"
      class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
      @click="address ? disconnectWallet() : connectWallet(true)"
    >
      Connect with Session
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
    <button
      v-if="address"
      class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded mt-3 mr-4 disabled:bg-slate-300"
      :disabled="isSendingEth"
      @click="sendTokens(false)"
    >
      Send 0.1 ETH
    </button>
    <button
      v-if="address"
      class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded mt-3 disabled:bg-slate-300"
      :disabled="isSendingEth"
      @click="sendTokens(true)"
    >
      Send 0.1 ETH with Paymaster
    </button>

    <div
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
          <strong>Signature:</strong> <span class="text-xs line-clamp-3">{{ typedDataSignature }}</span>
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
    </div>

    <div
      v-if="errorMessage"
      class="p-4 mt-4 mb-4 max-w-96 text-sm text-red-800 rounded-lg bg-red-50 dark:bg-gray-800 dark:text-red-400"
    >
      <span class="font-medium">{{ errorMessage }}</span>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { disconnect, getBalance, watchAccount, sendTransaction, createConfig, connect, reconnect, waitForTransactionReceipt, type GetBalanceReturnType, signTypedData, readContract } from "@wagmi/core";
import { zksyncSsoConnector } from "@zksync-sso/connector-export";
import { createWalletClient, createPublicClient, http, parseEther, type Address, type Hash } from "viem";
import { privateKeyToAccount } from "viem/accounts";
import { getGeneralPaymasterInput, zksyncInMemoryNode } from "viem/zksync";
import PaymasterContract from "../forge-output-paymaster.json";
import ERC1271CallerContract from "../forge-output-erc1271.json";

const chain = zksyncInMemoryNode;

const testTransferTarget = "0x55bE1B079b53962746B2e86d12f158a41DF294A6";

const publicClient = createPublicClient({
  chain: chain,
  transport: http(),
});
const zksyncConnectorWithSession = zksyncSsoConnector({
  authServerUrl: "http://localhost:3002/confirm",
  session: {
    feeLimit: parseEther("0.1"),
    transfers: [
      {
        to: testTransferTarget,
        valueLimit: parseEther("0.1"),
      },
    ],
  },
});
const zksyncConnector = zksyncSsoConnector({
  authServerUrl: "http://localhost:3002/confirm",
});
const wagmiConfig = createConfig({
  chains: [chain],
  connectors: [zksyncConnector],
  transports: {
    [chain.id]: http(),
  },
});
reconnect(wagmiConfig);

const address = ref<Address | null>(null);
const balance = ref<GetBalanceReturnType | null>(null);
const errorMessage = ref<string | null>(null);

const fundAccount = async () => {
  if (!address.value) throw new Error("Not connected");

  const richClient = createWalletClient({
    account: privateKeyToAccount("0x3eb15da85647edd9a1159a4a13b9e7c56877c4eb33f614546d4db06a51868b1c"),
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
    address.value = data.address || null;
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
  if (currentBalance && currentBalance.value < parseEther("0.2")) {
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

const connectWallet = async (useSession: boolean) => {
  try {
    errorMessage.value = "";
    connect(wagmiConfig, {
      connector: useSession ? zksyncConnectorWithSession : zksyncConnector,
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
  await disconnect(wagmiConfig);
};

/* Send ETH */
const isSendingEth = ref<boolean>(false);

const sendTokens = async (usePaymaster: boolean) => {
  if (!address.value) return;

  errorMessage.value = "";
  isSendingEth.value = true;
  try {
    let transactionHash;

    if (usePaymaster) {
      transactionHash = await sendTransaction(wagmiConfig, {
        to: testTransferTarget,
        value: parseEther("0.1"),
        paymaster: PaymasterContract.deployedTo as Address,
        paymasterInput: getGeneralPaymasterInput({ innerInput: "0x" }),
      });
    } else {
      transactionHash = await sendTransaction(wagmiConfig, {
        to: testTransferTarget,
        value: parseEther("0.1"),
      });
    }

    // FIXME: When not using sessions, sendTransaction returns a map and not a string
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    if ((transactionHash as any).value !== undefined) {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      transactionHash = (transactionHash as any).value;
    }

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

const signTypedDataHandler = async () => {
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
      domain: callerDomain,
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
        inputs: [
          {
            components: [
              { name: "message", type: "string" },
              { name: "value", type: "uint256" },
            ],
            name: "testStruct",
            type: "tuple",
          },
          {
            name: "signer",
            type: "address",
          },
          {
            name: "signature",
            type: "bytes",
          },
        ],
        name: "validateStruct",
        outputs: [
          {
            name: "",
            type: "bool",
          },
        ],
        stateMutability: "view",
        type: "function",
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
