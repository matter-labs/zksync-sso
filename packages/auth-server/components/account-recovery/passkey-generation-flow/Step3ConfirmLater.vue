<template>
  <div class="flex flex-col gap-4 text-center text-neutral-700 dark:text-neutral-300">
    <p>
      Please share the following url with your guardian to complete the recovery process:
    </p>
  </div>

  <div class="w-full  items-center gap-2 p-4 bg-neutral-100 dark:bg-neutral-900 rounded-zk">
    <a
      :href="recoveryUrl"
      target="_blank"
      class="text-sm text-neutral-800 dark:text-neutral-100 break-all hover:text-neutral-900 dark:hover:text-neutral-400 leading-relaxed underline underline-offset-4 decoration-neutral-400 hover:decoration-neutral-900 dark:decoration-neutral-600 dark:hover:decoration-neutral-400"
    >
      {{ recoveryUrl }}
    </a>
    <common-copy-to-clipboard
      :text="recoveryUrl ?? ''"
      class="!inline-flex ml-1"
    />
  </div>

  <p class="text-sm text-center text-neutral-600 dark:text-neutral-400">
    You'll be able to access your account once your guardian confirms the recovery.
  </p>

  <ZkLink
    type="primary"
    href="/"
    class="w-full"
  >
    Back to Home
  </ZkLink>
</template>

<script setup lang="ts">
import type { Address } from "viem";

import type { RegisterNewPasskeyReturnType } from "~/composables/usePasskeyRegister";

const props = defineProps<{
  accountAddress: Address;
  newPasskey: RegisterNewPasskeyReturnType;
}>();

const recoveryUrl = computedAsync(async () => {
  const queryParams = new URLSearchParams();

  // Use base64url format for credentialId (required by contract)
  const credentialId = props.newPasskey.credentialIdBase64url;
  // Serialize the public key as JSON since it's {x, y} format
  const credentialPublicKey = JSON.stringify(props.newPasskey.credentialPublicKey);

  /* eslint-disable no-console */
  console.log("🔧 URL GENERATION DEBUG");
  console.log("================================");
  console.log("Original values BEFORE URLSearchParams:");
  console.log("  accountAddress:", props.accountAddress);
  console.log("  credentialId:", credentialId);
  console.log("  credentialPublicKey:", credentialPublicKey);
  console.log("  credentialPublicKey (length):", credentialPublicKey.length);
  console.log("  credentialPublicKey (first 20 chars):", credentialPublicKey.substring(0, 20));

  queryParams.set("credentialId", credentialId);
  queryParams.set("credentialPublicKey", credentialPublicKey);
  queryParams.set("accountAddress", props.accountAddress);

  // Create checksum from concatenated credential data
  // Normalize accountAddress to lowercase for consistent hashing
  const normalizedAddress = props.accountAddress.toLowerCase();
  const dataToHash = `${normalizedAddress}:${credentialId}:${credentialPublicKey}`;
  console.log("\n🔐 Checksum generation:");
  console.log("  Data to hash:", dataToHash);
  console.log("  Data to hash (length):", dataToHash.length);

  const fullHash = new Uint8Array(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(dataToHash)));
  const shortHash = fullHash.slice(0, 8); // Take first 8 bytes of the hash
  const checksum = uint8ArrayToHex(shortHash);

  console.log("  Generated checksum:", checksum);

  queryParams.set("checksum", checksum);

  const finalUrl = new URL(`/recovery/guardian/confirm-recovery?${queryParams.toString()}`, window.location.origin).toString();

  console.log("\n📋 URLSearchParams encoding:");
  console.log("  credentialPublicKey (encoded):", queryParams.get("credentialPublicKey"));
  console.log("  Full query string:", queryParams.toString());
  console.log("\n🔗 Final URL:", finalUrl);
  console.log("================================\n");
  /* eslint-enable no-console */

  return finalUrl;
});
</script>
