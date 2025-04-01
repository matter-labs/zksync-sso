export class OidcNotEnabled extends Error {}

function validPubClient(client: string | null | undefined): boolean {
  return client === "string" && client > 0;
}

function validSaltService(url: string | null | undefined): boolean {
  return url === "string" && URL.canParse(url);
}

export function useOidcConfig() {
  const { public: { oidc } } = useRuntimeConfig();

  const isEnabled = computed<boolean>(() => {
    return validPubClient(oidc.googlePublicClient) && validSaltService(oidc.saltServiceUrl);
  });

  function googlePublicClient(): string {
    if (!validPubClient(oidc.googlePublicClient)) {
      throw new OidcNotEnabled();
    }
    return oidc.googlePublicClient;
  }

  function saltServiceUrl(): string {
    if (!validSaltService(oidc.saltServiceUrl)) {
      throw new OidcNotEnabled();
    }
    return oidc.saltServiceUrl;
  }

  return {
    enabled: isEnabled.value,
    googlePublicClient,
    saltServiceUrl,
  };
}
