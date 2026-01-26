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

    <!-- Only show session permissions for logged-in users (adding session to existing account) -->
    <div
      v-if="isLoggedIn"
      class="space-y-2 mt-2"
    >
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
      v-if="isLoggedIn"
      :onchain-actions-count="onchainActionsCount"
      :fetch-tokens-error="fetchTokensError"
      :tokens-loading="tokensLoading"
      :spend-limit-tokens="spendLimitTokens"
      :has-unlimited-spend="hasUnlimitedSpend"
      :total-usd="totalUsd"
      class="mt-1"
    />

    <!-- For new accounts creating session in two steps -->
    <div
      v-if="accountCreated && hasSessionParams"
      class="space-y-2 mt-2"
    >
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
      v-if="accountCreated && hasSessionParams"
      :onchain-actions-count="onchainActionsCount"
      :fetch-tokens-error="fetchTokensError"
      :tokens-loading="tokensLoading"
      :spend-limit-tokens="spendLimitTokens"
      :has-unlimited-spend="hasUnlimitedSpend"
      :total-usd="totalUsd"
      class="mt-1"
    />

    <!-- For new accounts without session yet, show a simple message -->
    <div
      v-if="!isLoggedIn && !accountCreated"
      class="space-y-2 mt-2"
    >
      <div class="bg-neutral-975 rounded-[28px]">
        <div class="px-5 py-2 text-neutral-400">
          Account Creation
        </div>
        <CommonLine class="text-neutral-100">
          <div class="py-3 px-3">
            <p class="text-sm text-neutral-300">
              A new smart account will be created for you.
              <span v-if="hasSessionParams">After that, you'll set up spending permissions.</span>
              <span v-else>You can add spending permissions later when needed.</span>
            </p>
          </div>
        </CommonLine>
      </div>
    </div>

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
import { encodePacked, keccak256, pad, parseEther } from "viem";
import { generatePrivateKey, privateKeyToAccount, privateKeyToAddress } from "viem/accounts";
import { formatSessionPreferences, getSessionHash, type SessionPreferences } from "zksync-sso-4337/client";
import { LimitType } from "zksync-sso-4337/client";
import type { ExtractReturnType, Method, RPCResponseMessage } from "zksync-sso-4337/client-auth-server";

const props = defineProps({
  sessionPreferences: {
    type: Object as PropType<SessionPreferences>,
    required: true,
  },
});

const { appMeta, appOrigin } = useAppMeta();
const { login } = useAccountStore();
const { isLoggedIn } = storeToRefs(useAccountStore());
const { responseInProgress, requestChainId, requestPaymaster } = storeToRefs(useRequestsStore());
const { createAccount, registerInProgress: accountCreationInProgress } = useAccountCreate(requestChainId);
const { respond, deny } = useRequestsStore();
const { getClient } = useClientStore();
const runtimeConfig = useRuntimeConfig();

// Track if we just created an account and need to create session
const accountCreated = ref(false);
const hasSessionParams = computed(() => {
  return props.sessionPreferences.contractCalls && props.sessionPreferences.contractCalls.length > 0;
});

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
const isButtonLoading = computed(() => !appMeta.value || responseInProgress.value || tokensLoading.value || accountCreationInProgress.value);
const confirmButtonAvailable = computed(() => arrivedAtBottom.value);
const transitionName = ref("slide-up");
const previousConfirmAvailable = ref(confirmButtonAvailable.value);
watch(confirmButtonAvailable, (newVal, oldVal) => {
  if (newVal !== oldVal) {
    transitionName.value = newVal ? "slide-up" : "slide-down";
    previousConfirmAvailable.value = newVal;
  }
});
const mainButtonText = computed(() => {
  if (isLoggedIn.value) return "Connect";
  if (accountCreated.value) return "Authorize";
  return "Create";
});

const confirmConnection = async () => {
  let response: RPCResponseMessage<ExtractReturnType<Method>>["content"];
  sessionError.value = "";

  try {
    if (!isLoggedIn.value && !accountCreated.value) {
      // Step 1: Create the account
      const accountData = await createAccount();
      if (!accountData) return;

      // Login with the new account
      login({
        address: accountData.address,
        credentialId: accountData.credentialId,
      });

      // If session params were provided, show session creation UI
      if (hasSessionParams.value) {
        accountCreated.value = true;
        // Force a UI update before returning
        await nextTick();
        return; // Don't respond yet, wait for session creation
      }

      // No session params, return account info only
      response = {
        result: constructReturn({
          address: accountData.address,
          chainId: accountData.chainId,
          prividiumMode: runtimeConfig.public.prividiumMode,
          prividiumProxyUrl: runtimeConfig.public.prividium?.rpcUrl || "",
        }),
      };
    } else if (accountCreated.value || isLoggedIn.value) {
      // create a new session for the existing account
      // Use paymaster if provided (needed for newly created accounts with no ETH)
      const client = getClient({
        chainId: requestChainId.value,
        usePaymaster: !!requestPaymaster.value,
        paymasterAddress: requestPaymaster.value,
      });
      const sessionKey = generatePrivateKey();
      const session = {
        sessionKey,
        sessionConfig: {
          signer: privateKeyToAddress(sessionKey),
          ...sessionConfig.value,
        },
      };

      // Proof: sign keccak256(abi.encode(sessionSpec, account)) with the session key
      const sessionHash = getSessionHash(session.sessionConfig);
      const digest = keccak256(encodePacked([
        "bytes32",
        "bytes32",
      ], [
        sessionHash,
        pad(client.account.address),
      ]));
      const sessionSigner = privateKeyToAccount(sessionKey);
      const proof = await sessionSigner.sign({ hash: digest });

      await client.createSession({
        sessionSpec: session.sessionConfig,
        proof,
        contracts: {
          sessionValidator: contractsByChain[requestChainId.value].sessionValidator,
        },
      });

      response = {
        result: constructReturn({
          address: client.account.address,
          chainId: client.chain.id,
          session,
          prividiumMode: runtimeConfig.public.prividiumMode,
          prividiumProxyUrl: runtimeConfig.public.prividium?.rpcUrl || "",
        }),
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
