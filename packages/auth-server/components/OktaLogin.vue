<template>
  <div class="h-full flex flex-col justify-center px-4">
    <AppAccountLogo class="dark:text-neutral-100 h-16 md:h-20 mb-14" />
    <CommonHeightTransition :opened="!!error">
      <p class="pb-3 text-sm text-error-300 text-center">
        {{ error }}
      </p>
    </CommonHeightTransition>
    <div class="flex flex-col gap-5">
      <ZkHighlightWrapper>
        <ZkButton
          class="w-full"
          :loading="loading"
          data-testid="okta-login"
          @click="handleOktaLogin"
        >
          <template #icon>
            <svg
              class="w-5 h-5"
              viewBox="0 0 24 24"
              fill="currentColor"
            >
              <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z" />
            </svg>
          </template>
          Authorize with Okta
        </ZkButton>
      </ZkHighlightWrapper>

      <p class="text-xs text-slate-500 text-center">
        This will redirect you to your Okta login page to authenticate and authorize chain access.
      </p>
    </div>
  </div>
</template>

<script lang="ts" setup>
const { signInWithRedirect, loading, error } = useOktaAuth();

const handleOktaLogin = async () => {
  await signInWithRedirect();
};
</script>
