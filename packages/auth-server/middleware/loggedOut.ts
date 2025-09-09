export default defineNuxtRouteMiddleware((to) => {
  const runtimeConfig = useRuntimeConfig();
  const { isLoggedIn } = storeToRefs(useAccountStore());
  const { isAuthenticated } = storeToRefs(usePrividiumAuthStore());

  const fullyAuthenticated = isLoggedIn.value && (!runtimeConfig.public.prividiumMode || isAuthenticated.value);

  if (fullyAuthenticated) {
    switch (to.path) {
      case "/":
        return navigateTo("/dashboard");
    }
  }
});
