import { generatePrivateKey, privateKeyToAddress } from "viem/accounts";

export const useAccountCreate = (_chainId: MaybeRef<SupportedChainId>) => {
  const chainId = toRef(_chainId);
  const { registerPasskey } = usePasskeyRegister();
  const runtimeConfig = useRuntimeConfig();
  const { getPrividiumInstance } = usePrividiumAuthStore();

  const { inProgress: registerInProgress, error: createAccountError, execute: createAccount } = useAsync(async () => {
    const result = await registerPasskey();
    if (!result) {
      throw new Error("Failed to register passkey");
    }
    const { credentialPublicKey, credentialId } = result;

    // EOA owner for initial deployment signing
    const ownerKey = generatePrivateKey();
    const ownerAddress = privateKeyToAddress(ownerKey);

    // Call backend API to deploy account (without session - session will be created separately)
    const apiUrl = runtimeConfig.public.authServerApiUrl;
    if (!apiUrl) {
      throw new Error("Auth Server API URL is not configured");
    }

    const requestBody = {
      chainId: chainId.value,
      credentialId,
      credentialPublicKey,
      originDomain: window.location.origin,
      userId: credentialId,
      eoaSigners: [ownerAddress],
    };

    // Build headers - include Prividium auth if in Prividium mode
    const headers: Record<string, string> = {
      "Content-Type": "application/json",
    };

    if (runtimeConfig.public.prividiumMode) {
      const prividium = getPrividiumInstance();
      if (prividium) {
        const authHeaders = prividium.getAuthHeaders();
        if (authHeaders?.Authorization) {
          headers.Authorization = authHeaders.Authorization;
        }
      }
    }

    let response;
    try {
      response = await fetch(`${apiUrl}/api/deploy-account`, {
        method: "POST",
        headers,
        body: JSON.stringify(requestBody),
      });
    } catch (error) {
      console.error("Failed to connect to auth-server API:", error);
      throw new Error(
        `Cannot connect to auth-server API at ${apiUrl}. Please ensure:\n`
        + "1. The auth-server-api is running (pnpm nx dev auth-server-api)\n"
        + "2. CORS is properly configured\n"
        + "3. The API URL is correct\n\n"
        + `Original error: ${error instanceof Error ? error.message : String(error)}`,
      );
    }

    let data;
    try {
      data = await response.json();
    } catch (error) {
      throw new Error(
        `${error} from auth-server API (status ${response.status}). `
        + "This may indicate a contract deployment mismatch or API error.",
      );
    }

    // Check for errors in response
    if (data.error) {
      throw new Error(data.error);
    }

    if (!data.address) {
      throw new Error("No address returned from deployment API");
    }

    const address = data.address;

    return {
      address,
      chainId: chainId.value,
      credentialId,
      credentialPublicKey,
    };
  });

  return {
    registerInProgress,
    createAccount,
    createAccountError,
  };
};
