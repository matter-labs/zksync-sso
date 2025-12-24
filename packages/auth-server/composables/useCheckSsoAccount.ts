import type { Address } from "viem";
import { AAFactoryAbi } from "zksync-sso-4337/abi";

export const useCheckSsoAccount = (_chainId: MaybeRef<SupportedChainId>) => {
  const chainId = toRef(_chainId);
  const { getThrowAwayClient } = useClientStore();

  const { inProgress: isLoading, error, execute: checkIsSsoAccount } = useAsync(async (accountId: Address): Promise<boolean> => {
    const client = getThrowAwayClient({ chainId: chainId.value });
    const factoryAddress = contractsByChain[chainId.value].accountFactory;

    const guardianAddress = await client.readContract({
      address: factoryAddress,
      abi: AAFactoryAbi,
      functionName: "accountMappings",
      args: [accountId],
    });

    return guardianAddress !== "0x0000000000000000000000000000000000000000";
  });

  return {
    checkIsSsoAccount,
    isLoading,
    error,
  };
};
