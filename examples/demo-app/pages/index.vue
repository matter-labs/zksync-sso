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
      v-if="errorMessage"
      class="p-4 mt-4 mb-4 max-w-96 text-sm text-red-800 rounded-lg bg-red-50 dark:bg-gray-800 dark:text-red-400"
    >
      <span class="font-medium">{{ errorMessage }}</span>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { disconnect, getBalance, watchAccount, sendTransaction, createConfig, connect, reconnect, waitForTransactionReceipt, type GetBalanceReturnType } from "@wagmi/core";
import { zksyncSsoConnector, eraTestNode } from "zksync-sso-wagmi-connector";
import { createWalletClient, http, parseEther, type Address } from "viem";
import { privateKeyToAccount } from "viem/accounts";
import { getGeneralPaymasterInput } from "viem/zksync";
import PaymasterContract from "../forge-output.json";

const chain = eraTestNode; // Now using the exported eraTestNode instead of zksyncInMemoryNode

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

const address = ref<Address | null>(null);
const balance = ref<GetBalanceReturnType | null>(null);
const errorMessage = ref<string | null>(null);
const isSendingEth = ref<boolean>(false);

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
        paymaster: PaymasterContract.deployedTo as `0x${string}`,
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
</script>
