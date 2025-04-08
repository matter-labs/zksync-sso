export class OidcNotEnabled extends Error {}

function validPubClient(client: string | null | undefined): boolean {
  return typeof client === "string" && client.length > 0;
}

function validUrl(url: string | null | undefined): boolean {
  return typeof url === "string" && URL.canParse(url);
}

function validPath(path: string | null | undefined): boolean {
  return typeof path === "string" && path.length > 0 && path.startsWith("/");
}

function validFile(path: string | null | undefined) {
  return validPath(path) || validUrl(path);
}

export function useOidcConfig() {
  const { public: { oidc } } = useRuntimeConfig();

  const isEnabled = computed<boolean>(() => {
    return validPubClient(oidc.googlePublicClient)
      && validUrl(oidc.saltServiceUrl)
      && validFile(oidc.zkeyUrl)
      && validFile(oidc.witnessUrl);
  });

  function googlePublicClient(): string {
    if (!validPubClient(oidc.googlePublicClient)) {
      throw new OidcNotEnabled();
    }
    return oidc.googlePublicClient;
  }

  function saltServiceUrl(): string {
    if (!validUrl(oidc.saltServiceUrl)) {
      throw new OidcNotEnabled();
    }
    return oidc.saltServiceUrl;
  }

  function zkeyUrl(): string {
    if (!validFile(oidc.zkeyUrl)) {
      throw new OidcNotEnabled();
    }
    return oidc.zkeyUrl;
  }

  function wasmUrl(): string {
    if (!validFile(oidc.witnessUrl)) {
      throw new OidcNotEnabled();
    }
    return oidc.witnessUrl;
  }

  return {
    enabled: isEnabled.value,
    googlePublicClient,
    saltServiceUrl,
    zkeyUrl,
    wasmUrl,
  };
}
