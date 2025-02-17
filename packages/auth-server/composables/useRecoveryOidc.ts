import { OidcRecoveryModuleAbi } from "zksync-sso/abi";

export const useRecoveryOidc = () => {
  const { getClient, defaultChain } = useClientStore();
  const paymasterAddress = contractsByChain[defaultChain!.id].accountPaymaster;

  const { inProgress: isLoading, error, execute: addOidcAccount } = useAsync(async () => {
    const client = getClient({ chainId: defaultChain.id });
    return await client.addOidcAccount({
      paymaster: {
        address: paymasterAddress,
      }
    });
  });

  return {
    addOidcAccount,
    isLoading,
    error,
  };
};
