import type { Address } from "viem";
import { GuardianRecoveryModuleAbi } from "zksync-sso/abi";

export const useRecoveryGuardian = () => {
  const { getPublicClient, defaultChain } = useClientStore();

  const getGuardedAccountsInProgress = ref(false);
  const getGuardedAccountsError = ref<Error | null>(null);

  async function getGuardedAccounts(guardianAddress: Address) {
    getGuardedAccountsInProgress.value = true;
    getGuardedAccountsError.value = null;

    try {
      const client = getPublicClient({ chainId: defaultChain.id });
      return await client.readContract({
        address: contractsByChain[defaultChain.id].recovery,
        abi: GuardianRecoveryModuleAbi,
        functionName: "guardianOf",
        args: [guardianAddress],
      });
    } catch (err) {
      getGuardedAccountsError.value = err as Error;
      return [];
    } finally {
      getGuardedAccountsInProgress.value = false;
    }
  }

  return {
    getGuardedAccountsInProgress,
    getGuardedAccountsError,
    getGuardedAccounts,
  };
};
