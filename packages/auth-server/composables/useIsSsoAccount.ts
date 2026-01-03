import type { Address } from "viem";

export function useIsSsoAccount() {
  const { getPublicClient, defaultChain } = useClientStore();
  const runtimeConfig = useRuntimeConfig();

  const { inProgress: isLoading, error, execute: isSsoAccount } = useAsync(async (accountAddress: Address): Promise<boolean> => {
    const publicClient = getPublicClient({ chainId: defaultChain.id });

    try {
      console.log("[useIsSsoAccount] Checking if address is SSO account:", accountAddress);
      console.log("[useIsSsoAccount] SSO Interface ID:", runtimeConfig.public.ssoAccountInterfaceId);

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

      console.log("[useIsSsoAccount] supportsInterface result:", result);
      return result;
    } catch (err: unknown) {
      console.error("[useIsSsoAccount] Error checking SSO account:", err);

      // Handle NoFallbackHandler error (0x48c9ceda) - ModularSmartAccount doesn't implement supportsInterface yet
      // WORKAROUND: In our dev environment, all accounts deployed via auth-server-api are ERC-4337 ModularSmartAccounts
      // that throw this error. We treat these as SSO accounts.
      // Check both the error message and the full error string representation
      const errorString = err.toString?.() || String(err);
      const errorMessage = err.message || "";

      console.log("[useIsSsoAccount] DEBUG - Error details:");
      console.log("  errorMessage includes 0x48c9ceda?", errorMessage.includes("0x48c9ceda"));
      console.log("  errorString includes 0x48c9ceda?", errorString.includes("0x48c9ceda"));
      console.log("  errorMessage:", errorMessage.substring(0, 200));
      console.log("  errorString:", errorString.substring(0, 200));

      if (errorMessage.includes("0x48c9ceda") || errorMessage.includes("NoFallbackHandler")
        || errorString.includes("0x48c9ceda") || errorString.includes("NoFallbackHandler")) {
        console.log("[useIsSsoAccount] ✅ MATCHED! Account has no fallback handler - this is a ModularSmartAccount (ERC-4337), treating as SSO");
        return true;
      }

      // For other errors, assume not an SSO account
      console.warn("[useIsSsoAccount] Unknown error checking supportsInterface, assuming not SSO");
      return false;
    }
  });

  return {
    isSsoAccount,
    isLoading,
    error,
  };
};
