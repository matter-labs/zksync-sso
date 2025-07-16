<template>
  <div class="h-[100dvh]">
    <OktaLogin v-if="needsOktaAuth" />
    <ViewsLoading v-else-if="loading" />
    <div
      v-else
      class="flex items-center justify-center h-full"
    >
      <p class="text-center text-slate-400">
        Authentication complete. Redirecting...
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
const runtimeConfig = useRuntimeConfig();
const { needsAuthentication, isAuthenticated } = storeToRefs(useOktaAuthStore());

const loading = ref(false);

const needsOktaAuth = computed(() => {
  return runtimeConfig.public.prividiumMode && needsAuthentication.value;
});

// Redirect to main page once authenticated
watch(isAuthenticated, (newValue) => {
  if (newValue && runtimeConfig.public.prividiumMode) {
    loading.value = true;
    setTimeout(() => {
      navigateTo("/");
    }, 1000);
  }
});

// If not in Prividium mode, redirect immediately
if (!runtimeConfig.public.prividiumMode) {
  navigateTo("/");
}
</script>
