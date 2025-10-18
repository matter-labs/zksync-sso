<template>
  <div class="space-y-4">
    <!-- Progress Overview -->
    <div
      v-if="currentStep > 0"
      class="rounded-lg border border-neutral-200 dark:border-neutral-700 bg-neutral-50 dark:bg-neutral-800 p-4"
    >
      <div class="space-y-2">
        <div
          v-for="step in recoverySteps"
          :key="step.id"
          class="flex items-start gap-3"
        >
          <div class="flex-shrink-0 mt-0.5">
            <div
              v-if="step.id < currentStep"
              class="h-5 w-5 rounded-full bg-green-500 flex items-center justify-center"
            >
              <svg
                class="h-3 w-3 text-white"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="3"
                  d="M5 13l4 4L19 7"
                />
              </svg>
            </div>
            <div
              v-else-if="step.id === currentStep"
              class="h-5 w-5 rounded-full border-2 border-blue-500 bg-white dark:bg-neutral-900 flex items-center justify-center"
            >
              <div class="h-2 w-2 rounded-full bg-blue-500 animate-pulse" />
            </div>
            <div
              v-else
              class="h-5 w-5 rounded-full border-2 border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-900"
            />
          </div>
          <div class="flex-1 min-w-0">
            <p
              class="text-sm font-medium"
              :class="{
                'text-green-600 dark:text-green-400': step.id < currentStep,
                'text-blue-600 dark:text-blue-400': step.id === currentStep,
                'text-neutral-500 dark:text-neutral-400': step.id > currentStep,
              }"
            >
              {{ step.label }}
            </p>
            <p
              v-if="step.id === currentStep && step.description"
              class="text-xs text-neutral-600 dark:text-neutral-400 mt-0.5"
            >
              {{ step.description }}
            </p>
          </div>
          <div
            v-if="step.id === currentStep"
            class="flex-shrink-0"
          >
            <div class="flex space-x-1">
              <div
                class="h-1.5 w-1.5 bg-blue-500 rounded-full animate-bounce"
                style="animation-delay: 0ms"
              />
              <div
                class="h-1.5 w-1.5 bg-blue-500 rounded-full animate-bounce"
                style="animation-delay: 150ms"
              />
              <div
                class="h-1.5 w-1.5 bg-blue-500 rounded-full animate-bounce"
                style="animation-delay: 300ms"
              />
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Error Display -->
    <div
      v-if="errorMessage"
      class="rounded-lg border border-red-200 dark:border-red-800 bg-red-50 dark:bg-red-900/20 p-4"
    >
      <div class="flex items-start gap-3">
        <svg
          class="h-5 w-5 text-red-600 dark:text-red-400 flex-shrink-0 mt-0.5"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
          />
        </svg>
        <div class="flex-1 min-w-0">
          <h4 class="text-sm font-semibold text-red-800 dark:text-red-200">
            Recovery Failed
          </h4>
          <p class="text-sm text-red-700 dark:text-red-300 mt-1">
            {{ errorMessage }}
          </p>
          <button
            v-if="debugInfo"
            class="text-xs text-red-600 dark:text-red-400 underline mt-2"
            @click="showDebug = !showDebug"
          >
            {{ showDebug ? "Hide" : "Show" }} Debug Info
          </button>
        </div>
      </div>
      <div
        v-if="showDebug && debugInfo"
        class="mt-3 pt-3 border-t border-red-200 dark:border-red-800"
      >
        <pre class="text-xs text-red-800 dark:text-red-200 overflow-x-auto p-2 bg-white dark:bg-neutral-900 rounded">{{ debugInfo }}</pre>
        <button
          class="text-xs text-red-600 dark:text-red-400 underline mt-2"
          @click="copyDebugInfo"
        >
          Copy to Clipboard
        </button>
      </div>
    </div>

    <!-- Success Message -->
    <div
      v-if="recoverySuccessful"
      class="rounded-lg border border-green-200 dark:border-green-800 bg-green-50 dark:bg-green-900/20 p-4 text-center"
    >
      <div class="flex justify-center mb-3">
        <div class="h-12 w-12 rounded-full bg-green-500 flex items-center justify-center">
          <svg
            class="h-6 w-6 text-white"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M5 13l4 4L19 7"
            />
          </svg>
        </div>
      </div>
      <h3 class="text-lg font-semibold text-green-800 dark:text-green-200">
        Recovery Successful!
      </h3>
      <p class="text-sm text-green-700 dark:text-green-300 mt-1">
        You can now use your new passkey to access your account.
      </p>
      <div
        v-if="transactionHash"
        class="mt-3 pt-3 border-t border-green-200 dark:border-green-800"
      >
        <p class="text-xs text-green-700 dark:text-green-300">
          Transaction Hash:
        </p>
        <a
          :href="`https://sepolia.explorer.zksync.io/tx/${transactionHash}`"
          target="_blank"
          class="text-xs text-green-600 dark:text-green-400 hover:underline font-mono break-all"
        >
          {{ transactionHash }}
        </a>
      </div>
    </div>

    <!-- Start/Retry Button -->
    <ZkButton
      v-if="!recoverySuccessful"
      class="w-full"
      :disabled="isProcessing"
      @click="errorMessage ? retry() : go()"
    >
      {{ errorMessage ? "Retry Recovery" : currentStep > 0 ? "Processing..." : "Start Recovery" }}
    </ZkButton>

    <!-- Ready State Message -->
    <p
      v-if="notStarted && !recoverySuccessful && !errorMessage"
      class="text-center text-sm text-neutral-600 dark:text-neutral-400"
    >
      Everything is ready. Click the button above to begin account recovery.
    </p>
  </div>
