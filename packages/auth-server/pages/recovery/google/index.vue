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
            @click="generateProf"
          >
            Test
          </ZkButton>
        </div>

        <span
          v-if="calculating"
        >
          Calculating...
        </span>
        <pre v-if="proof !== null">{{ proof }}</pre>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { type Address, bytesToBigInt, bytesToHex } from "viem";
import { ref } from "vue";
import { OidcRecoveryModuleAbi } from "zksync-sso/abi";
import { getPublicKeyBytesFromPasskeySignature } from "zksync-sso/utils";
import { createNonce, JwtTxValidationInputs } from "zksync-sso-circuits";

definePageMeta({
  layout: "dashboard",
});
const { buildOidcDigest } = useRecoveryOidc();
const { startGoogleOauth } = useGoogleOauth();
const { snarkJs } = useSnarkJs();
const { getRecoveryClient, defaultChain, getPublicClient } = useClientStore();
const { registerPasskey } = usePasskeyRegister();

const currentStep = ref(1);
const proof = ref<string | null>(null);
const calculating = ref(false);

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

async function generateProf(): Promise<void> {
  const buf = new Uint8Array(16);
  crypto.getRandomValues(buf);
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

  const recoveryClient = getRecoveryClient({ chainId: defaultChain.id, address: addressToRecover });
  const passkeyPubKey = await getNewPasskey();
  const txHash = await recoveryClient.calculateAddKeyTxHash({
    passkeyPubKey: passkeyPubKey,
    passkeyDomain: window.location.origin,
  });
  // TODO: replace with secure blinding factor calculation.
  const blindingFactor = buildBlindingFactor();
  const nonce = createNonce(txHash, blindingFactor);

  const txJwt = await startGoogleOauth(nonce, identityJwt.sub);

  if (txJwt === undefined) {
    throw new Error("jwt should not be undefined");
  }

  const googleResponse = await fetch("https://www.googleapis.com/oauth2/v3/certs").then((r) => r.json()) as KeysType;
  const key = googleResponse.keys.find((key) => key.kid === txJwt.kid);

  if (key === undefined) {
    throw new Error("Signer key not found in google exposed keys");
  }

  const inputs = new JwtTxValidationInputs(
    txJwt.raw,
    key.n,
    digest.salt.toBigInt(),
    txHash,
    blindingFactor,
  );
  calculating.value = true;
  const res = await snarkJs.groth16.fullProve(inputs.toObject(), "/circuit/witness.wasm", "/circuit/circuit.zkey");
  calculating.value = false;
  proof.value = JSON.stringify(res, null, 2);
}

async function getNewPasskey(): Promise<[Buffer, Buffer]> {
  const result = await registerPasskey();
  if (!result) {
    throw new Error("Failed to register passkey");
  }
  const { credentialPublicKey } = result;

  const [buf1, buf2] = getPublicKeyBytesFromPasskeySignature(credentialPublicKey);

  if (buf1 === undefined || buf2 === undefined) {
    throw new Error("Could not recover passkey");
  }

  return [buf1, buf2];
  // const encoded = encodePasskeyModuleParameters({
  //   passkeyPublicKey: getPublicKeyBytesFromPasskeySignature(credentialPublicKey),
  //   expectedOrigin: window.location.origin,
  // });
}
</script>
