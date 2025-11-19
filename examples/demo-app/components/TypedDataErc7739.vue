<template>
  <div
    class="bg-purple-50 p-4 rounded-lg mb-4 border border-purple-200"
    data-testid="typed-data-section"
  >
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
        <pre class="bg-white p-3 rounded border overflow-x-auto">{{ typedDataDisplay }}</pre>
      </div>

      <!-- Actions: keep always rendered to simplify e2e selection -->
      <div class="text-xs space-y-2">
        <button
          :disabled="isSigningErc7739 || !props.accountAddress || !hasErc1271Caller"
          class="w-full px-4 py-2 bg-purple-500 text-white rounded hover:bg-purple-600 disabled:opacity-50"
          data-testid="typeddata-sign"
          aria-label="Sign Typed Data"
          @click="signErc7739"
        >
          {{ isSigningErc7739 ? 'Signing...' : 'Sign Typed Data (ERC-7739)' }}
        </button>

        <button
          v-if="erc7739Signature"
          :disabled="isVerifyingErc7739 || !props.accountAddress || !hasErc1271Caller"
          class="w-full px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600 disabled:opacity-50"
          data-testid="typeddata-verify"
          aria-label="Verify Signature On-Chain"
          @click="verifyErc7739"
        >
          {{ isVerifyingErc7739 ? 'Verifying...' : 'Verify Signature On-Chain (ERC-1271)' }}
        </button>
      </div>
      <div
        v-if="hasErc1271Caller"
        class="text-xs text-gray-600"
      >
        <p class="font-medium">
          ERC1271 Caller:
        </p>
        <code
          class="bg-white px-2 py-1 rounded"
          data-testid="erc1271-caller-address"
        >{{ erc1271CallerAddress }}</code>
      </div>

      <!-- Smart Account (Verifier) (optional display) -->
      <div class="text-xs text-gray-600">
        <p class="font-medium">
          Smart Account (Verifier):
        </p>
        <code class="bg-white px-2 py-1 rounded">{{ props.accountAddress || '— not set —' }}</code>
      </div>

      <div
        v-if="erc7739Signature"
        class="text-xs"
      >
        <p class="font-medium text-gray-700 mb-1">
          Signature:
        </p>
        <code
          class="block bg-white p-2 rounded border text-xs break-all"
          data-testid="typeddata-signature"
        >{{ erc7739Signature }}</code>
      </div>

      <div
        v-if="erc7739VerifyResult !== null"
        class="text-xs"
      >
        <p
          :class="erc7739VerifyResult ? 'text-green-700' : 'text-red-700'"
          class="font-medium"
        >
          <span data-testid="typeddata-verify-result">{{ erc7739VerifyResult ? '✓ Valid' : '✗ Invalid' }}</span>
        </p>
      </div>

      <div
        v-if="erc7739Error"
        class="text-xs text-red-600 bg-red-50 p-2 rounded border border-red-200"
        data-testid="typeddata-error"
      >
        {{ erc7739Error }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import type { Address, Hex } from "viem";
import { createPublicClient, http } from "viem";
import { privateKeyToAccount } from "viem/accounts";
import Erc1271CallerDeployment from "../forge-output-erc1271.json";
import { loadContracts } from "~/utils/contracts";
import { signTypedDataErc7739, hashTypedDataErc7739 } from "~/utils/erc7739";

const props = defineProps<{
  accountAddress?: string;
}>();

const erc1271CallerAddress = (Erc1271CallerDeployment as { deployedTo?: string }).deployedTo as Address | undefined;
const ZERO_ADDRESS = "0x0000000000000000000000000000000000000000" as Address;
const hasErc1271Caller = computed(() => !!erc1271CallerAddress && erc1271CallerAddress.toLowerCase() !== ZERO_ADDRESS.toLowerCase());

// Anvil account #1 private key (same as used in deployAccount)
// Address: 0x70997970C51812dc3A010C7d01b50e0d17dc79C8
const ANVIL_ACCOUNT_1_PRIVATE_KEY = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d" as Hex;
const eoaAccount = privateKeyToAccount(ANVIL_ACCOUNT_1_PRIVATE_KEY);

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

// Create a displayable version of typed data (converting BigInts to strings)
const typedDataDisplay = computed(() => {
  return JSON.stringify({
    ...erc7739TypedData,
    message: {
      ...erc7739TypedData.message,
      value: erc7739TypedData.message.value.toString(),
    },
  }, null, 2);
});

const isSigningErc7739 = ref(false);
const isVerifyingErc7739 = ref(false);
const erc7739Signature = ref<Hex | null>(null);
const erc7739VerifyResult = ref<boolean | null>(null);
const erc7739Error = ref<string | null>(null);

async function signErc7739() {
  if (!hasErc1271Caller.value) {
    erc7739Error.value = "ERC1271Caller is not deployed. Run the deploy task first.";
    return;
  }
  if (!props.accountAddress) {
    erc7739Error.value = "Smart account not available. Deploy account first.";
    return;
  }
  try {
    isSigningErc7739.value = true;
    erc7739Error.value = null;
    erc7739VerifyResult.value = null;
    erc7739Signature.value = null;

    const contracts = await loadContracts();

    // Sign using ERC-7739 wrapping with validator prefix
    // The signer uses .sign() to sign the raw hash without personal_sign wrapper
    // The validatorAddress is prepended BEFORE wrapping per ERC-7739/MSA requirements
    const signature = await signTypedDataErc7739({
      typedData: {
        domain: {
          name: "ERC1271Caller",
          version: "1.0.0",
          chainId: contracts.chainId,
          verifyingContract: erc1271CallerAddress as Address,
        },
        types: erc7739TypedData.types,
        primaryType: erc7739TypedData.primaryType,
        message: erc7739TypedData.message,
      },
      smartAccountAddress: props.accountAddress as Hex,
      signer: async (hash) => eoaAccount.sign({ hash }),
      validatorAddress: contracts.eoaValidator as Hex,
      chainId: contracts.chainId,
    });

    erc7739Signature.value = signature;
  } catch (err) {
    // eslint-disable-next-line no-console
    console.error("Typed data signing (ERC-7739) failed:", err);
    erc7739Error.value = "Typed data signing failed, see console for details.";
  } finally {
    isSigningErc7739.value = false;
  }
}

async function verifyErc7739() {
  if (!hasErc1271Caller.value || !erc7739Signature.value || !props.accountAddress) return;
  try {
    isVerifyingErc7739.value = true;
    erc7739Error.value = null;

    const contracts = await loadContracts();
    const publicClient = createPublicClient({
      transport: http(contracts.rpcUrl),
    });

    // The wrapped signature from signTypedDataErc7739 already contains:
    // validator || ecdsa || domain || contents || description || length
    // Debug logging
    // eslint-disable-next-line no-console
    console.log("[ERC-7739 Debug] Verification parameters:");
    // eslint-disable-next-line no-console
    console.log("  Account Address:", props.accountAddress);
    // eslint-disable-next-line no-console
    console.log("  EOA Validator:", contracts.eoaValidator);
    // eslint-disable-next-line no-console
    console.log("  Wrapped Signature Length:", erc7739Signature.value.length);
    // eslint-disable-next-line no-console
    console.log("  ERC1271Caller:", erc1271CallerAddress);

    // Read contract computed struct hash and typed data wrapper
    const onchainStructHash = await publicClient.readContract({
      address: erc1271CallerAddress as Address,
      abi: [
        {
          type: "function",
          name: "computeStructHash",
          stateMutability: "view",
          inputs: [
            {
              name: "testStruct",
              type: "tuple",
              components: [
                { name: "message", type: "string" },
                { name: "value", type: "uint256" },
              ],
            },
          ],
          outputs: [{ name: "", type: "bytes32" }],
        },
      ],
      functionName: "computeStructHash",
      args: [
        {
          message: erc7739TypedData.message.message,
          value: erc7739TypedData.message.value,
        },
      ],
    });

    const onchainDebug = await publicClient.readContract({
      address: erc1271CallerAddress as Address,
      abi: [
        {
          type: "function",
          name: "computeTypedDataSignAndFinalHash",
          stateMutability: "view",
          inputs: [
            {
              name: "testStruct",
              type: "tuple",
              components: [
                { name: "message", type: "string" },
                { name: "value", type: "uint256" },
              ],
            },
            { name: "contentsDescription", type: "string" },
            { name: "verifierName", type: "string" },
            { name: "verifierVersion", type: "string" },
            { name: "verifierChainId", type: "uint256" },
            { name: "verifierContract", type: "address" },
            { name: "verifierSalt", type: "bytes32" },
          ],
          outputs: [
            { name: "typedDataSignTypehash", type: "bytes32" },
            { name: "wrapperStructHash", type: "bytes32" },
            { name: "finalHash", type: "bytes32" },
          ],
        },
      ],
      functionName: "computeTypedDataSignAndFinalHash",
      args: [
        {
          message: erc7739TypedData.message.message,
          value: erc7739TypedData.message.value,
        },
        "TestStruct(string message,uint256 value)",
        "zksync-sso-1271",
        "1.0.0",
        BigInt(contracts.chainId),
        props.accountAddress as Address,
        ("0x" + "00".repeat(32)) as Hex,
      ],
    });

    // JS local hash (ERC-7739 wrapped) — the digest we signed earlier
    const jsTypedData = {
      domain: {
        name: "ERC1271Caller",
        version: "1.0.0",
        chainId: contracts.chainId,
        verifyingContract: erc1271CallerAddress,
      },
      types: erc7739TypedData.types,
      primaryType: erc7739TypedData.primaryType,
      message: erc7739TypedData.message,
    };

    const jsErc7739Hash = hashTypedDataErc7739({
      typedData: jsTypedData,
      smartAccountAddress: props.accountAddress as Hex,
      chainId: contracts.chainId,
    });

    // Log values
    // eslint-disable-next-line no-console
    console.log("[ERC-7739 Debug] onchainStructHash:", onchainStructHash);
    // eslint-disable-next-line no-console
    console.log("[ERC-7739 Debug] onchainDebug (typedDataSignTypehash, wrapperStructHash, finalHash):", onchainDebug);
    // eslint-disable-next-line no-console
    console.log("[ERC-7739 Debug] jsErc7739Hash:", jsErc7739Hash);

    const isValid = await publicClient.readContract({
      address: erc1271CallerAddress as Address,
      abi: [{
        type: "function",
        name: "validateStruct",
        stateMutability: "view",
        inputs: [
          {
            name: "testStruct",
            type: "tuple",
            components: [
              { name: "message", type: "string" },
              { name: "value", type: "uint256" },
            ],
          },
          { name: "signer", type: "address" },
          { name: "signature", type: "bytes" },
        ],
        outputs: [{ name: "", type: "bool" }],
      }] as const,
      functionName: "validateStruct",
      args: [
        {
          message: erc7739TypedData.message.message,
          value: erc7739TypedData.message.value,
        },
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

// Expose actions for E2E bridge triggers
defineExpose({ signErc7739, verifyErc7739 });
</script>
