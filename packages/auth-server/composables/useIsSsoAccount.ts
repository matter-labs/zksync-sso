import type { Address } from "viem";

export function useIsSsoAccount() {
  const { getPublicClient, defaultChain } = useClientStore();
  const runtimeConfig = useRuntimeConfig();

  const { inProgress: isLoading, error, execute: isSsoAccount } = useAsync(async (accountAddress: Address): Promise<boolean> => {
    const publicClient = getPublicClient({ chainId: defaultChain.id });

    try {
      return await publicClient.readContract({
        address: accountAddress,
        abi: [{
          type: "function",
          name: "supportsInterface",
          inputs: [{ type: "bytes4", name: "interfaceId" }],
          outputs: [{ type: "bool" }],
          stateMutability: "view",
        }],
        functionName: "supportsInterface",
        args: [runtimeConfig.public.ssoAccountInterfaceId as Address],
      });
      ;
    } catch {
      return false;
    }
  });

  return {
    isSsoAccount,
    isLoading,
    error,
  };
};
