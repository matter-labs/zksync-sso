<template>
  <div class="session-row">
    <div class="session-id-container">
      <div :title="sessionId">
        #{{ index }}
      </div>
      <a
        class="session-created-time-ago"
        :title="fullCreatedAtDate || ''"
        :href="`${defaultChain.blockExplorers?.native.url}/tx/${transactionHash}`"
        target="_blank"
      >
        {{ createdTimeAgo }}
      </a>
    </div>
    <div class="session-expiry-container">
      <SessionRowExpiry
        v-if="sessionState"
        :status="sessionState.status"
        :is-expired="isExpired"
        :created-at="timestamp"
        :expires-at="expiresAt"
        :now="now"
      />
    </div>
    <div class="session-spend-limit-container">
      <SessionRowSpendLimit
        v-if="sessionState"
        :config="session"
        :state="sessionState"
        :now="now"
        :is-inactive="isInactive"
      />
    </div>
    <div class="session-buttons-container">
      <ZkButton
        v-if="sessionState && sessionState.status === SessionStatus.Active && !isExpired"
        title="Revoke Session"
        type="danger"
        class="ml-auto"
        :ui="{ button: 'block p-2.5 aspect-square', base: 'p-0' }"
        :loading="sessionsInProgress"
        @click="revokeSession()"
      >
        <HandRaisedIcon
          class="h-5 w-5"
          aria-hidden="true"
        />
        <span class="sr-only">Revoke Session</span>
      </ZkButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import { HandRaisedIcon } from "@heroicons/vue/24/outline";
import type { Hash } from "viem";
import { SessionKeyModuleAbi } from "zksync-sso/abi";
import { type SessionConfig, type SessionState, SessionStatus } from "zksync-sso/utils";

const props = defineProps<{
  session: SessionConfig;
  index: number;
  sessionId: Hash;
  transactionHash: Hash;
  blockNumber: bigint;
  timestamp: number;
}>();

const _now = useNow({ interval: 1000 });
const now = computed(() => _now.value.getTime());
const createdTimeAgo = useTimeAgo(props.timestamp);
const fullCreatedAtDate = computed(() => new Date(props.timestamp).toLocaleString());
const expiresAt = computed<number>(() => bigintDateToDate(props.session.expiresAt).getTime());
const timeLeft = computed<number>(() => Math.max(0, expiresAt.value - now.value));
const isExpired = computed(() => timeLeft.value <= 0);

const { defaultChain, getClient, getPublicClient } = useClientStore();
const { address } = storeToRefs(useAccountStore());

const {
  inProgress: sessionsInProgress,
  execute: revokeSession,
} = useAsync(async () => {
  const client = getClient({ chainId: defaultChain.id });
  await client.revokeSession({
    sessionId: props.sessionId,
  });
  await fetchSessionState();
});

const {
  result: sessionState,
  execute: fetchSessionState,
} = useAsync(async () => {
  const client = getPublicClient({ chainId: defaultChain.id });
  const res = await client.readContract({
    address: contractsByChain[defaultChain.id].session,
    abi: SessionKeyModuleAbi,
    functionName: "sessionState",
    args: [address.value!, props.session],
  });
  return res as SessionState;
});

const isInactive = computed(() => isExpired.value || !sessionState.value || sessionState.value.status === SessionStatus.Closed || sessionState.value.status === SessionStatus.NotInitialized);

fetchSessionState();
</script>

<style lang="scss">
/* Not scoped for a reason. Shares classes with RowLoader.vue */
.session-row {
  @apply grid px-4 items-center text-sm;
  @apply grid-cols-2 gap-y-2 py-4 gap-x-8;
  @apply md:grid-cols-[6rem_1fr_1fr_45px] md:py-7 md:h-[100px];

  grid-template-areas:
    "session-id-container session-buttons-container"
    "session-expiry-container session-expiry-container"
    "session-spend-limit-container session-spend-limit-container";
  @media screen and (min-width: 768px) {
    grid-template-areas: "session-id-container session-expiry-container session-spend-limit-container session-buttons-container";
  }

  .session-id-container {
    grid-area: session-id-container;
  }
  .session-expiry-container {
    grid-area: session-expiry-container;
  }
  .session-spend-limit-container {
    grid-area: session-spend-limit-container;
  }
  .session-buttons-container {
    grid-area: session-buttons-container;
  }

  .session-expiry-container, .session-spend-limit-container {
    @apply py-2 md:py-0;
  }

  .session-created-time-ago {
    @apply text-neutral-500 text-xs;
  }
  .session-row-line {
    @apply bg-neutral-900 rounded-full w-full h-1 mt-1;
  }
}
</style>
