<template>
  <div>
    <div
      v-if="fetchTokensError"
      class="text-error-300"
    >
      Failed fetching tokens info
    </div>
    <div v-else>
      <span v-if="tokensLoading">
        <CommonContentLoader :length="20" />
      </span>
      <span
        v-else-if="totalSpentUsd || totalUsd"
        :class="{ 'text-neutral-500': !totalSpentUsd }"
      >
        {{ formatPricePretty(totalSpentUsd) }}
        <span>used</span>
        <span
          v-if="hasUnlimitedSpend"
          :class="{ 'text-neutral-500': isInactive }"
        >
          <span class="text-neutral-500">&nbsp;of</span>
          unlimited
        </span>
        <span
          v-else-if="totalUsd"
          :class="{ 'text-neutral-500': isInactive }"
        >
          <span class="text-neutral-500">&nbsp;of</span>
          {{ formatPricePretty(totalUsd) }}
        </span>
      </span>
    </div>
    <div
      v-if="tokensLoading || (!fetchTokensError && !hasUnlimitedSpend)"
      class="session-row-line"
      :class="tokensLoading ? 'animate-pulse' : ''"
    >
      <div
        class="session-row-line-inner"
        :class="{ 'opacity-30': tokensLoading || !totalSpentUsd || isInactive }"
        :style="{ width: `${usedPercentage}%` }"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import type { SessionConfig, SessionState } from "zksync-sso-4337/client";

const props = defineProps<{
  config: SessionConfig;
  state: SessionState;
  now: number;
  isInactive: boolean;
}>();

const { defaultChain } = useClientStore();

const {
  fetchTokensError,
  tokensLoading,
  hasUnlimitedSpend,
  totalUsd,
} = useSessionConfigInfo(
  defaultChain.id,
  computed(() => props.config),
  computed(() => new Date(props.now)),
);
const {
  totalUsd: totalSpentUsd,
} = useSessionStateInfo(
  defaultChain.id,
  computed(() => props.config),
  computed(() => props.state),
);

const usedPercentage = computed(() => {
  if (tokensLoading.value) return 0;
  const total = totalUsd.value;
  return Math.min(100, (totalSpentUsd.value / total) * 100);
});
</script>
