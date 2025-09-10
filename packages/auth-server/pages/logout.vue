<template>
  <div class="h-full flex flex-col justify-center items-center px-4">
    <div class="text-center">
      <div
        v-if="processing"
        class="flex flex-col items-center space-y-4"
      >
        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-500" />
        <h2 class="text-xl font-semibold text-neutral-900 dark:text-neutral-100">
          Logging out...
        </h2>
        <p class="text-neutral-600 dark:text-neutral-400">
          Please wait while we complete your logout.
        </p>
      </div>

      <div
        v-else-if="error"
        class="max-w-md mx-auto"
      >
        <div class="bg-error-50 dark:bg-error-500/20 rounded-xl p-6 mb-4">
          <h2 class="text-xl font-semibold text-error-700 dark:text-error-300 mb-2">
            Logout Failed
          </h2>
          <p class="text-error-600 dark:text-error-400 text-sm mb-4">
            {{ error }}
          </p>
          <button
            class="bg-primary-500 hover:bg-primary-600 text-white font-medium py-2 px-4 rounded-lg transition-colors duration-200"
            @click="processLogout"
          >
            Try Again
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const runtimeConfig = useRuntimeConfig();
const { logout } = useAccountStore();
const prividiumAuthStore = usePrividiumAuthStore();

const processing = ref(true);
const error = ref<string | null>(null);

const processLogout = async () => {
  try {
    processing.value = true;
    error.value = null;

    // Clear local account store
    logout();

    if (runtimeConfig.public.prividiumMode) {
      // Sign out from Prividium
      prividiumAuthStore.signOut();
    }

    // Navigate to home after logout
    await navigateTo("/");
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Logout failed";
    processing.value = false;
  }
};

processLogout();
</script>
