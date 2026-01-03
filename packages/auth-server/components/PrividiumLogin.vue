<template>
  <div class="h-full flex flex-col justify-center px-4">
    <div class="max-w-md mx-auto w-full">
      <AppAccountLogo class="dark:text-neutral-100 h-16 md:h-20 mb-14" />
      <!-- Error display -->
      <CommonHeightTransition :opened="!!error">
        <p class="pb-3 text-sm text-error-300 text-center">
          {{ error }}
        </p>
      </CommonHeightTransition>

      <!-- Not authenticated: Show Prividium auth button -->
      <div
        v-if="!isAuthenticated"
        class="flex flex-col gap-5"
      >
        <ZkHighlightWrapper>
          <ZkButton
            class="w-full"
            :loading="loading"
            data-testid="prividium-login"
            @click="handlePrividiumLogin"
          >
            <template #icon>
              <svg
                class="w-5 h-5"
                viewBox="0 0 24 24"
                fill="currentColor"
              >
                <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8zm-1-13h2v6h-2zm0 8h2v2h-2z" />
              </svg>
            </template>
            Authorize with Prividium
          </ZkButton>
        </ZkHighlightWrapper>
      </div>

      <!-- Authenticated: Show account creation or login options -->
      <div
        v-else
        class="flex flex-col"
      >
        <!-- Profile display -->
        <div class="bg-slate-50 dark:bg-slate-800 rounded-lg p-3 border border-slate-200 dark:border-slate-700 mb-6">
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

        <!-- Account creation or login -->
        <div class="flex flex-col gap-5">
          <ZkHighlightWrapper>
            <ZkButton
              class="w-full"
              :loading="deployInProgress"
              data-testid="prividium-create-account"
              @click="deployAccount"
            >
              Create New Account
            </ZkButton>
          </ZkHighlightWrapper>

          <ZkButton
            type="secondary"
            class="!text-slate-400"
            :loading="loginInProgress"
            data-testid="prividium-login-existing"
            @click="logIn"
          >
            Log In to Existing Account
          </ZkButton>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
const runtimeConfig = useRuntimeConfig();
const chainId = runtimeConfig.public.chainId as SupportedChainId;

const prividiumAuthStore = usePrividiumAuthStore();
const { loading, isAuthenticated, profile } = storeToRefs(prividiumAuthStore);
const { login } = useAccountStore();
const { loginInProgress, loginToAccount } = useAccountLogin(chainId);
const { registerInProgress: deployInProgress, createAccount, createAccountError } = useAccountCreate(chainId);

const error = computed(() => createAccountError.value?.message || "");

const handlePrividiumLogin = async () => {
  await prividiumAuthStore.signInWithPopup();
};

const logout = () => {
  prividiumAuthStore.signOut();
};

const deployAccount = async () => {
  const result = await createAccount();
  if (result) {
    login({
      address: result.address,
      credentialId: result.credentialId,
    });
    navigateTo("/dashboard");
  }
};

const logIn = async () => {
  const result = await loginToAccount();
  if (result?.success) {
    navigateTo("/dashboard");
    return;
  }
};
</script>
