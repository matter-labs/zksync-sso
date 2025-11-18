import { type Hex, toHex } from "viem";
import { findAddressesByPasskey } from "zksync-sso-4337/client";

export const useAccountLogin = (_chainId: MaybeRef<SupportedChainId>) => {
  const chainId = toRef(_chainId);
  const { login } = useAccountStore();
  const { getPublicClient } = useClientStore();

  const { inProgress: loginInProgress, error: accountLoginError, execute: loginToAccount } = useAsync(async () => {
    const client = getPublicClient({ chainId: chainId.value });

    const credential = await getPasskeyCredential();
    if (!credential) {
      throw new Error("No credential found");
    }

    try {
      const contracts = contractsByChain[chainId.value];

      // Use findAddressesByPasskey from sdk-4337
      const result = await findAddressesByPasskey({
        client,
        contracts: {
          webauthnValidator: contracts.webauthnValidator,
        },
        passkey: {
          credentialId: credential.id as Hex,
          originDomain: typeof window !== "undefined" ? window.location.origin : "http://localhost",
        },
      });

      if (!result.addresses || result.addresses.length === 0) {
        throw new Error("No accounts found for this passkey");
      }

      // Use the first address found
      const address = result.addresses[0];

      login({
        username: credential.id, // Use credential ID as username for now
        address,
        passkey: toHex(new Uint8Array(0)), // Legacy field, not used with sdk-4337
        credentialId: credential.id as Hex,
      });
      return { success: true } as const;
    } catch (error) {
      // TODO: Guardian recovery not yet available in sdk-4337
      // Recovery fallback logic commented out
      // const { checkRecoveryRequest, executeRecovery, getRecovery } = useRecoveryGuardian();
      // ...recovery logic...

      // eslint-disable-next-line no-console
      console.warn("Login failed", error);
      throw new Error("Account not found");
    }
  });

  return {
    loginInProgress,
    accountLoginError,
    loginToAccount,
  };
};
