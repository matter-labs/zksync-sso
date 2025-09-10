<template>
  <SessionTemplate>
    <template
      v-if="isLoggedIn"
      #header
    >
      <SessionAccountHeader message="Connecting with" />
    </template>

    <SessionMetadata
      :app-meta="appMeta"
      :domain="domain"
      size="sm"
    />

    <div class="space-y-2 mt-2">
      <div class="bg-neutral-975 rounded-[28px]">
        <div class="px-5 py-2 text-neutral-400">
          Permissions
        </div>
        <CommonLine class="text-neutral-100">
          <div class="divide-y divide-neutral-800">
            <div class="flex items-center gap-2 py-3 px-3">
              <IconsFingerprint class="w-7 h-7" />
              <div>Act on your behalf</div>
            </div>
            <div class="flex items-center gap-2 py-3 px-3">
              <IconsClock class="w-7 h-7" />
              <div>{{ sessionExpiry }}</div>
            </div>
          </div>
        </CommonLine>
      </div>
    </div>
    <SessionTokens
      :onchain-actions-count="onchainActionsCount"
      :fetch-tokens-error="fetchTokensError"
      :tokens-loading="tokensLoading"
      :spend-limit-tokens="spendLimitTokens"
      :has-unlimited-spend="hasUnlimitedSpend"
      :total-usd="totalUsd"
      class="mt-1"
    />

    <div
      v-if="hasDangerousActions"
      class="mt-2 bg-neutral-975 rounded-[28px]"
    >
      <div class="px-5 py-2 text-error-600 font-bold">
        Warning
      </div>
      <CommonLine class="text-pretty">
        <ul class="text-sm px-5 py-2 space-y-2 text-error-50">
          <li
            v-for="action in dangerousActions"
            :key="action"
            class="list-['-'] list-outside pl-1 ml-3"
          >
            {{ action }}
          </li>
        </ul>
        <div
          ref="checkbox"
          class="px-5 mt-2 mb-3 text-white"
        >
          <ZkCheckbox
            v-model="dangerCheckboxConfirmed"
            :error="dangerCheckboxErrorHighlight"
          >
            I understand that by continuing, I risk losing my funds.
          </ZkCheckbox>
        </div>
      </CommonLine>
    </div>

    <SessionAdvancedInfo :session-config="sessionConfig" />

    <template #footer>
      <CommonHeightTransition :opened="!!sessionError">
        <p class="pb-2 text-sm text-error-300">
          <span>
            {{ sessionError }}
          </span>
        </p>
      </CommonHeightTransition>
      <div class="flex gap-4">
        <ZkButton
          class="w-full"
          type="secondary"
          @click="deny()"
        >
          Cancel
        </ZkButton>
        <ZkHighlightWrapper
          :show-highlight="!isLoggedIn"
          class="w-full"
        >
          <ZkButton
            class="w-full"
            :ui="{ button: 'isolate relative overflow-hidden dark:bg-neutral-100 group' }"
            :disabled="isButtonLoading"
            :loading="isButtonLoading"
            data-testid="connect"
            @click="mainButtonClick()"
          >
            <span
              class="block -z-[1] absolute left-0 top-0 h-full w-0 dark:bg-white group-hover:dark:bg-gray-50"
              :style="{ width: `${scrollProgressPercent}%` }"
            />
            <span class="inline-block w-0 h-full">&nbsp;</span>
            <transition
              :name="transitionName"
              mode="out-in"
            >
              <span
                :key="confirmButtonAvailable.toString()"
                class="inline-block w-full text-center relative"
              >
                {{ confirmButtonAvailable ? mainButtonText : "Continue" }}
              </span>
            </transition>
          </ZkButton>
        </ZkHighlightWrapper>
      </div>
    </template>
  </SessionTemplate>
</template>

<script lang="ts" setup>
import { useNow } from "@vueuse/core";
import { parseEther, toHex } from "viem";
import { generatePrivateKey, privateKeyToAddress } from "viem/accounts";
import type { SessionPreferences } from "zksync-sso";
import { type ExtractReturnType, formatSessionPreferences, type Method, type RPCResponseMessage } from "zksync-sso/client-auth-server";
import { LimitType } from "zksync-sso/utils";

