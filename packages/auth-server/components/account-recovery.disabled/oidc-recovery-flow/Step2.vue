<template>
  <p class="text-center text-neutral-700 dark:text-neutral-300">
    Log in with google to start the recover process
  </p>
  <ZkButton
    class="w-full"
    @click="findAddressUsingGoogleData"
  >
    Log in
  </ZkButton>
</template>

<script setup lang="ts">
import { type Address, bytesToHex } from "viem";
import { OidcRecoveryValidatorAbi } from "zksync-sso/abi";
import type { OidcDigest } from "zksync-sso-circuits";

const { startGoogleOauth } = useGoogleOauth();
const { buildOidcDigest } = useRecoveryOidc();
const { defaultChain, getPublicClient } = useClientStore();

const emit = defineEmits<{
  (e: "done", userAddress: Address, digest: OidcDigest, sub: string): void;
}>();

// At this stage we are just using the jwt to recover enough information
// to calculate the oidc digest. This is going to be used to recover the
// address of the user.
// The final result is the address of the user, the digest and the
// sub for the user (used later on to make the second oauth flow faster)
async function findAddressUsingGoogleData() {
  const buf = new Uint8Array(16);
  crypto.getRandomValues(buf);
  const jwt = await startGoogleOauth(bytesToHex(buf));

  if (jwt === undefined) {
    throw new Error("jwt should not be undefined");
  }

  const digest = await buildOidcDigest(jwt);
  const publicClient = getPublicClient({ chainId: defaultChain.id });

  const addressToRecover = await publicClient.readContract({
    address: contractsByChain[defaultChain.id].recoveryOidc,
    abi: OidcRecoveryValidatorAbi,
    functionName: "addressForDigest",
    args: [digest.toHex()],
  }) as Address;
  emit("done", addressToRecover, digest, jwt.sub);
}
</script>
