import { findAddressesByPasskey, getPasskeyCredential } from "zksync-sso-4337/client";

export const useAccountLogin = (_chainId: MaybeRef<SupportedChainId>) => {
  const chainId = toRef(_chainId);
  const { login } = useAccountStore();
  const { getPublicClient } = useClientStore();

  const { inProgress: loginInProgress, error: accountLoginError, execute: loginToAccount } = useAsync(async () => {
    const client = getPublicClient({ chainId: chainId.value });

    const credential = await getPasskeyCredential();
    if (!credential) throw new Error("No credential found");

    try {
      const contracts = contractsByChain[chainId.value];

      // Use findAddressesByPasskey from sdk-4337
      const result = await findAddressesByPasskey({
        client,
        contracts: {
          webauthnValidator: contracts.webauthnValidator,
        },
        passkey: {
          credentialId: credential.credentialIdHex,
          originDomain: window.location.origin,
        },
      });
      if (!result.addresses.length) throw new Error("No accounts found for this passkey");

      // Use the first address found
      const address = result.addresses[0];

      login({
        address,
        credentialId: credential.credentialIdHex,
      });
      return { success: true } as const;
    } catch (error) {
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
