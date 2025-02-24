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
        <pre
          v-if="proof !== null"
        >
          {{ proof }}
        </pre>

        <google-recovery-flow />
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { toHex } from "viem";
import { ref } from "vue";
import { createNonce, JwtTxValidationInputs } from "zksync-sso-circuits";

definePageMeta({
  layout: "dashboard",
});
const { buildOidcDigest } = useRecoveryOidc();
const { startGoogleOauth } = useGoogleOauth();
const { snarkJs } = useSnarkJs();

const currentStep = ref(1);
const proof = ref<string | null>(null);
const calculating = ref(false);

const stepTitle = computed(() => {
  switch (currentStep.value) {
    case 1:
      return "Test oidc recovery";
    default:
      throw new Error("Unknown error");
  }
});

async function generateProf(): Promise<void> {
  // TODO: replace with real txHash.
  const txHash = toHex(new Uint8Array(32));
  // TODO: replace with secure blinding factor calculation.
  const blindingFactor = 10n;
  const nonce = createNonce(txHash, blindingFactor);

  const jwt = await startGoogleOauth(nonce);
  if (jwt === undefined) {
    throw new Error("jwt should not be undefined");
  }

  const digest = await buildOidcDigest(jwt);

  type KeysType = {
    keys: {
      n: string;
      kid: string;
    }[];
  };

  const googleResponse = await fetch("https://www.googleapis.com/oauth2/v3/certs").then((r) => r.json()) as KeysType;
  const key = googleResponse.keys.find((key) => key.kid === jwt.kid);

  if (key === undefined) {
    throw new Error("Signed key not found in google exposed keys");
  }

  const inputs = new JwtTxValidationInputs(
    jwt.raw,
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
</script>
