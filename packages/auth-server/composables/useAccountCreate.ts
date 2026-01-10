import type { Address } from "viem";
import { generatePrivateKey, privateKeyToAddress } from "viem/accounts";
import type { SessionSpec } from "zksync-sso-4337/client";
import { sessionSpecToJSON } from "zksync-sso-4337/client";

export const useAccountCreate = (_chainId: MaybeRef<SupportedChainId>) => {
  const chainId = toRef(_chainId);
  const { registerPasskey } = usePasskeyRegister();
  const runtimeConfig = useRuntimeConfig();
  const { getPrividiumInstance } = usePrividiumAuthStore();

  const { inProgress: registerInProgress, error: createAccountError, execute: createAccount } = useAsync(async (session?: Omit<SessionSpec, "signer">, paymaster?: Address) => {
    const result = await registerPasskey();
    if (!result) {
      throw new Error("Failed to register passkey");
    }
    const { credentialPublicKey, credentialId } = result;

    // TODO: Session support during deployment - to be implemented
    // For now, sessions can be added after deployment
    let sessionData: SessionSpec | undefined;
    const sessionKey = generatePrivateKey();
    const signer = privateKeyToAddress(sessionKey);
    if (session) {
      sessionData = {
        ...session,
        signer: signer,
      };
    }

    // EOA owner for initial deployment signing
    const ownerKey = generatePrivateKey();
    const ownerAddress = privateKeyToAddress(ownerKey);

    // Call backend API to deploy account
    const apiUrl = runtimeConfig.public.authServerApiUrl;
    if (!apiUrl) {
      throw new Error("Auth Server API URL is not configured");
    }

    const requestBody = {
      chainId: chainId.value,
      credentialId,
      credentialPublicKey,
      originDomain: window.location.origin,
      session: sessionData ? sessionSpecToJSON(sessionData) : undefined,
      userId: credentialId,
      eoaSigners: [ownerAddress],
      paymaster,
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

    const response = await fetch(`${apiUrl}/api/deploy-account`, {
      method: "POST",
      headers,
      body: JSON.stringify(requestBody),
    });

    const data = await response.json();

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
      sessionKey: session ? sessionKey : undefined,
      signer,
      sessionConfig: sessionData,
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
