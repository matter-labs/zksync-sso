<template>
  <div class="h-full flex flex-col justify-center px-4">
    <!-- Show Prividium authentication first in Prividium mode -->
    <PrividiumLogin v-if="needsPrividiumAuth" />

    <!-- Show normal auth flow after Prividium auth or if not in Prividium mode -->
    <template v-else>
      <SessionMetadata
        :app-meta="appMeta"
        :connect="true"
        class="grow flex justify-center items-center flex-col"
      />

      <CommonHeightTransition :opened="!!accountLoginError">
        <p class="pt-3 text-sm text-error-300 text-center">
          <span>
            Account not found.
            <button
              type="button"
              class="underline underline-offset-4"
              @click="registerAccount"
            >
              Sign up?
            </button>
          </span>
        </p>
      </CommonHeightTransition>

      <CommonHeightTransition :opened="!!createAccountError">
        <p class="pt-3 text-sm text-error-300 text-center">
          <span>
            Creating account failed.
          </span>
        </p>
      </CommonHeightTransition>

      <div class="flex flex-col gap-5 mt-8 py-8">
        <ZkHighlightWrapper>
          <ZkButton
            class="w-full"
            :loading="registerInProgress"
            data-testid="signup"
            @click="registerAccount"
          >
            Sign Up
          </ZkButton>
        </ZkHighlightWrapper>

        <ZkButton
          v-if="canLogin"
          type="secondary"
          class="!text-slate-400"
          :loading="loginInProgress"
          data-testid="login"
          @click="loginAccount"
        >
          Log In
        </ZkButton>
      </div>
    </template>
  </div>
</template>

<script lang="ts" setup>
const { appMeta } = useAppMeta();
const { login } = useAccountStore();
const { requestMethod } = storeToRefs(useRequestsStore());
const runtimeConfig = useRuntimeConfig();

// Prividium authentication store
const { needsAuthentication, walletAddresses } = storeToRefs(usePrividiumAuthStore());

// Account creation and login composables
const { registerInProgress, createAccount, createAccountError } = useAccountCreate();
const { loginInProgress, accountLoginError, loginToAccount } = useAccountLogin();

// Check if Prividium authentication is needed
const needsPrividiumAuth = computed(() => {
  return runtimeConfig.public.prividiumMode && needsAuthentication.value;
});

// In Prividium mode, only show login if user has existing wallet addresses
const canLogin = computed(() => {
  if (!runtimeConfig.public.prividiumMode) return true;
  return walletAddresses.value.length > 0;
});

const registerAccount = async () => {
  const result = await createAccount();
  if (result) {
    login({
      address: result.address,
      credentialId: result.credentialId,
    });
    navigateTo("/confirm/connect");
  }
};

const loginAccount = async () => {
  await loginToAccount();
  // TODO: if app provides a session, check if session for user is active.
  // if active, close the popup and log user in
  // if not active, navigate to connect session page

  // if app does not have sessions, navigate to /confirm/connect page
  // and display request accounts view

  if (requestMethod.value === "eth_requestAccounts") {
    navigateTo("/confirm/connect");
  }
};
</script>
