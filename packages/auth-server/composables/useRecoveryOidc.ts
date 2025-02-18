import type { OidcData } from "zksync-sso/client";

export const useRecoveryOidc = () => {
  const { getClient, defaultChain } = useClientStore();
  const paymasterAddress = contractsByChain[defaultChain!.id].accountPaymaster;

  const { inProgress: isLoading, error, execute: addOidcAccount } = useAsync(async (oidcData: OidcData) => {
    const client = getClient({ chainId: defaultChain.id });

    return await client.addOidcAccount({
      paymaster: {
        address: paymasterAddress,
      },
      oidcData,
    });
  });

  return {
    addOidcAccount,
    isLoading,
    error,
  };
};
