import type { AuthState } from "@okta/okta-auth-js";
import { StorageSerializers, useStorage } from "@vueuse/core";

interface OktaAuthData {
  isAuthenticated: boolean;
  idToken: string | null;
  accessToken: string | null;
  user: unknown | null;
  tokenExpiry: number | null;
}

export const useOktaAuthStore = defineStore("oktaAuth", () => {
  const runtimeConfig = useRuntimeConfig();

  // Persistent storage for Okta auth state
  const authData = useStorage<OktaAuthData | null>("okta-auth", null, undefined, {
    serializer: StorageSerializers.object,
  });

  // Reactive computed properties
  const isAuthenticated = computed(() => {
    if (!runtimeConfig.public.prividiumMode) return true; // Skip auth check if not in Prividium mode
    return authData.value?.isAuthenticated || false;
  });

  const idToken = computed(() => authData.value?.idToken || null);
  const accessToken = computed(() => authData.value?.accessToken || null);
  const user = computed(() => authData.value?.user || null);
  const tokenExpiry = computed(() => authData.value?.tokenExpiry || null);

  // Check if tokens are expired
  const isTokenExpired = computed(() => {
    if (!tokenExpiry.value) return false;
    return Date.now() > tokenExpiry.value;
  });

  // Check if user needs to authenticate for Prividium mode
  const needsAuthentication = computed(() => {
    return runtimeConfig.public.prividiumMode && (!isAuthenticated.value || isTokenExpired.value);
  });

  // Update authentication state
  const setAuthState = (authState: AuthState) => {
    if (!authState) {
      authData.value = null;
      return;
    }

    authData.value = {
      isAuthenticated: authState.isAuthenticated || false,
      idToken: authState.idToken?.idToken || null,
      accessToken: authState.accessToken?.accessToken || null,
      user: authState.idToken?.claims || null,
      tokenExpiry: authState.idToken?.expiresAt ? authState.idToken.expiresAt * 1000 : null,
    };
  };

  // Update tokens manually
  const setTokens = (tokens: { idToken?: string; accessToken?: string; expiresAt?: number }) => {
    const currentData = authData.value || {
      isAuthenticated: false,
      idToken: null,
      accessToken: null,
      user: null,
      tokenExpiry: null,
    };

    authData.value = {
      ...currentData,
      isAuthenticated: true,
      idToken: tokens.idToken || currentData.idToken,
      accessToken: tokens.accessToken || currentData.accessToken,
      tokenExpiry: tokens.expiresAt ? tokens.expiresAt * 1000 : currentData.tokenExpiry,
    };
  };

  // Clear authentication state
  const clearAuthState = () => {
    authData.value = null;
  };

  // Get Bearer token for RPC requests
  const getBearerToken = () => {
    if (!runtimeConfig.public.prividiumMode) return null;
    if (!isAuthenticated.value || isTokenExpired.value) return null;
    return idToken.value;
  };

  // Observable for authentication state changes
  const { subscribe: subscribeOnAuthChange, notify: notifyOnAuthChange } = useObservable<boolean>();

  watch(isAuthenticated, (newIsAuthenticated) => {
    notifyOnAuthChange(newIsAuthenticated);
  });

  return {
    isAuthenticated,
    idToken,
    accessToken,
    user,
    tokenExpiry,
    isTokenExpired,
    needsAuthentication,
    setAuthState,
    setTokens,
    clearAuthState,
    getBearerToken,
    subscribeOnAuthChange,
  };
});
