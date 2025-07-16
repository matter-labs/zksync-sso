export default defineNuxtRouteMiddleware(() => {
  const runtimeConfig = useRuntimeConfig();

  // Only apply this middleware when in Prividium mode
  if (!runtimeConfig.public.prividiumMode) return;

  const { needsAuthentication } = useOktaAuthStore();

  // If user needs Okta authentication, redirect to auth flow
  if (needsAuthentication) {
    return navigateTo("/auth");
  }
});
