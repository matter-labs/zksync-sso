import type { Address } from "viem";

export function useIsSsoAccount() {
  const { getPublicClient, defaultChain } = useClientStore();
  const runtimeConfig = useRuntimeConfig();

  const { inProgress: isLoading, error, execute: isSsoAccount } = useAsync(async (accountAddress: Address): Promise<boolean> => {
    const publicClient = getPublicClient({ chainId: defaultChain.id });

    try {
      const result = await publicClient.readContract({
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

      return result;
    } catch (err: unknown) {
      // Handle NoFallbackHandler error (0x48c9ceda) - ModularSmartAccount doesn't implement supportsInterface yet
      // WORKAROUND: In our dev environment, all accounts deployed via auth-server-api are ERC-4337 ModularSmartAccounts
      // that throw this error. We treat these as SSO accounts.
      // Check both the error message and the full error string representation
      const errorString = err.toString?.() || String(err);
      const errorMessage = err.message || "";

      if (errorMessage.includes("0x48c9ceda") || errorMessage.includes("NoFallbackHandler")
        || errorString.includes("0x48c9ceda") || errorString.includes("NoFallbackHandler")) {
        return true;
      }

      // For other errors, assume not an SSO account
      return false;
    }
  });

  return {
    isSsoAccount,
    isLoading,
    error,
  };
};