</template>

<script setup lang="ts">
import { useAppKitAccount } from "@reown/appkit/vue";
import type { Address, Hex } from "viem";
import { bytesToBigInt, encodeAbiParameters, keccak256, pad, zeroAddress } from "viem";
import { waitForTransactionReceipt } from "viem/actions";
import { sendTransaction } from "viem/zksync";
import { OidcRecoveryValidatorAbi } from "zksync-sso/abi";
import { createNonceV2 } from "zksync-sso-circuits";

import { GOOGLE_CERTS_URL, GOOGLE_ISS } from "./constants";

const { getWalletClient, getPublicClient, defaultChain, getOidcClient } = useClientStore();
const { startGoogleOauth } = useGoogleOauth();
const accountData = useAppKitAccount();
const {
  recoveryStep1Calldata,
  hashPasskeyData,
  generateZkProof,
  getOidcAccounts,
} = useRecoveryOidc();

type PasskeyData = {
  credentialId: Hex;
  passkeyPubKey: [Hex, Hex];
};

const salt = defineModel<Hex>("salt", { required: true });
const sub = defineModel<string>("sub", { required: true });
const passkey = defineModel<PasskeyData>("passkey", { required: true });
const userAddress = defineModel<Address>("userAddress", { required: true });

type KeysType = {
  keys: {
    n: string;
    kid: string;
  }[];
};

// Recovery progress tracking
const currentStep = ref<number>(0);
const recoverySuccessful = ref<boolean>(false);
const errorMessage = ref<string>("");
const debugInfo = ref<string>("");
const showDebug = ref<boolean>(false);
const transactionHash = ref<string>("");

const recoverySteps = [
  { id: 1, label: "Preparing Recovery", description: "Calculating nonce and generating parameters" },
  { id: 2, label: "Google Authentication", description: "Signing in with Google to get JWT token" },
  { id: 3, label: "Fetching Google Keys", description: "Retrieving Google's public signing keys" },
  { id: 4, label: "Generating ZK Proof", description: "Creating zero-knowledge proof (this may take 30-60 seconds)" },
  { id: 5, label: "Validating Contracts", description: "Checking OIDC recovery validator configuration" },
  { id: 6, label: "Submitting Transaction", description: "Broadcasting recovery transaction to blockchain" },
  { id: 7, label: "Confirming Transaction", description: "Waiting for transaction confirmation" },
  { id: 8, label: "Adding Passkey", description: "Finalizing passkey addition to your account" },
];

const notStarted = computed<boolean>(() => {
  return currentStep.value === 0 && !recoverySuccessful.value;
});

