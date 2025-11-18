<template>
  <div class="bg-purple-50 p-4 rounded-lg mb-4 border border-purple-200">
    <h2 class="text-lg font-semibold mb-3 text-purple-800">
      Typed Data (ERC-7739) Signature & ERC-1271 Verification
    </h2>
    <p class="text-sm text-gray-600 mb-4">
      Sign EIP-712 typed data wrapped via ERC-7739 and validate it on-chain through ERC-1271.
    </p>

    <div class="space-y-3">
      <div class="text-xs text-gray-600">
        <p class="font-medium">
          Typed Data:
        </p>
        <pre class="bg-white p-3 rounded border overflow-x-auto">{{ JSON.stringify(erc7739TypedData, null, 2) }}</pre>
      </div>

      <div
        v-if="erc1271CallerAddress"
        class="text-xs text-gray-600"
      >
        <p class="font-medium">
          ERC1271 Caller:
        </p>
        <code class="bg-white px-2 py-1 rounded">{{ erc1271CallerAddress }}</code>
      </div>

      <div
        v-if="!connectedForTypedData"
        class="text-xs text-gray-600"
      >
        <p class="mb-2">
          Connect a wallet (smart account) for signing. This uses the SSO connector locally for this section.
        </p>
        <button
          :disabled="isConnectingTypedData"
          class="w-full px-4 py-2 bg-indigo-500 text-white rounded hover:bg-indigo-600 disabled:opacity-50"
          @click="connectForTypedData"
        >
          {{ isConnectingTypedData ? 'Connecting...' : 'Connect for Typed Data' }}
        </button>
      </div>

      <div
        v-else
        class="space-y-3"
      >
        <button
          :disabled="isSigningErc7739 || !accountAddress"
          class="w-full px-4 py-2 bg-purple-500 text-white rounded hover:bg-purple-600 disabled:opacity-50"
          @click="signErc7739"
        >
          {{ isSigningErc7739 ? 'Signing...' : 'Sign Typed Data (ERC-7739)' }}
        </button>

        <div
          v-if="erc7739Signature"
          class="text-xs"
        >
          <p class="font-medium">
            Encoded Signature:
          </p>
          <p class="mt-1 break-all bg-white p-2 rounded border">
            {{ erc7739Signature }}
          </p>
        </div>

        <div>
          <button
            :disabled="!erc7739Signature || isVerifyingErc7739 || !accountAddress"
            class="w-full px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600 disabled:opacity-50"
            @click="verifyErc7739"
          >
            {{ isVerifyingErc7739 ? 'Verifying...' : 'Verify On-Chain (ERC-1271)' }}
          </button>
        </div>

        <div
          v-if="erc7739VerifyResult !== null"
          class="mt-2"
        >
          <p :class="erc7739VerifyResult ? 'text-green-700' : 'text-red-700'">
            <strong>Verification Result:</strong>
            {{ erc7739VerifyResult ? 'Valid ✓' : 'Invalid ✗' }}
          </p>
        </div>

        <div
          v-if="erc7739Error"
          class="p-3 bg-red-50 rounded border border-red-300 text-xs text-red-700"
        >
          {{ erc7739Error }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import type { Address, Hex } from "viem";
import { http } from "viem";
import { createConfig as createWagmiConfig, reconnect as wagmiReconnect, signTypedData as wagmiSignTypedData, readContract as wagmiReadContract, connect as wagmiConnect } from "@wagmi/core";
import { zksyncSsoConnector as demoZksyncSsoConnector } from "zksync-sso-wagmi-connector";
import Erc1271CallerDeployment from "../forge-output-erc1271.json";
import { loadContracts, getChainConfig } from "~/utils/contracts";

const props = defineProps<{
  accountAddress?: string;
}>();

const erc1271CallerAddress = (Erc1271CallerDeployment as { deployedTo?: string }).deployedTo as Address | undefined;

const erc7739TypedData = {
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

const typedDataWagmiConfig = ref<ReturnType<typeof createWagmiConfig> | null>(null);
const isConnectingTypedData = ref(false);
const connectedForTypedData = ref(false);
const isSigningErc7739 = ref(false);
const isVerifyingErc7739 = ref(false);
const erc7739Signature = ref<Hex | null>(null);
const erc7739VerifyResult = ref<boolean | null>(null);
const erc7739Error = ref<string | null>(null);

async function connectForTypedData() {
  try {
    isConnectingTypedData.value = true;
    erc7739Error.value = null;
    const contracts = await loadContracts();
    const chain = getChainConfig(contracts);
    typedDataWagmiConfig.value = createWagmiConfig({
      chains: [chain],
      connectors: [demoZksyncSsoConnector()],
      transports: { [chain.id]: http(contracts.rpcUrl) },
    });
    wagmiReconnect(typedDataWagmiConfig.value);
    await wagmiConnect(typedDataWagmiConfig.value, { connector: demoZksyncSsoConnector(), chainId: chain.id });
    connectedForTypedData.value = true;
  } catch (err) {
    // eslint-disable-next-line no-console
    console.error("Connect (typed data) failed:", err);
    erc7739Error.value = "Connect failed, see console for details.";
  } finally {
    isConnectingTypedData.value = false;
  }
}

async function signErc7739() {
  if (!erc1271CallerAddress) {
    erc7739Error.value = "ERC1271Caller is not deployed. Run the deploy task first.";
    return;
  }
  if (!props.accountAddress) {
    erc7739Error.value = "Smart account not available. Deploy/connect first.";
    return;
  }
  if (!typedDataWagmiConfig.value) {
    erc7739Error.value = "Not connected for typed data. Click 'Connect for Typed Data' first.";
    return;
  }
  try {
    isSigningErc7739.value = true;
    erc7739Error.value = null;
    erc7739VerifyResult.value = null;
    erc7739Signature.value = null;

    const contracts = await loadContracts();
    const domain = {
      name: "ERC1271Caller",
      version: "1.0.0",
      chainId: contracts.chainId,
      verifyingContract: erc1271CallerAddress,
    } as const;

    const signature = await wagmiSignTypedData(typedDataWagmiConfig.value, {
      domain,
      types: erc7739TypedData.types,
      primaryType: erc7739TypedData.primaryType,
      message: erc7739TypedData.message,
    });
    erc7739Signature.value = signature as Hex;
  } catch (err) {
    // eslint-disable-next-line no-console
    console.error("Typed data signing (ERC-7739) failed:", err);
    erc7739Error.value = "Typed data signing failed, see console for details.";
  } finally {
    isSigningErc7739.value = false;
  }
}

async function verifyErc7739() {
  if (!erc1271CallerAddress || !erc7739Signature.value || !props.accountAddress || !typedDataWagmiConfig.value) return;
  try {
    isVerifyingErc7739.value = true;
    erc7739Error.value = null;

    const isValid = await wagmiReadContract(typedDataWagmiConfig.value, {
      address: erc1271CallerAddress,
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
        erc7739TypedData.message,
        props.accountAddress as Address,
        erc7739Signature.value as Hex,
      ],
    });
    erc7739VerifyResult.value = Boolean(isValid);
  } catch (err) {
    // eslint-disable-next-line no-console
    console.error("Typed data verification failed:", err);
    erc7739VerifyResult.value = false;
    erc7739Error.value = "Typed data verification failed, see console for details.";
  } finally {
    isVerifyingErc7739.value = false;
  }
}
</script>
