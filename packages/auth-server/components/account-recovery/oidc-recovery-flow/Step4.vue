<template>
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
import { useAppKitAccount } from "@reown/appkit/vue";
import type { Address, Hex } from "viem";
import { bytesToBigInt, pad, zeroAddress } from "viem";
import { waitForTransactionReceipt } from "viem/actions";
import { sendTransaction } from "viem/zksync";
import { OidcRecoveryValidatorAbi } from "zksync-sso/abi";
import { createNonceV2 } from "zksync-sso-circuits";

import { GOOGLE_CERTS_URL } from "./constants";

const { getWalletClient, getPublicClient, defaultChain, getOidcClient } = useClientStore();
const { startGoogleOauth } = useGoogleOauth();
const accountData = useAppKitAccount();
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

function buildBlindingFactor(): bigint {
  const randomValues = new Uint8Array(31);
  crypto.getRandomValues(randomValues);
  return bytesToBigInt(randomValues);
}

async function go() {
  const client = await getWalletClient({ chainId: defaultChain.id });
  const publicClient = getPublicClient({ chainId: defaultChain.id });
  const blindingFactor = buildBlindingFactor();
  const oidcData = await getOidcAccounts(userAddress.value);
  if (oidcData === undefined) {
    throw new Error("Could not find OIDC data");
  }

  const contractNonce = oidcData.recoverNonce;
  const currentTime = BigInt(new Date().valueOf()) / 1000n; // convert to seconds:wq
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

  const jwt = await startGoogleOauth(jwtNonce, sub.value);

  if (jwt === undefined) {
    throw new Error("Failed to start GoogleOauth");
  }

  const googleResponse = await fetch(GOOGLE_CERTS_URL).then((r) => r.json()) as KeysType;
  const key = googleResponse.keys.find((key) => key.kid === jwt.kid);

  if (key === undefined) {
    throw new Error("Signer key not found in google exposed keys");
  }

  const proof = await generateZkProof(
    jwt.raw,
    key.n,
    salt.value,
    hashForCircuitInput,
    blindingFactor,
  );

  if (proof === undefined) {
    throw new Error("`proof` should be defined");
  }

  const calldata = recoveryStep1Calldata(
    proof,
    pad(`0x${key.kid.replace(/^0x/, "")}` as Hex),
    passkeyHash,
    userAddress.value,
    timeLimit,
  );

  // Preflight checks
  const recoveryAddress = contractsByChain[defaultChain.id].recoveryOidc as Address;
  const senderAddress = client.account.address as Address;

  // Ensure OIDC validator is initialized and wired correctly
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
    throw new Error(`OIDC recovery validator at ${recoveryAddress} is not initialized`);
  }

  const expectedPasskey = contractsByChain[defaultChain.id].passkey as Address | undefined;
  if (expectedPasskey && webAuthValidatorAddr.toLowerCase() !== expectedPasskey.toLowerCase()) {
    throw new Error(`webAuthValidator mismatch: on-chain=${webAuthValidatorAddr}, expected=${expectedPasskey}`);
  }

  // Ensure sender has some balance for gas
  const balance = await publicClient.getBalance({ address: senderAddress });
  if (balance === 0n) {
    throw new Error("Insufficient balance to pay gas for recovery transaction");
  }

  const sendTransactionArgs = {
    account: client.account,
    to: contractsByChain[defaultChain.id].recoveryOidc,
    data: calldata,
    value: 0n,
    gas: 20_000_000n,
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
  } as any;

  const sentTx = await sendTransaction(client, sendTransactionArgs);

  const startRecoveryReceipt = await waitForTransactionReceipt(client, { hash: sentTx, confirmations: 1 });
  if (startRecoveryReceipt.status !== "success") {
    throw new Error(`Recovery transaction ${startRecoveryReceipt.status}`);
  }
  const oidcClient = getOidcClient({ chainId: defaultChain.id, address: userAddress.value });

  const addedPasskey = await oidcClient.addNewPasskeyViaOidc({
    credentialId: passkey.value.credentialId,
    passkeyPubKey: passkey.value.passkeyPubKey,
    passkeyDomain: window.location.origin,
  });

  if (addedPasskey.status !== "success") {
    throw new Error(`Failed to add passkey via OIDC: ${addedPasskey.status}`);
  }

  recoverySuccessful.value = true;
}
</script>
