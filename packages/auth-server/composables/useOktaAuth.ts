import { OktaAuth, type RefreshToken } from "@okta/okta-auth-js";

// Singleton instance - not reactive
let oktaAuthInstance: OktaAuth | null = null;

export const useOktaAuth = () => {
  const runtimeConfig = useRuntimeConfig();
  const { setAuthState } = useOktaAuthStore();
  const loading = ref(false);
  const error = ref<string | null>(null);

  // Get or create Okta Auth instance
  const getOktaAuthInstance = () => {
    if (!runtimeConfig.public.prividiumMode) return null;

    if (!oktaAuthInstance) {
      if (!runtimeConfig.public.okta.issuer || !runtimeConfig.public.okta.clientId) {
        error.value = "Okta configuration is incomplete";
        return null;
      }

      oktaAuthInstance = new OktaAuth({
        issuer: runtimeConfig.public.okta.issuer,
        clientId: runtimeConfig.public.okta.clientId,
        redirectUri: runtimeConfig.public.okta.redirectUri || (import.meta.client ? window.location.origin : ""),
        scopes: ["openid", "profile", "email"],
        responseType: "code",
        pkce: true,
        transformAuthState: async (_oktaAuth, authState) => {
          // Update the store with auth state
          setAuthState(authState);
          return authState;
        },
      });

      // Handle authentication state changes
      oktaAuthInstance.authStateManager.subscribe((authState) => {
        setAuthState(authState);
      });

      // Start the auth state manager
      oktaAuthInstance.start();
    }

    return oktaAuthInstance;
  };

  // Get ID token for Bearer authentication
  const getIdToken = () => {
    const oktaAuth = getOktaAuthInstance();
    if (!oktaAuth) return null;

    try {
      const idToken = oktaAuth.tokenManager.getSync("idToken");
      return idToken || null;
    } catch {
      error.value = "Error getting ID token";
      return null;
    }
  };

  // Get access token
  const getAccessToken = () => {
    const oktaAuth = getOktaAuthInstance();
    if (!oktaAuth) return null;

    try {
      const accessToken = oktaAuth.tokenManager.getSync("accessToken");
      return accessToken || null;
    } catch {
      error.value = "Error getting access token";
      return null;
    }
  };

  // Login with redirect
  const signInWithRedirect = async () => {
    const oktaAuth = getOktaAuthInstance();
    if (!oktaAuth) {
      error.value = "Okta Auth not initialized";
      return;
    }

    try {
      loading.value = true;
      error.value = null;
      await oktaAuth.signInWithRedirect();
    } catch {
      error.value = "Failed to sign in. Please try again.";
    } finally {
      loading.value = false;
    }
  };

  // Handle redirect callback
  const handleLoginRedirect = async () => {
    const oktaAuth = getOktaAuthInstance();
    if (!oktaAuth) return;

    try {
      const tokens = await oktaAuth.token.parseFromUrl();
      oktaAuth.tokenManager.setTokens(tokens.tokens);
    } catch {
      error.value = "Failed to handle login redirect";
    }
  };

  // Logout
  const signOut = async () => {
    const oktaAuth = getOktaAuthInstance();
    if (!oktaAuth) return;

    try {
      loading.value = true;
      await oktaAuth.signOut();
    } catch {
      error.value = "Failed to sign out";
    } finally {
      loading.value = false;
    }
  };

  // Refresh tokens
  const refreshTokens = async () => {
    const oktaAuth = getOktaAuthInstance();
    if (!oktaAuth) return;

    try {
      const refreshToken = oktaAuth.tokenManager.getSync("refreshToken");
      if (refreshToken) {
        const tokenParams = {
          scopes: ["openid", "profile", "email"],
        };
        const tokens = await oktaAuth.token.renewTokensWithRefresh(tokenParams, refreshToken as RefreshToken);
        oktaAuth.tokenManager.setTokens(tokens);
      }
    } catch {
      error.value = "Failed to refresh tokens";
    }
  };

  // Initialize Okta Auth (called from plugin)
  const initializeOktaAuth = () => {
    if (import.meta.client && runtimeConfig.public.prividiumMode) {
      getOktaAuthInstance();
    }
  };

  return {
    loading,
    error,
    getIdToken,
    getAccessToken,
    signInWithRedirect,
    handleLoginRedirect,
    signOut,
    refreshTokens,
    initializeOktaAuth,
  };
};
