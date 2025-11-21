<template>
  <div class="bg-yellow-50 p-4 rounded-lg border border-yellow-200">
    <h2 class="text-lg font-semibold mb-3 text-yellow-800">
      WebAuthn Passkey Configuration (Optional)
    </h2>
    <p class="text-sm text-gray-600 mb-4">
      Configure a WebAuthn passkey for the smart account. Leave empty to deploy with EOA signer only.
    </p>

    <div class="space-y-3">
      <div>
        <label class="block text-sm font-medium mb-1">
          <input
            v-model="config.enabled"
            type="checkbox"
            class="mr-2"
          >
          Enable Passkey Deployment
        </label>
      </div>

      <div
        v-if="config.enabled"
        class="space-y-3 pl-6 border-l-2 border-yellow-300"
      >
        <div class="mb-3">
          <button
            :disabled="loading"
            class="w-full px-4 py-2 bg-yellow-600 text-white rounded hover:bg-yellow-700 disabled:opacity-50"
            @click="handleCreatePasskey"
          >
            {{ loading ? 'Creating Passkey...' : 'Create New WebAuthn Passkey' }}
          </button>
          <p class="text-xs text-gray-600 mt-2">
            Click to create a new passkey using your device's authenticator (fingerprint, face ID, security key, etc.)
          </p>

          <!-- Success Message -->
          <div
            v-if="successMessage"
            class="mt-3 p-3 bg-green-50 rounded border border-green-300"
          >
            <p class="text-sm text-green-800">
              {{ successMessage }}
            </p>
          </div>

          <!-- Error Message -->
          <div
            v-if="errorMessage"
            class="mt-3 p-3 bg-red-50 rounded border border-red-300"
          >
            <p class="text-sm text-red-600">
              {{ errorMessage }}
            </p>
          </div>
        </div>

        <div>
          <label class="block text-sm font-medium mb-1">Credential ID (hex):</label>
          <input
            v-model="config.credentialId"
            type="text"
            placeholder="0x2868baa08431052f6c7541392a458f64"
            class="w-full px-3 py-2 border border-gray-300 rounded text-xs font-mono"
          >
          <p class="text-xs text-gray-500 mt-1">
            Example: 0x2868baa08431052f6c7541392a458f64
          </p>
        </div>

        <div>
          <label class="block text-sm font-medium mb-1">Passkey X Coordinate (32 bytes hex):</label>
          <input
            v-model="config.passkeyX"
            type="text"
            placeholder="0xe0a43b9c64a2357ea7f66a0551f57442fbd32031162d9be762800864168fae40"
            class="w-full px-3 py-2 border border-gray-300 rounded text-xs font-mono"
          >
          <p class="text-xs text-gray-500 mt-1">
            32-byte public key X coordinate
          </p>
        </div>

        <div>
          <label class="block text-sm font-medium mb-1">Passkey Y Coordinate (32 bytes hex):</label>
          <input
            v-model="config.passkeyY"
            type="text"
            placeholder="0x450875e2c28222e81eb25ae58d095a3e7ca295faa3fc26fb0e558a0b571da501"
            class="w-full px-3 py-2 border border-gray-300 rounded text-xs font-mono"
          >
          <p class="text-xs text-gray-500 mt-1">
            32-byte public key Y coordinate
          </p>
        </div>

        <div>
          <label class="block text-sm font-medium mb-1">Origin Domain:</label>
          <input
            v-model="config.originDomain"
            type="text"
            placeholder="https://example.com"
            class="w-full px-3 py-2 border border-gray-300 rounded text-sm"
          >
          <p class="text-xs text-gray-500 mt-1">
            The origin domain where the passkey was created
          </p>
        </div>

        <div>
          <label class="block text-sm font-medium mb-1">WebAuthn Validator Address:</label>
          <input
            v-model="config.validatorAddress"
            type="text"
            readonly
            class="w-full px-3 py-2 border border-gray-300 rounded text-xs font-mono bg-gray-50"
          >
          <p class="text-xs text-gray-500 mt-1">
            Address of the WebAuthn validator module (loaded from contracts.json)
          </p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, ref } from "vue";
import { createWebAuthnCredential } from "zksync-sso-4337/client/passkey";

// Props
const props = defineProps({
  modelValue: {
    type: Object,
    required: true,
  },
});

// Emits
const emit = defineEmits(["update:modelValue"]);

// Local state for two-way binding
const config = computed({
  get: () => props.modelValue,
  set: (value) => emit("update:modelValue", value),
});

const loading = ref(false);
const successMessage = ref("");
const errorMessage = ref("");

/**
 * Create a new WebAuthn credential using SimpleWebAuthn helper
 */
async function handleCreatePasskey() {
  loading.value = true;
  successMessage.value = "";
  errorMessage.value = "";

  try {
    // eslint-disable-next-line no-console
    console.log("Creating WebAuthn credential using SimpleWebAuthn...");

    // Create the credential with custom options
    const credential = await createWebAuthnCredential({
      rpName: "SSO Demo",
      rpId: window.location.hostname,
      displayName: "Demo User",
      name: "demo-user@zksync-sso.example",
      timeout: 60000,
    });

    // Update the passkey configuration
    config.value.credentialId = credential.credentialId;
    config.value.passkeyX = credential.publicKey.x;
    config.value.passkeyY = credential.publicKey.y;
    config.value.originDomain = credential.origin;

    // eslint-disable-next-line no-console
    console.log("WebAuthn credential created successfully:");
    // eslint-disable-next-line no-console
    console.log("  Credential ID:", config.value.credentialId);
    // eslint-disable-next-line no-console
    console.log("  Public Key X:", config.value.passkeyX);
    // eslint-disable-next-line no-console
    console.log("  Public Key Y:", config.value.passkeyY);
    // eslint-disable-next-line no-console
    console.log("  Origin:", config.value.originDomain);

    successMessage.value = "Passkey created successfully! The credential details have been populated below.";
  } catch (err) {
    // eslint-disable-next-line no-console
    console.error("Failed to create WebAuthn credential:", err);
    errorMessage.value = `Failed to create passkey: ${err.message}`;
  } finally {
    loading.value = false;
  }
}
</script>
