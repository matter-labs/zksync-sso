export class OidcNotEnabled extends Error {}

export function useOidcConfig() {
  const { public: { oidc } } = useRuntimeConfig();

  function googlePublicClient(): string {
    if (!oidc.enabled) {
      throw new OidcNotEnabled();
    }
    if (!oidc.googlePublicClient) {
      throw new OidcNotEnabled();
    }
    return oidc.googlePublicClient;
  }

  function saltServiceUrl(): string {
    if (!oidc.enabled) {
      throw new OidcNotEnabled();
    }
    if (!oidc.saltServiceUrl) {
      throw new OidcNotEnabled();
    }
    return oidc.saltServiceUrl;
  }

  return {
    enabled: oidc.enabled,
    googlePublicClient,
    saltServiceUrl,
  };
}
