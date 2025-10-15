<template>
  <div
    v-if="preflightErrors.length > 0"
    class="mb-4 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg"
  >
    <h3 class="font-semibold text-red-800 dark:text-red-200 mb-2">
      Pre-flight Check Errors
    </h3>
    <ul class="list-disc list-inside space-y-1 text-sm text-red-700 dark:text-red-300">
      <li
        v-for="(error, index) in preflightErrors"
        :key="index"
      >
        {{ error }}
      </li>
    </ul>
  </div>

  <div
    v-if="preflightChecks.length > 0"
    class="mb-4 p-4 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg"
  >
    <h3 class="font-semibold text-blue-800 dark:text-blue-200 mb-2">
      Progress
    </h3>
    <ul class="space-y-1 text-sm text-blue-700 dark:text-blue-300">
      <li
        v-for="(check, index) in preflightChecks"
        :key="index"
        class="flex items-center gap-2"
      >
        <span class="text-green-500">✓</span>
        {{ check }}
      </li>
    </ul>
  </div>

  <p
    v-if="notStarted && !recoverySuccessful"
    class="text-center text-neutral-700 dark:text-neutral-300"
  >
    Everything is ready to start your recovery
  </p>
  <p
    v-if="calculatingProof && !recoverySuccessful"
    class="text-center text-neutral-700 dark:text-neutral-300"
  >
    You recovery is being prepared. Please don't close the window.
  </p>
  <p
    v-if="proofReady && !recoverySuccessful"
    class="text-center text-neutral-700 dark:text-neutral-300"
  >
    Waiting for the transaction to confirm...
  </p>
  <p
    v-if="recoverySuccessful"
    class="text-center text-neutral-700 dark:text-neutral-300"
  >
    Recovery successful. You can now use your new passkey.
  </p>
  <ZkButton
    v-if="!recoverySuccessful"
    class="w-full"
    :disabled="calculatingProof"
    @click="go"
  >
    Start
  </ZkButton>
</template>

<script setup lang="ts">
import type { Address, Hex } from "viem";
import { bytesToBigInt, pad, zeroAddress } from "viem";
import { waitForTransactionReceipt } from "viem/actions";
import { sendTransaction } from "viem/zksync";
import { OidcRecoveryValidatorAbi } from "zksync-sso/abi";
import { createNonceV2 } from "zksync-sso-circuits";

import { GOOGLE_CERTS_URL } from "./constants";

