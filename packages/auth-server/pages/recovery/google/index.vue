<template>
  <div class="min-h-screen">
    <header class="max-w-[1920px] mx-auto mb-12">
      <app-generic-nav />
    </header>
    <main class="max-w-[900px] mx-auto flex flex-col gap-6">
      <CommonStepper
        :current-step="currentStep"
        :total-steps="3"
      />

      <div class="flex flex-col items-center gap-8 mt-4">
        <h1 class="text-3xl font-medium text-neutral-900 dark:text-neutral-100">
          {{ stepTitle }}
        </h1>
        <div
          v-if="currentStep === 1"
          class="w-full max-w-md flex flex-col gap-6"
        >
          <ZkButton
            class="w-full"
            @click="tryToGenerateProof"
          >
            Test
          </ZkButton>
        </div>
        <pre>{{ guideText }}</pre>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { type Address, bytesToBigInt, bytesToHex, type Hex, toHex } from "viem";
import { ref } from "vue";
import { OidcRecoveryModuleAbi } from "zksync-sso/abi";
import { base64UrlToUint8Array, getPublicKeyBytesFromPasskeySignature } from "zksync-sso/utils";
import { createNonce, JwtTxValidationInputs } from "zksync-sso-circuits";

definePageMeta({
  layout: "dashboard",
});
const { buildOidcDigest } = useRecoveryOidc();
const { startGoogleOauth } = useGoogleOauth();
const { snarkjs } = useSnarkJs();
const { defaultChain, getPublicClient, getOidcClient } = useClientStore();
const { registerPasskey } = usePasskeyRegister();
const { getOidcKeys, getOidcKeysData, getMerkleProof } = useOidcKeys();

const currentStep = ref(1);
const guideText = ref<string>("Starting");

type KeysType = {
  keys: {
    n: string;
    kid: string;
  }[];
};

const stepTitle = computed(() => {
  switch (currentStep.value) {
    case 1:
      return "Test oidc recovery";
    default:
      throw new Error("Unknown error");
  }
});

function buildBlindingFactor(): bigint {
  const randomValues = new Uint8Array(31);
  crypto.getRandomValues(randomValues);
  return bytesToBigInt(randomValues);
}

async function tryToGenerateProof() {
  try {
    await generateProf();
  } catch (e) {
    if (e instanceof Error) {
      guide(`An error ocurred: ${e.message}`);
    } else {
      guide("An unknown error happened");
    }
  }
}

function guide(text: string) {
  guideText.value = (text);
  console.log(text);
}

async function generateProf(): Promise<void> {
  const buf = new Uint8Array(16);
  crypto.getRandomValues(buf);
  guide("Looking for address...");
  const identityJwt = await startGoogleOauth(bytesToHex(buf));

  if (identityJwt === undefined) {
    throw new Error("jwt should be defined");
  }

  const digest = await buildOidcDigest(identityJwt);
  const publicClient = getPublicClient({ chainId: defaultChain.id });

  // recover address
  const addressToRecover = await publicClient.readContract({
    address: contractsByChain[defaultChain.id].recoveryOidc,
    abi: OidcRecoveryModuleAbi,
    functionName: "addressForDigest",
    args: [digest.toHex()],
  }) as Address;

  guide("Generating signature jwt...");
  const oidcClient = getOidcClient({ chainId: defaultChain.id, address: addressToRecover });

  guide("Asking for new passkey...");
  const { credentialId, passkeyPubKey } = await getNewPasskey();
  const txHash = await oidcClient.calculateTxHash({
    credentialId,
    passkeyPubKey,
    passkeyDomain: window.location.origin,
  });

  const blindingFactor = buildBlindingFactor();
  const nonce = createNonce(txHash, blindingFactor);

  const txJwt = await startGoogleOauth(nonce, identityJwt.sub);

  if (txJwt === undefined) {
    throw new Error("jwt should not be undefined");
  }

  guide("Getting oidc keys...");
  await getOidcKeys();
  const oidcKey = getOidcKeysData.value?.find((key) =>
    key.kid.replace("0x", "").replace(/^0+/, "") === txJwt.kid,
  );
  if (oidcKey === undefined) {
    throw new Error("Signer key not found in oidc keys");
  }

  const merkleProof = getMerkleProof(oidcKey);

  const googleResponse = await fetch("https://www.googleapis.com/oauth2/v3/certs").then((r) => r.json()) as KeysType;
  const key = googleResponse.keys.find((key) => key.kid === txJwt.kid);

  if (key === undefined) {
    throw new Error("Signer key not found in google exposed keys");
  }

  const inputs = new JwtTxValidationInputs(
    txJwt.raw,
    key.n,
    digest.salt.toHex(),
    txHash,
    blindingFactor,
  );

  guide("Generating proof...");
  const res = await snarkjs.groth16.fullProve(inputs.toObject(), "/circuit/witness.wasm", "/circuit/circuit.zkey", console);

  guide("Adding proof...");
  oidcClient.account.addProof({
    public: res.publicSignals.map((str) => BigInt(str)),
    groth16Proof: res.proof,
    txHash,
    oidcKey,
    merkleProof: merkleProof.map((proof) => `${proof}` as Hex),
  });

  guide("Broadcasting tx...");
  await oidcClient.addNewPasskeyViaOidc({
    credentialId,
    passkeyPubKey,
    passkeyDomain: window.location.origin,
  });
  guide("Success!");
}

type PasskeyData = {
  credentialId: Hex;
  passkeyPubKey: [Hex, Hex];
};

async function getNewPasskey(): Promise<PasskeyData> {
  const result = await registerPasskey();
  if (!result) {
    throw new Error("Failed to register passkey");
  }
  const { credentialPublicKey, credentialId } = result;

  const [buf1, buf2] = getPublicKeyBytesFromPasskeySignature(credentialPublicKey);

  if (buf1 === undefined || buf2 === undefined) {
    throw new Error("Could not recover passkey");
  }

  return {
    credentialId: toHex(base64UrlToUint8Array(credentialId)),
    passkeyPubKey: [bytesToHex(buf1), bytesToHex(buf2)],
  };
}
</script>
