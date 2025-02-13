import type { Address } from "viem";
import { FactoryAbi } from "zksync-sso/abi";

export const useCheckSsoAccount = (_chainId: MaybeRef<SupportedChainId>) => {
  const chainId = toRef(_chainId);
  const { getThrowAwayClient } = useClientStore();

  const { inProgress: isLoading, error, execute: checkIsSsoAccount } = useAsync(async (guardianAddress: Address): Promise<boolean> => {
    const client = getThrowAwayClient({ chainId: chainId.value });
    const factoryAddress = contractsByChain[chainId.value].accountFactory;

    const accountId = await client.readContract({
      address: factoryAddress,
      abi: FactoryAbi,
      functionName: "accountIds",
      args: [guardianAddress],
    });

    return accountId !== "";
  });

  return {
    checkIsSsoAccount,
    isLoading,
    error,
  };
};
