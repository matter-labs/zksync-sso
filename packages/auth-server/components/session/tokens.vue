<template>
  <div
    v-if="tokensLoading || spendLimitTokens.length || onchainActionsCount"
    class="bg-neutral-975 rounded-[28px] text-neutral-100"
  >
    <div class="pl-5 pr-3 py-2 text-neutral-400">
      <div class="flex justify-between">
        <div>Allowed spending</div>
        <div v-if="!hasUnlimitedSpend">
          <CommonContentLoader
            v-if="tokensLoading"
            :length="12"
          />
          <span v-else-if="totalUsd">
            {{ formatPricePretty(totalUsd) }}
          </span>
        </div>
        <div v-else>
          Unlimited
        </div>
      </div>
    </div>
    <CommonLine v-if="!tokensLoading && !spendLimitTokens.length && fetchTokensError">
      <p class="p-4 text-sm text-error-300 break-all">
        Failed to fetch token information. {{ fetchTokensError }}
      </p>
    </CommonLine>
    <CommonLine v-else>
      <div class="divide-y divide-neutral-900">
        <template v-if="tokensLoading">
          <TokenAmountLoader
            v-for="item in 1"
            :key="item"
            size="sm"
            variant="headless"
          />
        </template>
        <template v-else>
          <TokenAmount
            v-for="item in spendLimitTokens"
            :key="item.token.address"
            as="div"
            size="sm"
            variant="headless"
            :symbol="item.token.symbol"
            :name="item.token.name"
            :decimals="item.token.decimals"
            :address="item.token.address"
            :price="item.token.price"
            :icon-url="item.token.iconUrl"
            :amount="item.amount.toString()"
          />
        </template>
        <div
          v-if="onchainActionsCount"
          class="py-2.5 text-center text-sm text-neutral-400"
        >
          <span v-if="spendLimitTokens.length">And</span>
          {{ onchainActionsCount }} onchain action{{ onchainActionsCount > 1 ? 's' : '' }}...
        </div>
      </div>
    </CommonLine>
  </div>
</template>

<script setup lang="ts">
import { useNow } from "@vueuse/core";
import type { SessionConfig } from "zksync-sso/utils";

const props = defineProps<{
  session: Omit<SessionConfig, "signer">;
}>();

const { requestChain } = storeToRefs(useRequestsStore());

const now = useNow({ interval: 1000 });
const {
  onchainActionsCount,
  fetchTokensError,
  tokensLoading,
  spendLimitTokens,
  hasUnlimitedSpend,
  totalUsd,
} = useSessionConfigInfo(
  computed(() => requestChain.value!.id),
  props.session,
  now,
);
</script>
