<template>
  <div class="session-row">
    <div class="session-id-container">
      <div
        v-if="sessionHash"
        :title="sessionHash"
        class="truncate"
      >
        {{ sessionHash.slice(0, 10) }}...{{ sessionHash.slice(-8) }}
      </div>
      <div
        v-if="sessionSpec?.signer"
        class="session-signer text-xs text-neutral-500"
      >
        Signer: {{ sessionSpec.signer.slice(0, 6) }}...{{ sessionSpec.signer.slice(-4) }}
      </div>
    </div>
    <div class="session-expiry-container">
      <!-- TODO: created-at is hardcoded to 0 because the current session data
           does not include a creation timestamp. Update this when a real
           created-at value is available or when SessionRowExpiry is changed
           to not rely on created-at for active sessions. -->
      <SessionRowExpiry
        v-if="sessionState"
        :status="sessionState.status"
        :is-expired="isExpired"
        :created-at="0"
        :expires-at="expiresAt"
        :now="now"
      />
    </div>
    <div class="session-spend-limit-container">
      <SessionRowSpendLimit
        v-if="sessionState"
        :config="sessionSpec"
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
import type { Hex } from "viem";
import { SessionKeyValidatorAbi } from "zksync-sso-4337/abi";
import { type SessionConfig, type SessionState, SessionStatus } from "zksync-sso-4337/client";

const props = defineProps<{
  sessionHash: Hex;
  sessionSpec: SessionConfig;
}>();

const _now = useNow({ interval: 1000 });
const now = computed(() => _now.value.getTime());
const expiresAt = computed<number>(() => bigintDateToDate(props.sessionSpec.expiresAt).getTime());
const timeLeft = computed<number>(() => Math.max(0, expiresAt.value - now.value));
const isExpired = computed(() => timeLeft.value <= 0);

const { defaultChain, getClient, getPublicClient } = useClientStore();
const { address } = storeToRefs(useAccountStore());

const {
  inProgress: sessionsInProgress,
  execute: revokeSession,
} = useAsync(async () => {
  const client = getClient({ chainId: defaultChain.id });
  const paymasterAddress = contractsByChain[defaultChain.id].accountPaymaster;
  await client.revokeSession({
    sessionId: props.sessionHash,
    paymaster: {
      address: paymasterAddress,
    },
  });
  await fetchSessionState();
});

const {
  result: sessionState,
  execute: fetchSessionState,
} = useAsync(async () => {
  const client = getPublicClient({ chainId: defaultChain.id });
  const res = await client.readContract({
    address: contractsByChain[defaultChain.id].sessionValidator,
    abi: SessionKeyValidatorAbi,
    functionName: "sessionState",
    args: [address.value!, props.sessionSpec],
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
  @apply md:grid-cols-[10rem_1fr_1fr_45px] md:py-7 md:h-[100px];

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
    @apply bg-neutral-100 dark:bg-neutral-900 rounded-full w-full h-1 mt-1;

    .session-row-line-inner {
      @apply bg-primary-300 dark:bg-white rounded-full h-full will-change-[width,opacity] transition-[width,opacity];
    }
  }
}
</style>
