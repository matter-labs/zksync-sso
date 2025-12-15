import type { Address } from "viem";
import { generatePrivateKey, privateKeyToAddress } from "viem/accounts";
import type { SessionSpec } from "zksync-sso-4337/client";
import { sessionSpecToJSON } from "zksync-sso-4337/client";

export const useAccountCreate = (_chainId: MaybeRef<SupportedChainId>, prividiumMode = false) => {
  const chainId = toRef(_chainId);
  const { getThrowAwayClient } = useClientStore();
  const { registerPasskey } = usePasskeyRegister();
  const { fetchAddressAssociationMessage, associateAddress, deleteAddressAssociation } = usePrividiumAddressAssociation();
  const runtimeConfig = useRuntimeConfig();

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

    const deployerClient = getThrowAwayClient({ chainId: chainId.value });

    // For Prividium mode, associate temporary address before deployment
    if (prividiumMode) {
      const { message } = await fetchAddressAssociationMessage(deployerClient.account.address);
      const signature = await deployerClient.signMessage({ message });
      await associateAddress(deployerClient.account.address, message, signature);
    }

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
      userId: credentialId, // Use credential ID as unique user ID
      eoaSigners: [ownerAddress],
      paymaster,
    };
    console.log("[DEBUG] useAccountCreate - Request body:", JSON.stringify(requestBody, null, 2));

    const response = await fetch(`${apiUrl}/api/deploy-account`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
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

    // Clean up temporary association for Prividium mode
    if (prividiumMode) {
      await deleteAddressAssociation(deployerClient.account.address).catch((err) => {
        // Ignore errors on cleanup
        // eslint-disable-next-line no-console
        console.warn("Failed to delete temporary address association:", err);
      });
    }

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
