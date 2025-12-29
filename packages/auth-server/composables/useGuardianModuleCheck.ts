/**
 * Composable for checking if the guardian module is installed on an account
 */

import type { Address } from "viem";
import { isGuardianModuleInstalled } from "zksync-sso-4337";

export const useGuardianModuleCheck = () => {
  const { getPublicClient, defaultChain, contractsByChain } = useClientStore();
  const contracts = contractsByChain[defaultChain!.id];

  const checkInProgress = ref(false);
  const checkError = ref<Error | null>(null);
  const isInstalled = ref<boolean | null>(null);

  /**
   * Check if the guardian executor module is installed on the given account
   *
   * This is useful for verifying that accounts deployed on testnet have the
   * guardian recovery module properly configured.
   *
   * @param accountAddress - The smart account address to check
   * @returns Promise that resolves to true if module is installed, false otherwise
   */
  async function checkGuardianModuleInstalled(accountAddress: Address): Promise<boolean> {
    checkInProgress.value = true;
    checkError.value = null;
    isInstalled.value = null;

    try {
      if (!contracts.guardianExecutor) {
        throw new Error("GuardianExecutor contract address not configured for this chain");
      }

      const client = getPublicClient({ chainId: defaultChain.id });

      const result = await isGuardianModuleInstalled({
        client,
        accountAddress,
        guardianExecutorAddress: contracts.guardianExecutor,
      });

      isInstalled.value = result.isInstalled;
      return result.isInstalled;
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      checkError.value = error;
      throw error;
    } finally {
      checkInProgress.value = false;
    }
  }

  return {
    checkGuardianModuleInstalled,
    checkInProgress: readonly(checkInProgress),
    checkError: readonly(checkError),
    isInstalled: readonly(isInstalled),
  };
};
