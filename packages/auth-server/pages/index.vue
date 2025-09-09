<template>
  <main class="h-full flex flex-col justify-center px-4">
    <!-- Show PrividiumLogin component when in Prividium mode and not authenticated -->
    <PrividiumLogin v-if="isPrividiumMode && !isAuthenticated" />

    <!-- Show normal auth flow after Prividium auth or if not in Prividium mode -->
    <template v-if="!isPrividiumMode || isAuthenticated">
      <AppAccountLogo
        class="dark:text-neutral-100 h-16 md:h-20 mb-8"
      />

      <!-- Small Prividium profile block when authenticated -->
      <div
        v-if="isPrividiumMode && isAuthenticated"
        class="bg-slate-50 dark:bg-slate-800 rounded-lg p-3 border border-slate-200 dark:border-slate-700"
      >
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-2">
            <div class="w-2 h-2 bg-green-500 rounded-full" />
            <span class="text-sm text-slate-600 dark:text-slate-400">
              {{ profile?.displayName || profile?.userId || "Authenticated User" }}
            </span>
          </div>
          <button
            class="text-xs text-slate-500 dark:text-slate-400 hover:text-slate-600 dark:hover:text-slate-300 underline"
            @click="logout"
          >
            Sign Out
          </button>
        </div>
      </div>

      <div class="flex flex-col gap-5 mt-6 py-8">
        <ZkHighlightWrapper>
          <ZkButton
            class="w-full"
            :loading="registerInProgress"
            data-testid="signup"
            @click="signUp"
          >
            Sign Up
          </ZkButton>
        </ZkHighlightWrapper>

        <ZkButton
          type="secondary"
          class="!text-slate-400"
          :loading="loginInProgress"
          data-testid="login"
          @click="logIn"
        >
          Log In
        </ZkButton>

        <ZkLink
          class="w-fit mx-auto mt-2"
          href="/recovery"
        >
          Recover your account
        </ZkLink>
      </div>

      <CommonHeightTransition :opened="!!accountLoginError">
        <p class="pt-3 text-sm text-error-300 text-center">
          <span>
            Account not found.
            <button
              type="button"
              class="underline underline-offset-4"
              @click="signUp"
            >
              Sign up?
            </button>
          </span>
        </p>
      </CommonHeightTransition>
    </template>
  </main>
</template>

<script setup lang="ts">
definePageMeta({
  middleware: ["logged-out"],
});

const runtimeConfig = useRuntimeConfig();
const { $config } = useNuxtApp();
const prividiumAuthStore = usePrividiumAuthStore();

const chainId = runtimeConfig.public.chainId as SupportedChainId;

const isPrividiumMode = computed(() => $config.public.prividiumMode);
const { isAuthenticated, profile } = storeToRefs(prividiumAuthStore);

const { registerInProgress, createAccount } = useAccountCreate(chainId);
const { loginInProgress, accountLoginError, loginToAccount } = useAccountLogin(chainId);

const signUp = async () => {
  await createAccount();
  navigateTo("/dashboard");
};

const logIn = async () => {
  const result = await loginToAccount();
  if (result?.success) {
    navigateTo("/dashboard");
    return;
  }
  if (result?.recoveryRequest?.isReady === false) {
    navigateTo(`/recovery/account-not-ready?address=${result!.recoveryRequest.accountAddress}`);
    return;
  }
  // TODO: handle rest of the cases
};

const logout = () => {
  prividiumAuthStore.signOut();
};
</script>
