<template>
  <SessionTemplate>
    <template #header>
      <SessionAccountHeader
        message="Connecting with"
      />
    </template>

    <SessionMetadata
      :app-meta="appMeta"
      :domain="domain"
      :address="address"
    />

    <ul class="mt-6 text-neutral-100 bg-neutral-800/50 p-3 px-4 rounded-zk">
      <li class="flex items-center gap-4 leading-tight my-4">
        <CheckIcon class="w-6 h-6 text-primary-300 shrink-0" />
        Let it see your address, balance and activity
      </li>
      <li class="flex items-center gap-4 leading-tight my-4">
        <CheckIcon class="w-6 h-6 text-primary-300 shrink-0" />
        Let it send you requests for transactions
      </li>
      <li class="flex items-center gap-4 leading-tight my-4">
        <CheckIcon class="w-6 h-6 text-primary-300 shrink-0" />
        Funds will not leave your account without your confirmation
      </li>
    </ul>
    <template #footer>
      <div class="flex gap-4">
        <ZkButton
          class="w-full"
          type="secondary"
          @click="deny()"
        >
          Cancel
        </ZkButton>
        <ZkButton
          class="w-full"
          :loading="!appMeta || responseInProgress"
          data-testid="connect"
          @click="confirmConnection()"
        >
          Connect
        </ZkButton>
      </div>
    </template>
  </SessionTemplate>
</template>

<script lang="ts" setup>
import { CheckIcon } from "@heroicons/vue/24/outline";

const { appMeta, domain } = useAppMeta();
const { respond, deny } = useRequestsStore();
const { responseInProgress, requestChain } = storeToRefs(useRequestsStore());
const { address } = storeToRefs(useAccountStore());
const { getClient } = useClientStore();
const runtimeConfig = useRuntimeConfig();

const confirmConnection = () => {
  console.log("[RequestAccounts] confirmConnection called");
  console.log("[RequestAccounts] requestChain:", requestChain.value);

  respond(async () => {
    console.log("[RequestAccounts] Inside respond callback");
    try {
      const client = getClient({ chainId: requestChain.value!.id });
      console.log("[RequestAccounts] Got client, account:", client.account.address);
      const result = {
        result: constructReturn({
          address: client.account.address,
          chainId: client.chain.id,
          prividiumMode: runtimeConfig.public.prividiumMode,
          prividiumProxyUrl: runtimeConfig.public.prividium?.rpcUrl || "",
        }),
      };
      console.log("[RequestAccounts] Returning result:", result);
      return result;
    } catch (error) {
      console.error("[RequestAccounts] Error in respond:", error);
      throw error;
    }
  });
};
</script>
