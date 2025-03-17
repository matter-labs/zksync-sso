<template>
  <p
    v-if="notStarted"
    class="text-center text-neutral-700 dark:text-neutral-300"
  >
    Everything is ready to start your recovery
  </p>
  <p
    v-if="calculatingProof"
    class="text-center text-neutral-700 dark:text-neutral-300"
  >
    You recovery is being prepared. Please don't close the window.
  </p>
  <p
    v-if="proofReady"
    class="text-center text-neutral-700 dark:text-neutral-300"
  >
    Waiting for the transaction to confirm...
  </p>
  <ZkButton
    class="w-full"
    :disabled="calculatingProof"
    @click="go"
  >
    Start
  </ZkButton>
</template>

<script setup lang="ts">
import { useAppKitAccount } from "@reown/appkit/vue";
import { type Address, bytesToBigInt, type Hex, pad, toHex } from "viem";
import { sendTransaction } from "viem/zksync";
import { createNonceV2 } from "zksync-sso-circuits";

const { getWalletClient, defaultChain } = useClientStore();
const { startGoogleOauth } = useGoogleOauth();
const accountData = useAppKitAccount();
const {
  recoveryStep1Calldata,
  zkProofInProgress,
  zkProof,
  generateZkProof,
  hashIssuer,
} = useRecoveryOidc();

type PasskeyData = {
  credentialId: Hex;
  passkeyPubKey: [Hex, Hex];
};

const salt = defineModel<Hex>("salt", { required: true });
const sub = defineModel<string>("sub", { required: true });
const passkey = defineModel<PasskeyData>("passkey", { required: true });
const userAddress = defineModel<Address>("userAddress", { required: true });
// const disabledSteps = defineModel<number[]>("disabledSteps", { required: true });

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
  return !zkProofInProgress.value && zkProof.value;
});

function buildBlindingFactor(): bigint {
  const randomValues = new Uint8Array(31);
  crypto.getRandomValues(randomValues);
  return bytesToBigInt(randomValues);
}

async function go() {
  const client = await getWalletClient({ chainId: defaultChain.id });
  const blindingFactor = buildBlindingFactor();
  const contractNonce = 1n;
  const [hashForCircuitInput, jwtNonce] = createNonceV2(accountData.value.address as Hex, contractNonce, blindingFactor);

  const jwt = await startGoogleOauth(jwtNonce, sub.value);

  if (jwt === undefined) {
    throw new Error("Failed to start GoogleOauth");
  }

  const googleResponse = await fetch("https://www.googleapis.com/oauth2/v3/certs").then((r) => r.json()) as KeysType;
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
    {
      issHash: await hashIssuer(),
      kid: pad(toHex(Buffer.from(key.kid, "hex"))),
    },
    passkey.value.passkeyPubKey,
    userAddress.value,
  );

  const sendTransactionArgs = {
    account: client.account,
    to: contractsByChain[defaultChain.id].recoveryOidc,
    data: calldata,
    gas: 40_000_000,
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
  } as any;
  await sendTransaction(client, sendTransactionArgs);
}
</script>
