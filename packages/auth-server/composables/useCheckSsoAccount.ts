import type { Address } from "viem";
import { AAFactoryAbi } from "zksync-sso-4337/abi";

export const useCheckSsoAccount = () => {
  const { getThrowAwayClient, contracts } = useClientStore();

  const { inProgress: isLoading, error, execute: checkIsSsoAccount } = useAsync(async (accountId: Address): Promise<boolean> => {
    const client = getThrowAwayClient();
    const factoryAddress = contracts.factory;

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
