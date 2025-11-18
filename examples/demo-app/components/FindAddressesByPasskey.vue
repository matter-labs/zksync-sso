<template>
  <div class="bg-indigo-50 p-4 rounded-lg mb-4 border border-indigo-200" data-testid="find-addresses-section">
    <h2 class="text-lg font-semibold mb-3 text-indigo-800">
      Find Addresses by Passkey
    </h2>
    <p class="text-sm text-gray-600 mb-4">
      Authenticate with a passkey to find all smart account addresses associated with it.
    </p>

    <div class="space-y-3">
      <button
        :disabled="loading"
        class="w-full px-4 py-2 bg-indigo-500 text-white rounded hover:bg-indigo-600 disabled:opacity-50"
        data-testid="scan-passkey-button"
        @click="scanPasskeyForFindAccounts"
      >
        {{ loading ? 'Authenticating...' : (findPasskeyScanned ? 'Scan Different Passkey' : 'Scan Passkey to Find Accounts') }}
      </button>

      <div v-if="findPasskeyScanned" class="p-3 bg-white rounded border border-indigo-300">
        <div class="space-y-3">
          <div>
            <p class="text-xs text-gray-600 mb-1"><strong>Passkey Credential ID:</strong></p>
            <code class="text-xs font-mono break-all">{{ findPasskeyCredentialId }}</code>
          </div>
          <div>
            <p class="text-xs text-gray-600 mb-1"><strong>Origin Domain:</strong></p>
            <code class="text-xs font-mono">{{ findPasskeyOriginDomain }}</code>
          </div>

          <div v-if="foundAddresses !== null">
            <p class="text-xs text-gray-600 mb-1"><strong>Associated Accounts:</strong></p>
            <div v-if="foundAddresses.length > 0" class="mt-2" data-testid="found-addresses-result">
              <ul class="space-y-1" data-testid="found-addresses-list">
                <li
                  v-for="(address, index) in foundAddresses"
                  :key="address"
                  class="text-xs font-mono bg-gray-100 px-2 py-1 rounded"
                  data-testid="found-address-item"
                >
                  {{ index + 1 }}. {{ address }}
                </li>
              </ul>
            </div>
            <div v-else class="mt-2 p-2 bg-yellow-50 rounded border border-yellow-200" data-testid="no-addresses-found">
              <p class="text-xs text-yellow-800">No accounts found for this passkey.</p>
            </div>
          </div>
        </div>
      </div>

      <div v-if="findPasskeyScanError" class="p-3 bg-red-50 rounded border border-red-300">
        <strong class="text-sm text-red-800">Scan Error:</strong>
        <p class="text-xs text-red-600 mt-1">{{ findPasskeyScanError }}</p>
      </div>

      <div v-if="findAddressesError" class="p-3 bg-red-50 rounded border border-red-300" data-testid="find-addresses-error">
        <strong class="text-sm text-red-800">Search Error:</strong>
        <p class="text-xs text-red-600 mt-1">{{ findAddressesError }}</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import type { Address, Hex } from "viem";
import { findAddressesByPasskey } from "zksync-sso-4337/client";
import { loadContracts, createPublicClient } from "~/utils/contracts";

const loading = ref(false);
const findPasskeyScanned = ref(false);
const findPasskeyCredentialId = ref("");
const findPasskeyOriginDomain = ref("");
const findPasskeyScanError = ref("");
const foundAddresses = ref<Address[] | null>(null);
const findAddressesError = ref("");

async function scanPasskeyForFindAccounts() {
  loading.value = true;
  findPasskeyScanError.value = "";
  foundAddresses.value = null;
  findAddressesError.value = "";
  findPasskeyScanned.value = false;

  try {
    // eslint-disable-next-line no-console
    console.log("Requesting WebAuthn authentication to scan passkey...");

    const challenge = new Uint8Array(32);
    crypto.getRandomValues(challenge);

    const credential = await navigator.credentials.get({
      publicKey: {
        challenge,
        timeout: 60000,
        rpId: window.location.hostname,
        userVerification: "required",
      },
    });

    if (!credential || credential.type !== "public-key") {
      throw new Error("Failed to authenticate with passkey");
    }

    const pkCredential = credential as PublicKeyCredential;
    const credentialId = new Uint8Array(pkCredential.rawId);
    const credentialIdHex = `0x${Array.from(credentialId).map((b) => b.toString(16).padStart(2, "0")).join("")}`;

    findPasskeyCredentialId.value = credentialIdHex;
    findPasskeyOriginDomain.value = window.location.origin;
    findPasskeyScanned.value = true;

    // eslint-disable-next-line no-console
    console.log("Passkey scanned successfully:");
    // eslint-disable-next-line no-console
    console.log("  Credential ID:", credentialIdHex);
    // eslint-disable-next-line no-console
    console.log("  Origin:", window.location.origin);

    // Load contracts configuration and create client
    const contracts = await loadContracts();
    const publicClient = await createPublicClient(contracts);
    // eslint-disable-next-line no-console
    console.log("  WebAuthn Validator:", contracts.webauthnValidator);

    const result = await findAddressesByPasskey({
      client: publicClient,
      contracts: { webauthnValidator: contracts.webauthnValidator as Address },
      passkey: { credentialId: credentialIdHex as Hex, originDomain: window.location.origin },
    });

    foundAddresses.value = result.addresses;
    // eslint-disable-next-line no-console
    console.log(`  Found ${result.addresses.length} account(s):`, result.addresses);
  } catch (err: unknown) {
    // eslint-disable-next-line no-console
    console.error("Failed to scan passkey or find accounts:", err);
    if (!findPasskeyScanned.value) {
      findPasskeyScanError.value = `Failed to scan passkey: ${err instanceof Error ? err.message : String(err)}`;
    } else {
      findAddressesError.value = `Failed to find accounts: ${err instanceof Error ? err.message : String(err)}`;
    }
  } finally {
    loading.value = false;
  }
}
</script>
