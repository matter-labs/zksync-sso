<template>
  <div v-if="deploymentResult" class="bg-green-50 p-4 rounded-lg mb-4 border border-green-200">
    <h2 class="text-lg font-semibold mb-2 text-green-800">
      Account Deployed Successfully!
    </h2>
    <div class="space-y-2 text-sm">
      <div v-if="deploymentResult.userId">
        <strong>User ID:</strong>
        <code class="bg-white px-2 py-1 rounded text-xs ml-2">{{ deploymentResult.userId }}</code>
      </div>
      <div v-if="deploymentResult.accountId">
        <strong>Account ID (computed):</strong>
        <code class="bg-white px-2 py-1 rounded text-xs ml-2 block mt-1">{{ deploymentResult.accountId }}</code>
      </div>
      <div>
        <strong>Account Address:</strong>
        <code class="bg-white px-2 py-1 rounded text-xs ml-2" data-testid="deployed-account-address">{{ deploymentResult.address }}</code>
      </div>
      <div v-if="deploymentResult.eoaSigner">
        <strong>EOA Signer:</strong>
        <code class="bg-white px-2 py-1 rounded text-xs ml-2">{{ deploymentResult.eoaSigner }}</code>
        <span class="text-xs text-gray-600 ml-2">(Anvil Rich Wallet #1)</span>
      </div>
      <div v-if="deploymentResult.passkeyEnabled">
        <span><strong>Passkey Enabled:</strong> Yes</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
export interface DeploymentResult {
  userId: string;
  accountId: string;
  address: string;
  eoaSigner: string;
  passkeyEnabled: boolean;
}

defineProps<{ deploymentResult: DeploymentResult | null }>();
</script>
