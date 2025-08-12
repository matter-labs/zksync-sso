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
        Message Signature Verification
      </h2>
      <div class="mb-4">
        <label class="block text-gray-700 text-sm font-bold mb-2">
          Message to Sign:
        </label>
        <input
          v-model="messageToSign"
          type="text"
          class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline max-w-md"
        >
      </div>
      <button
        class="bg-green-500 hover:bg-green-700 text-white font-bold py-2 px-4 rounded disabled:bg-slate-300"
        :disabled="isSigningMessage"
        @click="signMessageHandler"
      >
        {{ isSigningMessage ? 'Signing...' : 'Sign Message' }}
      </button>
      <div
        v-if="messageSignature"
        class="mt-4"
      >
        <p class="break-all">
          <strong>Signature:</strong> <span class="text-xs line-clamp-3">{{ messageSignature }}</span>
        </p>
      </div>
      <div
        v-if="isVerifyingSignature"
        class="mt-4"
      >
        <p class="text-gray-600">
          Verifying signature...
        </p>
      </div>
      <div
        v-else-if="isValidSignature !== null"
        class="mt-4"
      >
        <p :class="isValidSignature ? 'text-green-600' : 'text-red-600'">
          <strong>Verification Result:</strong> {{ isValidSignature ? 'Valid ✓' : 'Invalid ✗' }}
        </p>
      </div>
    </div>

    <div
      v-if="address"
      class="mt-8 border-t pt-4"
    >
      <h2 class="text-xl font-bold mb-4">
        Typed Data Signature Verification
      </h2>
      <div class="mb-4">
        <label class="block text-gray-700 text-sm font-bold mb-2">
          Typed Data (EIP-712):
        </label>
        <pre class="bg-gray-100 p-3 rounded text-sm overflow-x-auto max-w-2xl">{{ JSON.stringify(typedData, null, 2) }}</pre>
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
          <strong>Typed Data Signature:</strong> {{ typedDataSignature }}
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
import { disconnect, getBalance, watchAccount, sendTransaction, createConfig, connect, reconnect, waitForTransactionReceipt, signMessage, signTypedData, type GetBalanceReturnType } from "@wagmi/core";
import { zksyncSsoConnector } from "zksync-sso/connector";
import { zksyncInMemoryNode } from "@wagmi/core/chains";
import { createWalletClient, http, parseEther, type Address, createPublicClient, type Hash } from "viem";
import { privateKeyToAccount } from "viem/accounts";
import { getGeneralPaymasterInput } from "viem/zksync";
import PaymasterContract from "../forge-output.json";

const chain = zksyncInMemoryNode;

const testTransferTarget = "0x55bE1B079b53962746B2e86d12f158a41DF294A6";
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

const publicClient = createPublicClient({
  chain: chain,
  transport: http(),
});

const address = ref<Address | null>(null);
const balance = ref<GetBalanceReturnType | null>(null);
const errorMessage = ref<string | null>(null);
const isSendingEth = ref<boolean>(false);
const messageToSign = ref<string>("hello world");
const messageSignature = ref<Hash | null>(null);
const isValidSignature = ref<boolean | null>(null);
const isSigningMessage = ref<boolean>(false);
const isVerifyingSignature = ref<boolean>(false);

const typedDataSignature = ref<Hash | null>(null);
const isValidTypedDataSignature = ref<boolean | null>(null);
const isSigningTypedData = ref<boolean>(false);
const isVerifyingTypedDataSignature = ref<boolean>(false);

const typedData = {
  domain: {
    name: "ZKsync SSO Demo",
    version: "1",
    chainId: chain.id,
    verifyingContract: "0x0000000000000000000000000000000000000000",
  },
  types: {
    Person: [
      { name: "name", type: "string" },
      { name: "wallet", type: "address" },
    ],
    Mail: [
      { name: "from", type: "Person" },
      { name: "to", type: "Person" },
      { name: "contents", type: "string" },
    ],
  },
  primaryType: "Mail" as const,
  message: {
    from: {
      name: "Alice",
      wallet: "0xa1cf087DB965Ab02Fb3CFaCe1f5c63935815f044",
    },
    to: {
      name: "Bob",
      wallet: "0x6cC8cf7f6b488C58AA909B77E6e65c631c204784",
    },
    contents: "Hello, ZKsync!",
  },
} as const;

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

const signMessageHandler = async () => {
  if (!address.value) return;

  errorMessage.value = "";
  isSigningMessage.value = true;
  isValidSignature.value = null;
  try {
    const signature = await signMessage(wagmiConfig, {
      message: messageToSign.value,
    });
    messageSignature.value = signature;
  } catch (error) {
    // eslint-disable-next-line no-console
    console.error("Message signing failed:", error);
    errorMessage.value = "Message signing failed, see console for more info.";
  } finally {
    isSigningMessage.value = false;
  }
};

const verifySignatureAutomatically = async () => {
  if (!address.value || !messageSignature.value) {
    isVerifyingSignature.value = false;
    isValidSignature.value = null;
    return;
  }

  isVerifyingSignature.value = true;
  try {
    // const { salt, ...domain } = (await publicClient.getEip712Domain({
    //   address: address.value,
    // })).domain;
    // const struct = {
    //   domain,
    //   types: {
    //     PersonalSign: [{ name: "prefixed", type: "bytes" }],
    //   },
    //   primaryType: "PersonalSign",
    //   message: {
    //     prefixed: toPrefixedMessage(message),
    //   },
    // } as const;
    // const isValid = await erc1271Caller.validateStruct(
    //   struct,
    //   address.value,
    //   messageSignature.value,
    // );
    const isValid = await publicClient.verifyMessage({
      address: address.value,
      message: messageToSign.value,
      signature: messageSignature.value,
    });
    isValidSignature.value = isValid;
  } catch (error) {
    // eslint-disable-next-line no-console
    console.error("Signature verification failed:", error);
    isValidSignature.value = false;
  } finally {
    isVerifyingSignature.value = false;
  }
};

const signTypedDataHandler = async () => {
  if (!address.value) return;

  errorMessage.value = "";
  isSigningTypedData.value = true;
  isValidTypedDataSignature.value = null;
  try {
    const signature = await signTypedData(wagmiConfig, typedData);
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
    const isValid = await publicClient.verifyTypedData({
      address: address.value,
      domain: typedData.domain,
      types: typedData.types,
      primaryType: typedData.primaryType,
      message: typedData.message,
      signature: typedDataSignature.value,
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

watch([messageToSign, address], () => messageSignature.value = null);
watch(messageSignature, verifySignatureAutomatically);
watch(typedDataSignature, verifyTypedDataSignatureAutomatically);
</script>
