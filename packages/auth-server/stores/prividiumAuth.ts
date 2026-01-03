import { createPrividiumChain, type PrividiumChain, type UserProfile } from "prividium";

let prividiumInstance: PrividiumChain | null = null;

export const usePrividiumAuthStore = defineStore("prividiumAuth", () => {
  const runtimeConfig = useRuntimeConfig();
  const defaultChainId = runtimeConfig.public.chainId as SupportedChainId;
  const defaultChain = supportedChains.find((chain) => chain.id === defaultChainId);
  if (!defaultChain)
    throw new Error(`Default chain is set to ${defaultChainId}, but is missing from the supported chains list`);

  // Reactive state
  const profile = ref<UserProfile | null>(null);
  const isAuthenticated = ref(false);
  const loading = ref(false);
  const error = ref<string | null>(null);

  // Get or create Prividium instance
  const getPrividiumInstance = (): PrividiumChain | null => {
    if (!runtimeConfig.public.prividiumMode) return null;

    if (!prividiumInstance) {
      const { clientId, rpcUrl, authBaseUrl, permissionsApiBaseUrl } = runtimeConfig.public.prividium || {};

      if (!clientId || !rpcUrl || !authBaseUrl || !permissionsApiBaseUrl) {
        error.value = "Prividium configuration is incomplete";
        return null;
      }

      prividiumInstance = createPrividiumChain({
        clientId,
        chain: defaultChain,
        rpcUrl,
        authBaseUrl,
        permissionsApiBaseUrl: permissionsApiBaseUrl,
        redirectUrl: `${window.location.origin}/callback`,
        onAuthExpiry: () => {
          isAuthenticated.value = false;
          profile.value = null;
          error.value = "Authentication expired. Please log in again.";
        },
      });

      // Initialize auth state from SDK
      isAuthenticated.value = prividiumInstance.isAuthorized();
      if (isAuthenticated.value) fetchProfile();
    }

    return prividiumInstance;
  };

  // Check if user needs to authenticate for Prividium mode
  const needsAuthentication = computed(() => {
    if (!runtimeConfig.public.prividiumMode) return false;
    return !isAuthenticated.value;
  });

  const signInWithPopup = async () => {
    const prividium = getPrividiumInstance();
    if (!prividium) {
      error.value = "Prividium not initialized";
      return;
    }

    try {
      loading.value = true;
      error.value = null;

      await prividium.authorize();
      profile.value = await prividium.fetchUser();

      // Update authentication state after successful login
      isAuthenticated.value = true;
    } catch (err) {
      error.value = err instanceof Error ? err.message : "Failed to sign in. Please try again.";
    } finally {
      loading.value = false;
    }
  };

  const signOut = () => {
    const prividium = getPrividiumInstance();
    if (!prividium) return;

    try {
      loading.value = true;
      prividium.unauthorize();
      // Reset authentication state on logout
      isAuthenticated.value = false;
      profile.value = null;
    } catch (err) {
      error.value = err instanceof Error ? err.message : "Failed to sign out";
    } finally {
      loading.value = false;
    }
  };

  const fetchProfile = async () => {
    const prividium = getPrividiumInstance();
    if (!prividium) {
      error.value = "Prividium not initialized";
      return;
    }

    try {
      loading.value = true;
      error.value = null;

      profile.value = await prividium.fetchUser();
      return profile;
    } catch (err) {
      if (err instanceof Error && err.message.includes("401 Unauthorized")) {
        signOut();
        return;
      }
      error.value = err instanceof Error ? err.message : "Failed to fetch profile";
    } finally {
      loading.value = false;
    }
  };

  const getTransport = () => {
    const prividium = getPrividiumInstance();
    if (!prividium) return null;
    return prividium.transport;
  };

  const initializePrividium = () => {
    if (!runtimeConfig.public.prividiumMode) return;
    getPrividiumInstance();
  };
  initializePrividium();

  return {
    // State
    isAuthenticated: readonly(isAuthenticated),
    profile: readonly(profile),
    loading: readonly(loading),
    error: readonly(error),
    needsAuthentication,
    // Methods
    signInWithPopup,
    signOut,
    getTransport,
    initializePrividium,
    getPrividiumInstance,
  };
});