const isProcessing = computed<boolean>(() => {
  return currentStep.value > 0 && !recoverySuccessful.value && !errorMessage.value;
});

function buildBlindingFactor(): bigint {
  const randomValues = new Uint8Array(31);
  crypto.getRandomValues(randomValues);
  return bytesToBigInt(randomValues);
}

function formatDebugInfo(step: string, error: unknown, context?: Record<string, unknown>): string {
  const timestamp = new Date().toISOString();
  const debugObj = {
    timestamp,
    step,
    error: error instanceof Error ? error.message : String(error),
    errorStack: error instanceof Error ? error.stack : undefined,
    context,
    contracts: {
      recoveryOidc: contractsByChain[defaultChain.id].recoveryOidc,
      oidcKeyRegistry: contractsByChain[defaultChain.id].oidcKeyRegistry,
      oidcVerifier: contractsByChain[defaultChain.id].oidcVerifier,
      passkey: contractsByChain[defaultChain.id].passkey,
    },
    chainId: defaultChain.id,
    userAddress: userAddress.value,
    accountAddress: accountData.value.address,
    googleCertsUrl: GOOGLE_CERTS_URL,
    zkProofDebugging: {
      note: "If ZK proof verification failed (0xaf14037c):",
      possibleCauses: [
        "JWT nonce doesn't match expected value",
        "Google key not registered in Key Registry",
        "Blinding factor mismatch",
        "Time limit expired",
        "Circuit input hash mismatch",
        "ZK verifier contract issue",
      ],
      verificationSteps: [
        "1. Check JWT nonce matches the one generated from createNonceV2",
        "2. Verify Google key (kid) is registered: call keyRegistry.getKey(issuerHash, kid)",
        "3. Ensure blinding factor used in proof matches the one in nonce",
        "4. Confirm timeLimit hasn't expired",
        "5. Verify hashForCircuitInput matches the JWT sub claim hash",
        "6. Check verifier contract is correctly deployed and configured",
      ],
    },
  };
  return JSON.stringify(debugObj, null, 2);
}

function copyDebugInfo() {
  if (debugInfo.value) {
    navigator.clipboard.writeText(debugInfo.value);
  }
}

function retry() {
  currentStep.value = 0;
  errorMessage.value = "";
  debugInfo.value = "";
  showDebug.value = false;
  transactionHash.value = "";
}

