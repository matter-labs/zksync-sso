export default defineNuxtRouteMiddleware((to) => {
  const runtimeConfig = useRuntimeConfig();
  const { isLoggedIn } = storeToRefs(useAccountStore());
  const { isAuthenticated } = storeToRefs(useOktaAuthStore());

  const fullyAuthenticated = isLoggedIn.value && (!runtimeConfig.public.prividiumMode || isAuthenticated.value);

  if (to.path !== "/" && !fullyAuthenticated) {
    return navigateTo("/");
  }
});
