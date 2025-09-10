<template>
  <main class="h-full flex flex-col justify-center px-4">
    <!-- Show PrividiumLogin component when in Prividium mode -->
    <PrividiumLogin v-if="isPrividiumMode" />

    <!-- Show normal auth flow when not in Prividium mode -->
    <template v-else>
      <AppAccountLogo
        class="dark:text-neutral-100 h-16 md:h-20 mb-8"
      />

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
import { toHex } from "viem";

definePageMeta({
  middleware: ["logged-out"],
});

const runtimeConfig = useRuntimeConfig();
const { $config } = useNuxtApp();

const chainId = runtimeConfig.public.chainId as SupportedChainId;

const isPrividiumMode = computed(() => $config.public.prividiumMode);

const { login } = useAccountStore();
const { registerInProgress, createAccount } = useAccountCreate(chainId);
const { loginInProgress, accountLoginError, loginToAccount } = useAccountLogin(chainId);

const signUp = async () => {
  const result = await createAccount();
  if (!result) return;
  login({
    username: result.credentialId,
    address: result.address,
    passkey: toHex(result.credentialPublicKey),
  });
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
</script>