const props = defineProps({
  sessionPreferences: {
    type: Object as PropType<SessionPreferences>,
    required: true,
  },
});

const { appMeta, appOrigin } = useAppMeta();
const { login } = useAccountStore();
const { isLoggedIn } = storeToRefs(useAccountStore());
const { responseInProgress, requestChainId } = storeToRefs(useRequestsStore());
const { createAccount } = useAccountCreate(requestChainId);
const { respond, deny } = useRequestsStore();
const { getClient } = useClientStore();

const defaults = {
  expiresAt: BigInt(Math.floor(Date.now() / 1000) + 60 * 60 * 24), // 24 hours
  feeLimit: {
    limitType: LimitType.Lifetime,
    limit: parseEther("0.001"),
    period: 0n,
  },
};

const sessionConfig = computed(() => formatSessionPreferences(props.sessionPreferences, defaults));

const domain = computed(() => new URL(appOrigin.value).host);
const now = useNow({ interval: 1000 });
const sessionExpiry = computed(() => {
  const expiresDate = bigintDateToDate(sessionConfig.value.expiresAt);

  const { isToday, isTomorrow, formattedDate, formattedTime } = formatExpiryDate({
    expiresAt: expiresDate,
    now: now.value,
  });

  if (isToday) return `Expires today at ${formattedTime}`;
  if (isTomorrow) return `Expires tomorrow at ${formattedTime}`;

  return `Expires on ${formattedDate} at ${formattedTime}`;
});

const {
  onchainActionsCount,
  fetchTokensError,
  tokensLoading,
  spendLimitTokens,
  hasUnlimitedSpend,
  totalUsd,
  dangerousActions,
} = useSessionConfigInfo(
  requestChainId,
  sessionConfig,
  now,
);
const dangerCheckboxConfirmed = ref(false);
const hasDangerousActions = computed(() => dangerousActions.value.length > 0);
const dangerCheckboxErrorHighlight = ref(false);
const { start: startCheckboxErrorHighlightReset, stop: stopCheckboxErrorHighlightReset } = useTimeoutFn(() => {
  dangerCheckboxErrorHighlight.value = false;
}, 3000);
watch(dangerCheckboxConfirmed, (newVal) => {
  if (newVal) {
    dangerCheckboxErrorHighlight.value = false;
    stopCheckboxErrorHighlightReset();
  }
});
const startCheckboxErrorHighlight = () => {
  dangerCheckboxErrorHighlight.value = true;
  startCheckboxErrorHighlightReset();
};

const sessionError = ref("");

const sessionScrollableArea = ref<HTMLElement | undefined>();
const scrollOffsetPx = 60;
const sessionScrollY = ref(0);
const scrollProgressPercent = ref(0);
const arrivedAtBottom = ref(false);

const handleScroll = () => {
  const el = sessionScrollableArea.value;
  if (!el) return;

  const scrollTop = el.scrollTop;
  const scrollHeight = el.scrollHeight;
  const clientHeight = el.clientHeight;

  sessionScrollY.value = scrollTop;

  const scrollBottom = scrollHeight - scrollTop - clientHeight;
  arrivedAtBottom.value = scrollBottom <= scrollOffsetPx;

  // Adjust total scrollable height to treat offset as part of 100%
  const effectiveScrollable = scrollHeight - clientHeight - scrollOffsetPx;

  if (effectiveScrollable > 0) {
    const adjustedProgress = (scrollTop / effectiveScrollable) * 100;
    scrollProgressPercent.value = Math.min(100, adjustedProgress);
  } else {
    scrollProgressPercent.value = 100; // Edge case: no scrolling needed
  }
};

onMounted(() => {
  sessionScrollableArea.value = (document.querySelector("#sessionScrollableArea") as HTMLElement) || undefined;
  if (sessionScrollableArea.value) {
    sessionScrollableArea.value.addEventListener("scroll", handleScroll, { passive: true });
    // Initial check in case it's already scrolled
    handleScroll();
  }
});

