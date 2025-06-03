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
        <div
          v-else
          class="text-error-500"
        >
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
          >
            <template
              v-if="item.amount === 'unlimited'"
              #right
            >
              <ZkTooltip :label="`Unlimited ${item.token.symbol} spend limit requested`">
                <div class="flex items-center gap-2 text-error-500">
                  <span class="text-sm underline underline-offset-2 decoration-dotted">Attention</span>
                  <ExclamationCircleIcon class="w-6 h-6 inline-block flex-shrink-0" />
                </div>
              </ZkTooltip>
            </template>
            <template
              v-else-if="formatTokenPriceToNumber(item.amount, item.token.decimals, item.token.price || 0) > 1000"
              #right
            >
              <ZkTooltip :label="`High ${item.token.symbol} spend limit requested`">
                <div class="flex items-center gap-2 text-error-500">
                  <span class="text-sm underline underline-offset-2 decoration-dotted">Attention</span>
                  <ExclamationCircleIcon class="w-6 h-6 inline-block flex-shrink-0" />
                </div>
              </ZkTooltip>
            </template>
          </TokenAmount>
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
import { ExclamationCircleIcon } from "@heroicons/vue/24/outline";

defineProps<{
  onchainActionsCount: UseSessionConfigInfoReturn["onchainActionsCount"];
  fetchTokensError: UseSessionConfigInfoReturn["fetchTokensError"];
  tokensLoading: UseSessionConfigInfoReturn["tokensLoading"];
  spendLimitTokens: UseSessionConfigInfoReturn["spendLimitTokens"];
  hasUnlimitedSpend: UseSessionConfigInfoReturn["hasUnlimitedSpend"];
  totalUsd: UseSessionConfigInfoReturn["totalUsd"];
}>();
</script>
