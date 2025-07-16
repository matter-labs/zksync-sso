export default defineNuxtPlugin(async () => {
  const runtimeConfig = useRuntimeConfig();

  // Only initialize Okta in Prividium mode
  if (!runtimeConfig.public.prividiumMode) return;

  const { handleLoginRedirect, initializeOktaAuth } = useOktaAuth();

  // Initialize Okta Auth
  initializeOktaAuth();

  // Handle Okta redirect callback
  if (window.location.search.includes("code=") || window.location.search.includes("state=")) {
    await handleLoginRedirect();
  }
});
