<template>
  <div class="mb-8">
    <div
      v-if="!isAuthenticated"
      class="flex flex-col items-center gap-8 text-center"
    >
      <AppAccountLogo class="dark:text-neutral-100 h-16 md:h-20" />

      <h2 class="text-xl font-semibold text-neutral-900 dark:text-neutral-100">
        ZKsync SSO - Prividium Mode
      </h2>
      <button
        class="bg-[#007fdb] hover:bg-[#0066b3] disabled:opacity-60 disabled:cursor-not-allowed text-white font-medium py-3 px-6 rounded-lg transition-colors duration-200"
        @click="signInWithRedirect"
      >
        Authenticate with Okta
      </button>

      <div
        v-if="error"
        class="bg-error-50 dark:bg-error-500/20 rounded-xl p-4 max-w-md"
      >
        <p class="text-error-600 dark:text-error-400 text-sm">
          {{ error }}
        </p>
      </div>
    </div>

    <div
      v-else
      class="bg-neutral-50 dark:bg-neutral-800/50 border border-neutral-200 dark:border-neutral-700 rounded-lg p-3 mb-6"
    >
      <div class="flex items-center justify-between">
        <div class="flex items-center space-x-3">
          <div class="w-2 h-2 bg-green-500 rounded-full" />
          <div>
            <div class="text-xs text-neutral-500 dark:text-neutral-400">
              Authenticated with Okta
            </div>
            <div class="text-sm font-medium text-neutral-700 dark:text-neutral-300 truncate">
              {{ userEmail || 'Unknown' }}
            </div>
          </div>
        </div>
        <button
          class="text-xs text-neutral-400 hover:text-neutral-600 dark:text-neutral-500 dark:hover:text-neutral-300 underline underline-offset-2 transition-colors duration-200"
          @click="signOut"
        >
          Logout
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const { isAuthenticated, userEmail } = storeToRefs(useOktaAuthStore());
const { signInWithRedirect, signOut, error } = useOktaAuth();
</script>
