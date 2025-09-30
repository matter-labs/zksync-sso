<!-- Simple test page to verify web SDK integration -->
<template>
  <div class="container mx-auto p-6">
    <h1 class="text-2xl font-bold mb-4">
      zkSync SSO Web SDK Test
    </h1>

    <div class="bg-gray-100 p-4 rounded-lg mb-4">
      <h2 class="text-lg font-semibold mb-2">
        Web SDK Status
      </h2>
      <p class="text-sm text-gray-600 mb-2">
        Testing WASM-based zkSync SSO ERC-4337 integration
      </p>

      <div class="space-y-2">
        <div>
          <strong>SDK Loaded:</strong>
          <span :class="sdkLoaded ? 'text-green-600' : 'text-red-600'">
            {{ sdkLoaded ? 'Yes' : 'No' }}
          </span>
        </div>

        <div v-if="sdkLoaded">
          <strong>Test Result:</strong>
          <span class="text-blue-600">{{ testResult }}</span>
        </div>

        <div v-if="error">
          <strong>Error:</strong>
          <span class="text-red-600">{{ error }}</span>
        </div>
      </div>

      <button
        :disabled="loading"
        class="mt-4 px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:opacity-50"
        @click="testWebSDK"
      >
        {{ loading ? 'Testing...' : 'Test Web SDK' }}
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from "vue";

// Reactive state
const sdkLoaded = ref(false);
const testResult = ref("");
const error = ref("");
const loading = ref(false);

// Test the web SDK
async function testWebSDK() {
  loading.value = true;
  error.value = "";
  testResult.value = "";

  try {
    // Import the web SDK (dynamic import for client-side only)
    const { ZkSyncSsoClient } = await import("@zksync-sso/web-sdk/bundler");

    // Test basic configuration
    const config = {
      rpcUrl: "https://sepolia.era.zksync.dev",
      bundlerUrl: "https://bundler.example.com",
      contracts: {
        entryPoint: "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789",
        accountFactory: "0x9406Cc6185a346906296840746125a0E44976454",
      },
    };

    // Create client instance (this will use stub implementation for now)
    const client = new ZkSyncSsoClient(config);

    testResult.value = "Web SDK client created successfully!";
    console.log("zkSync SSO Web SDK client:", client);
  } catch (err) {
    console.error("Web SDK test failed:", err);
    error.value = `Failed to test Web SDK: ${err.message}`;
  } finally {
    loading.value = false;
  }
}

// Check if SDK is available on mount
onMounted(async () => {
  // Only run on client side
  if (import.meta.client) {
    try {
      await import("@zksync-sso/web-sdk/bundler");
      sdkLoaded.value = true;
    } catch (err) {
      console.error("Failed to load Web SDK:", err);
      error.value = `Failed to load Web SDK: ${err.message}`;
    }
  }
});
</script>
