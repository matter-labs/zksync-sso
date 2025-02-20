import type { Address, Hex } from "viem";
import { OidcRecoveryModuleAbi } from "zksync-sso/abi";
import { type OidcData, type ParsedOidcData, parseOidcData } from "zksync-sso/client";

export const useRecoveryOidc = () => {
  const { getClient, getPublicClient, defaultChain } = useClientStore();
  const paymasterAddress = contractsByChain[defaultChain!.id].accountPaymaster;

  const getOidcAccountsInProgress = ref(false);
  const getOidcAccountsError = ref<Error | null>(null);
  const getOidcAccountsData = ref<readonly ParsedOidcData[] | null>(null);

  async function getOidcAccounts(oidcAddress: Address) {
    getOidcAccountsInProgress.value = true;
    getOidcAccountsError.value = null;

    try {
      const client = getPublicClient({ chainId: defaultChain.id });
      const data = await client.readContract({
        address: contractsByChain[defaultChain.id].recoveryOidc,
        abi: OidcRecoveryModuleAbi,
        functionName: "accountData",
        args: [oidcAddress],
      });
      const oidcData = {
        oidcDigest: data[0] as Hex,
        iss: data[1] as Hex,
        aud: data[2] as Hex,
      };
      getOidcAccountsData.value = [parseOidcData(oidcData)];
      return;
    } catch (err) {
      getOidcAccountsError.value = err as Error;
      return;
    } finally {
      getOidcAccountsInProgress.value = false;
    }
  }

  const { inProgress: addOidcAccountIsLoading, error: addOidcAccountError, execute: addOidcAccount } = useAsync(async (oidcData: OidcData) => {
    const client = getClient({ chainId: defaultChain.id });

    return await client.addOidcAccount({
      paymaster: {
        address: paymasterAddress,
      },
      oidcData,
    });
  });

  return {
    getOidcAccounts,
    getOidcAccountsInProgress,
    getOidcAccountsError,
    getOidcAccountsData,
    addOidcAccount,
    addOidcAccountIsLoading,
    addOidcAccountError,
  };
};
