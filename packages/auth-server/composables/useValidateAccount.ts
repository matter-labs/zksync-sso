import type { Address } from "viem";
import { ref } from "vue";

import { useClientStore } from "~/stores/client";

export function useValidateAccount() {
  const { getPublicClient, defaultChain } = useClientStore();
  const runtimeConfig = useRuntimeConfig();

  const isValidatingAccount = ref(false);
  const error = ref<Error | null>(null);

  const validateAccount = async (address: Address): Promise<boolean> => {
    isValidatingAccount.value = true;
    error.value = null;

    try {
      const publicClient = getPublicClient({ chainId: defaultChain.id });

      const isSSO = await publicClient.readContract({
        address,
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
      return isSSO;
    } catch (err) {
      error.value = err as Error;
      return false;
    } finally {
      isValidatingAccount.value = false;
    }
  };

  return {
    validateAccount,
    isValidatingAccount,
    error,
  };
}
