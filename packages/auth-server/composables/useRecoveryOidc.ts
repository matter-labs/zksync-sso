import { OidcRecoveryModuleAbi } from "zksync-sso/abi";

export const useRecoveryOidc = () => {
  const { getClient, getPublicClient, getWalletClient, getRecoveryClient, defaultChain } = useClientStore();


  const getOidcAccountsInProgress = ref(false);
  const getOidcAccountsError = ref<Error | null>(null);

  async function getOidcAccounts() {
    getOidcAccountsInProgress.value = true;
    getOidcAccountsError.value = null;

    try {
      const client = getPublicClient({ chainId: defaultChain.id });
      return await client.readContract({
        address: contractsByChain[defaultChain.id].recovery,
        abi: OidcRecoveryModuleAbi,
        functionName: "oidcAccounts",
        args: [],
      });
    } catch (err) {
      getOidcAccountsError.value = err as Error;
      return [];
    } finally {
      getOidcAccountsInProgress.value = false;
    }
  }


  const { inProgress: isLoading, error, execute: addOidcAccount } = useAsync(async () => {
    await new Promise((resolve) => setTimeout(resolve, 1500));
  });

  return {
    addOidcAccount,
    isLoading,
    error,
  };
};