async function go() {
  try {
    errorMessage.value = "";
    debugInfo.value = "";
    transactionHash.value = "";

    // Step 1: Prepare Recovery
    currentStep.value = 1;
    const client = await getWalletClient({ chainId: defaultChain.id });
    const publicClient = getPublicClient({ chainId: defaultChain.id });
    const blindingFactor = buildBlindingFactor();
    const oidcData = await getOidcAccounts(userAddress.value);

    if (oidcData === undefined) {
      throw new Error("Could not find OIDC data for this account. Ensure the account has OIDC recovery enabled.");
    }

    const contractNonce = oidcData.recoverNonce;
    const currentTime = BigInt(new Date().valueOf()) / 1000n;
    const timeLimit = currentTime + 3600n; // 1 hour from now

    const passkeyHash = hashPasskeyData(
      passkey.value.credentialId,
      passkey.value.passkeyPubKey,
      window.location.origin,
    );

    const [hashForCircuitInput, jwtNonce] = createNonceV2(
      accountData.value.address as Hex,
      userAddress.value,
      passkeyHash,
      contractNonce,
      blindingFactor,
      timeLimit,
    );

    // Step 2: Google Authentication
    currentStep.value = 2;
    const jwt = await startGoogleOauth(jwtNonce, sub.value);

    if (jwt === undefined) {
      throw new Error("Google OAuth failed. Please ensure you're signed in with the correct Google account.");
    }

    // Step 3: Fetch Google Keys
    currentStep.value = 3;
    const googleResponse = await fetch(GOOGLE_CERTS_URL).then((r) => r.json()) as KeysType;
    const key = googleResponse.keys.find((key) => key.kid === jwt.kid);

    if (key === undefined) {
      throw new Error(`Signer key (kid: ${jwt.kid}) not found in Google's public keys. This may indicate a key rotation issue.`);
    }

    // Step 4: Generate ZK Proof (this is the slow step)
    currentStep.value = 4;
    const proof = await generateZkProof(
      jwt.raw,
      key.n,
      salt.value,
      hashForCircuitInput,
      blindingFactor,
    );

    if (proof === undefined) {
      throw new Error("Failed to generate zero-knowledge proof. The circuit computation may have failed.");
    }

    const calldata = recoveryStep1Calldata(
      proof,
      pad(`0x${key.kid.replace(/^0x/, "")}` as Hex),
      passkeyHash,
      userAddress.value,
      timeLimit,
    );

    // Step 5: Validate Contracts
    currentStep.value = 5;
    const recoveryAddress = contractsByChain[defaultChain.id].recoveryOidc as Address;
    const senderAddress = client.account.address as Address;

    const [webAuthValidatorAddr, keyRegistryAddr, verifierAddr] = await Promise.all([
      publicClient.readContract({ address: recoveryAddress, abi: OidcRecoveryValidatorAbi, functionName: "webAuthValidator", args: [] }),
      publicClient.readContract({ address: recoveryAddress, abi: OidcRecoveryValidatorAbi, functionName: "keyRegistry", args: [] }),
      publicClient.readContract({ address: recoveryAddress, abi: OidcRecoveryValidatorAbi, functionName: "verifier", args: [] }),
    ]) as [Address, Address, Address];

    if (
      webAuthValidatorAddr === zeroAddress
      || keyRegistryAddr === zeroAddress
      || verifierAddr === zeroAddress
    ) {
      throw new Error(`OIDC recovery validator at ${recoveryAddress} is not properly initialized. Please contact support.`);
    }

    const expectedPasskey = contractsByChain[defaultChain.id].passkey as Address | undefined;
    if (expectedPasskey && webAuthValidatorAddr.toLowerCase() !== expectedPasskey.toLowerCase()) {
      throw new Error(`WebAuth validator mismatch. Expected: ${expectedPasskey}, Found: ${webAuthValidatorAddr}`);
    }

    const balance = await publicClient.getBalance({ address: senderAddress });
    if (balance === 0n) {
      throw new Error(`Insufficient balance to pay gas. Your wallet (${senderAddress}) has 0 ETH.`);
    }

    // Validate Google key is registered in the key registry
    const keyRegistryAbi = [
      {
        inputs: [
          { name: "issuerHash", type: "bytes32" },
          { name: "kid", type: "bytes32" },
        ],
        name: "getKey",
        outputs: [
          {
            components: [
              { name: "rsaModulus", type: "uint256[17]" },
              { name: "isValid", type: "bool" },
            ],
            name: "",
            type: "tuple",
          },
        ],
        stateMutability: "view",
        type: "function",
      },
    ];

    // Calculate Google issuer hash: keccak256(abi.encode("https://accounts.google.com"))
    // This matches the on-chain OidcKeyRegistry.hashIssuer() function
    const googleIssuerHash = keccak256(encodeAbiParameters([{ type: "string" }], [GOOGLE_ISS]));
    const kidBytes32 = pad(`0x${key.kid.replace(/^0x/, "")}` as Hex);

    try {
      const registeredKey = await publicClient.readContract({
        address: keyRegistryAddr,
        abi: keyRegistryAbi,
        functionName: "getKey",
        args: [googleIssuerHash, kidBytes32],
      }) as { rsaModulus: bigint[]; isValid: boolean };

      if (!registeredKey.isValid) {
        throw new Error(
          `Google signing key (kid: ${key.kid}) is not registered in the OIDC Key Registry. `
          + `This key may have been rotated by Google. Registry: ${keyRegistryAddr}`,
        );
      }
    } catch (keyCheckError) {
      throw new Error(
        `Failed to verify Google key registration: ${keyCheckError instanceof Error ? keyCheckError.message : String(keyCheckError)}. `
        + `Key Registry: ${keyRegistryAddr}, Kid: ${key.kid}`,
      );
    }

    // Pre-flight simulation to catch revert early
    try {
      await publicClient.simulateContract({
        address: recoveryAddress,
        abi: OidcRecoveryValidatorAbi,
        functionName: "startRecoveryFlow",
        args: [calldata],
        account: senderAddress,
      });
    } catch (simulationError: unknown) {
      const errorStr = simulationError instanceof Error ? simulationError.message : String(simulationError);

      // Check for ZKP verification failure
      if (errorStr.includes("0xaf14037c") || errorStr.includes("OIDC_ZKP_VERIFICATION_FAILED")) {
        throw new Error(
          "ZK Proof verification failed. This typically means:\n"
          + "1. The JWT signature doesn't match Google's public key\n"
          + "2. The nonce in the JWT doesn't match the expected value\n"
          + "3. The proof was generated with incorrect parameters\n"
          + "4. The ZK verifier contract has issues\n\n"
          + "Debug Info:\n"
          + `- JWT Kid: ${key.kid}\n`
          + `- Key Registry: ${keyRegistryAddr}\n`
          + `- Verifier: ${verifierAddr}\n`
          + `- Nonce: ${jwtNonce}\n`
          + `- Blinding Factor: ${blindingFactor.toString()}\n`
          + `- Time Limit: ${timeLimit.toString()}\n`
          + `- Contract Nonce: ${contractNonce.toString()}\n`
          + `- Passkey Hash: ${passkeyHash}\n`
          + `- Hash for Circuit: ${hashForCircuitInput}\n\n`
          + `Raw Error: ${errorStr}`,
        );
      }

      throw new Error(`Transaction simulation failed: ${errorStr}`);
    }

    // Step 6: Submit Transaction
    currentStep.value = 6;
    const sendTransactionArgs = {
      account: client.account,
      to: contractsByChain[defaultChain.id].recoveryOidc,
      data: calldata,
      value: 0n,
      gas: 20_000_000n,
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
    } as any;

    let sentTx: Hex;
    try {
      sentTx = await sendTransaction(client, sendTransactionArgs);
      transactionHash.value = sentTx;
    } catch (txError: unknown) {
      const errorStr = txError instanceof Error ? txError.message : String(txError);

      // Check for ZKP verification failure in transaction
      if (errorStr.includes("0xaf14037c") || errorStr.includes("OIDC_ZKP_VERIFICATION_FAILED")) {
        throw new Error(
          "ZK Proof verification failed during transaction submission.\n\n"
          + "Proof Details:\n"
          + `- Proof Length: ${proof.length} bytes\n`
          + `- JWT Kid: ${key.kid}\n`
          + "- JWT Issuer: https://accounts.google.com\n"
          + `- Time Limit: ${new Date(Number(timeLimit) * 1000).toISOString()}\n\n`
          + "Contract Addresses:\n"
          + `- Recovery OIDC: ${recoveryAddress}\n`
          + `- Key Registry: ${keyRegistryAddr}\n`
          + `- Verifier: ${verifierAddr}\n\n`
          + "Please copy the debug info below and contact support.\n\n"
          + `Raw Error: ${errorStr}`,
        );
      }

      throw new Error(`Transaction submission failed: ${errorStr}`);
    }

    // Step 7: Confirm Transaction
    currentStep.value = 7;
    const startRecoveryReceipt = await waitForTransactionReceipt(client, { hash: sentTx, confirmations: 1 });

    if (startRecoveryReceipt.status !== "success") {
      // Try to get revert reason from receipt
      let revertReason = "Unknown";
      if (startRecoveryReceipt.logs) {
        try {
          const txDetails = await publicClient.getTransaction({ hash: sentTx });
          revertReason = JSON.stringify(txDetails, null, 2);
        } catch {
          // Ignore error getting transaction details
        }
      }

      throw new Error(
        `Recovery transaction failed with status: ${startRecoveryReceipt.status}\n\n`
        + `Transaction Hash: ${sentTx}\n`
        + `Explorer: https://sepolia.explorer.zksync.io/tx/${sentTx}\n\n`
        + "This may indicate:\n"
        + "- ZK proof verification failed (0xaf14037c)\n"
        + "- Nonce mismatch\n"
        + "- Invalid passkey hash\n"
        + "- Contract state issue\n\n"
        + `Transaction Details: ${revertReason}`,
      );
    }

    // Step 8: Add Passkey
    currentStep.value = 8;
    const oidcClient = getOidcClient({ chainId: defaultChain.id, address: userAddress.value });

    const addedPasskey = await oidcClient.addNewPasskeyViaOidc({
      credentialId: passkey.value.credentialId,
      passkeyPubKey: passkey.value.passkeyPubKey,
      passkeyDomain: window.location.origin,
    });

    if (addedPasskey.status !== "success") {
      throw new Error(`Failed to add passkey via OIDC. Status: ${addedPasskey.status}`);
    }

    recoverySuccessful.value = true;
    currentStep.value = 9; // All done
  } catch (error: unknown) {
    const stepName = recoverySteps[currentStep.value - 1]?.label || "Unknown Step";
    errorMessage.value = error instanceof Error ? error.message : String(error);

    // Build comprehensive debug context
    const debugContext: Record<string, unknown> = {
      currentStepId: currentStep.value,
      currentStepName: stepName,
      salt: salt.value,
      sub: sub.value,
      userAddress: userAddress.value,
      accountAddress: accountData.value.address,
      transactionHash: transactionHash.value || "Not submitted yet",
    };

    // Add ZK-specific debugging if we got far enough
    if (currentStep.value >= 2) {
      try {
        const oidcData = await getOidcAccounts(userAddress.value);
        if (oidcData) {
          debugContext.zkProofContext = {
            contractNonce: oidcData.recoverNonce?.toString(),
            passkeyHash: hashPasskeyData(
              passkey.value.credentialId,
              passkey.value.passkeyPubKey,
              window.location.origin,
            ),
            credentialId: passkey.value.credentialId,
            passkeyPubKey: passkey.value.passkeyPubKey,
            origin: window.location.origin,
          };
        }
      } catch {
        // Ignore errors in debug context collection
      }
    }

    // Add JWT details if we got the JWT
    if (currentStep.value >= 3) {
      try {
        const googleResponse = await fetch(GOOGLE_CERTS_URL).then((r) => r.json()) as KeysType;
        debugContext.googleKeys = {
          availableKids: googleResponse.keys.map((k) => k.kid),
          certsUrl: GOOGLE_CERTS_URL,
        };
      } catch {
        // Ignore errors in debug context collection
      }
    }

    // Add contract validation details if we got there
    if (currentStep.value >= 5) {
      const recoveryAddress = contractsByChain[defaultChain.id].recoveryOidc as Address;
      const publicClient = getPublicClient({ chainId: defaultChain.id });

      try {
        const [webAuthValidatorAddr, keyRegistryAddr, verifierAddr] = await Promise.all([
          publicClient.readContract({ address: recoveryAddress, abi: OidcRecoveryValidatorAbi, functionName: "webAuthValidator", args: [] }),
          publicClient.readContract({ address: recoveryAddress, abi: OidcRecoveryValidatorAbi, functionName: "keyRegistry", args: [] }),
          publicClient.readContract({ address: recoveryAddress, abi: OidcRecoveryValidatorAbi, functionName: "verifier", args: [] }),
        ]) as [Address, Address, Address];

        debugContext.onChainValidation = {
          recoveryOidcAddress: recoveryAddress,
          webAuthValidator: webAuthValidatorAddr,
          keyRegistry: keyRegistryAddr,
          verifier: verifierAddr,
          expectedPasskey: contractsByChain[defaultChain.id].passkey,
          expectedKeyRegistry: contractsByChain[defaultChain.id].oidcKeyRegistry,
        };
      } catch {
        // Ignore errors in debug context collection
      }
    }

    debugInfo.value = formatDebugInfo(stepName, error, debugContext);

    // eslint-disable-next-line no-console
    console.error("Recovery failed:", error);
    // eslint-disable-next-line no-console
    console.error("Debug info:", debugInfo.value);
    // eslint-disable-next-line no-console
    console.error("Full context:", debugContext);
  }
}
</script>