onUnmounted(() => {
  if (sessionScrollableArea.value) {
    sessionScrollableArea.value.removeEventListener("scroll", handleScroll);
  }
});
const scrollDown = () => {
  const el = sessionScrollableArea.value;
  el?.scrollTo({
    top: el.scrollTop + (el.clientHeight * 0.7),
    behavior: "smooth",
  });
};
const isButtonLoading = computed(() => !appMeta.value || responseInProgress.value || tokensLoading.value);
const confirmButtonAvailable = computed(() => arrivedAtBottom.value);
const transitionName = ref("slide-up");
const previousConfirmAvailable = ref(confirmButtonAvailable.value);
watch(confirmButtonAvailable, (newVal, oldVal) => {
  if (newVal !== oldVal) {
    transitionName.value = newVal ? "slide-up" : "slide-down";
    previousConfirmAvailable.value = newVal;
  }
});
const mainButtonText = computed(() => isLoggedIn.value ? "Connect" : "Create");

const confirmConnection = async () => {
  let response: RPCResponseMessage<ExtractReturnType<Method>>["content"];
  sessionError.value = "";

  try {
    if (!isLoggedIn.value) {
      // create a new account with initial session data
      const accountData = await createAccount(sessionConfig.value);
      if (!accountData) return;
      login({
        username: accountData.credentialId,
        address: accountData.address,
        passkey: toHex(accountData.credentialPublicKey),
      });

      response = {
        result: constructReturn(
          accountData!.address,
          accountData!.chainId,
          {
            sessionConfig: accountData!.sessionConfig!,
            sessionKey: accountData!.sessionKey!,
          },
        ),
      };
    } else {
      // create a new session for the existing account
      const client = getClient({ chainId: requestChainId.value });
      const paymasterAddress = contractsByChain[requestChainId.value].accountPaymaster;
      const sessionKey = generatePrivateKey();
      const session = {
        sessionKey,
        sessionConfig: {
          signer: privateKeyToAddress(sessionKey),
          ...sessionConfig.value,
        },
      };

      await client.createSession({
        sessionConfig: session.sessionConfig,
        paymaster: {
          address: paymasterAddress,
        },
      });
      response = {
        result: constructReturn(
          client.account.address,
          client.chain.id,
          session,
        ),
      };
    }
  } catch (error) {
    if ((error as Error).message.includes("Passkey validation failed")) {
      sessionError.value = "Passkey validation failed";
    } else {
      sessionError.value = "Error during session creation. Please see console for more info.";
    }
    // eslint-disable-next-line no-console
    console.error(error);
    return;
  }

  if (response) {
    respond(() => response);
  }
};

const mainButtonClick = () => {
  if (!confirmButtonAvailable.value) {
    scrollDown();
  } else if (!dangerCheckboxConfirmed.value && hasDangerousActions.value) {
    startCheckboxErrorHighlight();
  } else {
    confirmConnection();
  }
};
</script>

<style lang="scss" scoped>
/* Common styles */
.slide-up-enter-active,
.slide-up-leave-active,
.slide-down-enter-active,
.slide-down-leave-active {
  @apply transition-all duration-150 ease-in-out absolute inset-0 flex items-center justify-center will-change-[transform,opacity];
}

/* Slide UP (next step) */
.slide-up-enter-from {
  @apply translate-y-full opacity-0;
}
.slide-up-enter-to {
  @apply translate-y-0 opacity-100;
}
.slide-up-leave-from {
  @apply translate-y-0 opacity-100;
}
.slide-up-leave-to {
  @apply -translate-y-full opacity-0;
}

/* Slide DOWN (previous step) */
.slide-down-enter-from {
  @apply -translate-y-full opacity-0;
}
.slide-down-enter-to {
  @apply translate-y-0 opacity-100;
}
.slide-down-leave-from {
  @apply translate-y-0 opacity-100;
}
.slide-down-leave-to {
  @apply translate-y-full opacity-0;
}
</style>