const { getWalletClient, getPublicClient, defaultChain, getOidcClient } = useClientStore();
const { startGoogleOauth } = useGoogleOauth();
const {
  recoveryStep1Calldata,
  hashPasskeyData,
  zkProofInProgress,
  zkProof,
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

const notStarted = computed<boolean>(() => {
  return !zkProofInProgress.value && !zkProof.value;
});

const calculatingProof = computed<boolean>(() => {
  return zkProofInProgress.value && !zkProof.value;
});

const proofReady = computed<boolean>(() => {
  return !zkProofInProgress.value && !!zkProof.value;
});

const recoverySuccessful = ref<boolean>(false);
const preflightChecks = ref<string[]>([]);
const preflightErrors = ref<string[]>([]);

function buildBlindingFactor(): bigint {
  const randomValues = new Uint8Array(31);
  crypto.getRandomValues(randomValues);
  return bytesToBigInt(randomValues);
}

async function go() {
  preflightChecks.value = [];
  preflightErrors.value = [];

  try {
    // ========== PRE-FLIGHT CHECKS ==========
    const publicClient = getPublicClient({ chainId: defaultChain.id });
    const recoveryAddress = contractsByChain[defaultChain.id].recoveryOidc as Address;

    // Check 1: Verify contract addresses are configured
    preflightChecks.value.push("Checking recovery validator configuration...");
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
      throw new Error(`❌ OIDC recovery validator at ${recoveryAddress} is not initialized properly`);
    }

    // Check 2: Verify account has OIDC data registered
    preflightChecks.value.push("Checking account OIDC registration...");
    const oidcData = await getOidcAccounts(userAddress.value);
    if (oidcData === undefined) {
      throw new Error(`❌ Account ${userAddress.value} does not have OIDC data registered. Please register with Google first.`);
    }

    if (!oidcData.oidcDigest || oidcData.oidcDigest === "0x0000000000000000000000000000000000000000000000000000000000000000") {
      throw new Error(`❌ Account ${userAddress.value} has invalid OIDC digest. Please re-register.`);
    }

    if (oidcData.readyToRecover) {
      preflightErrors.value.push("⚠️  Warning: Account already has a pending recovery. You may want to cancel it first.");
    }

    // Check 3: Verify sender has balance for gas
    preflightChecks.value.push("Checking wallet balance...");
    const client = await getWalletClient({ chainId: defaultChain.id });
    const senderAddress = client.account.address as Address;
    const balance = await publicClient.getBalance({ address: senderAddress });

    if (balance === 0n) {
      throw new Error(`❌ Wallet ${senderAddress} has zero balance. Cannot pay for gas.`);
    }

    // Check 4: Pre-fetch Google keys to verify JWT kid will be valid
    preflightChecks.value.push("Fetching current Google signing keys...");
    const googleResponse = await fetch(GOOGLE_CERTS_URL).then((r) => r.json()) as KeysType;

    if (!googleResponse.keys || googleResponse.keys.length === 0) {
      throw new Error("❌ Failed to fetch Google keys. Network issue?");
    }

    // Check 5: Verify Google keys are registered in the key registry
    preflightChecks.value.push("Verifying Google keys are registered...");
    const { OidcKeyRegistryAbi } = await import("zksync-sso/abi");
    const googleIss = "https://accounts.google.com";
    const issHash = await publicClient.readContract({
      address: keyRegistryAddr,
      abi: OidcKeyRegistryAbi,
      functionName: "hashIssuer",
      args: [googleIss],
    }) as Hex;

    const registeredKeys = (await publicClient.readContract({
      address: keyRegistryAddr,
      abi: OidcKeyRegistryAbi,
      functionName: "getKeys",
      args: [issHash],
    })) as unknown as Array<{ kid: Hex; rsaModulus: bigint[] }>;

    const registeredKids = registeredKeys
      .filter((k) => k.kid !== "0x0000000000000000000000000000000000000000000000000000000000000000")
      .map((k) => k.kid.toLowerCase());

    // Check if at least one Google key is registered
    const googleKidsHex = googleResponse.keys.map((key) => {
      const paddedKid = pad(`0x${key.kid.replace(/^0x/, "")}` as Hex);
      return paddedKid.toLowerCase();
    });

    const matchingKeys = googleKidsHex.filter((kid) => registeredKids.includes(kid));

    if (matchingKeys.length === 0) {
      throw new Error("❌ CRITICAL: None of Google's current signing keys are registered in the key registry! Recovery will fail. Please update the key registry.");
    }

    // ========== PREPARE RECOVERY DATA ==========
    preflightChecks.value.push("Preparing recovery parameters...");

    const contractNonce = oidcData.recoverNonce;
    const currentTime = BigInt(new Date().valueOf()) / 1000n;
    const timeLimit = currentTime + 3600n; // 1 hour from now

    const passkeyHash = hashPasskeyData(
      passkey.value.credentialId,
      passkey.value.passkeyPubKey,
      window.location.origin,
    );

    // Generate blinding factor ONCE - must be the same for nonce creation and proof generation
    const blindingFactor = buildBlindingFactor();

    const [hashForCircuitInput, jwtNonce] = createNonceV2(
      senderAddress,
      userAddress.value,
      passkeyHash,
      contractNonce,
      blindingFactor,
      timeLimit,
    );

    // ========== GENERATE JWT ==========
    preflightChecks.value.push("Requesting JWT from Google...");
    const jwt = await startGoogleOauth(jwtNonce, sub.value);

    if (jwt === undefined) {
      throw new Error("❌ Failed to get JWT from Google OAuth");
    }

    // Check 6: Verify JWT kid matches a registered key
    const jwtKidPadded = pad(`0x${jwt.kid.replace(/^0x/, "")}` as Hex).toLowerCase();

    if (!registeredKids.includes(jwtKidPadded)) {
      preflightErrors.value.push(`⚠️  CRITICAL: JWT kid ${jwt.kid} is NOT registered in the key registry!`);
      preflightErrors.value.push("This will cause OIDC_KEY_NOT_FOUND error. Please update the key registry before proceeding.");
      throw new Error(`❌ JWT kid ${jwt.kid} is not registered. Recovery will fail.`);
    }

    const key = googleResponse.keys.find((key) => key.kid === jwt.kid);
    if (key === undefined) {
      throw new Error(`❌ JWT kid ${jwt.kid} not found in Google's current keys`);
    }

    // ========== GENERATE ZK PROOF ==========
    preflightChecks.value.push("Generating zero-knowledge proof (this may take a minute)...");

    const proof = await generateZkProof(
      jwt.raw,
      key.n,
      salt.value,
      hashForCircuitInput,
      blindingFactor,
    );

    if (proof === undefined) {
      throw new Error("❌ Failed to generate ZK proof");
    }

    // ========== PREPARE TRANSACTION ==========
    preflightChecks.value.push("Preparing recovery transaction...");

    const calldata = recoveryStep1Calldata(
      proof,
      pad(`0x${key.kid.replace(/^0x/, "")}` as Hex),
      passkeyHash,
      userAddress.value,
      timeLimit,
    );

    // ========== SEND TRANSACTION ==========
    preflightChecks.value.push("Sending recovery transaction...");

    const sendTransactionArgs = {
      account: client.account,
      to: recoveryAddress,
      data: calldata,
      value: 0n,
      gas: 20_000_000n,
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    } as any;

    const sentTx = await sendTransaction(client, sendTransactionArgs);

    preflightChecks.value.push("Waiting for transaction confirmation...");
    const startRecoveryReceipt = await waitForTransactionReceipt(client, { hash: sentTx, confirmations: 1 });

    if (startRecoveryReceipt.status !== "success") {
      throw new Error(`❌ Recovery transaction failed with status: ${startRecoveryReceipt.status}`);
    }

    // ========== ADD NEW PASSKEY ==========
    preflightChecks.value.push("Adding new passkey to account...");
    const oidcClient = getOidcClient({ chainId: defaultChain.id, address: userAddress.value });

    const addedPasskey = await oidcClient.addNewPasskeyViaOidc({
      credentialId: passkey.value.credentialId,
      passkeyPubKey: passkey.value.passkeyPubKey,
      passkeyDomain: window.location.origin,
    });

    if (addedPasskey.status !== "success") {
      throw new Error(`❌ Failed to add passkey: ${addedPasskey.status}`);
    }

    recoverySuccessful.value = true;
  } catch (error) {
    preflightErrors.value.push(error instanceof Error ? error.message : String(error));
    throw error;
  }
}
</script>
