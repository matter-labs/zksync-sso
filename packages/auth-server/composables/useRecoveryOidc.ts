import type { Address, Hex } from "viem";
import { OidcRecoveryModuleAbi } from "zksync-sso/abi";
import { type OidcData, type ParsedOidcData, parseOidcData } from "zksync-sso/client";
import { ByteVector, type JWT, OidcDigest } from "zksync-sso-circuits";

export const useRecoveryOidc = () => {
  const { getClient, getPublicClient, defaultChain } = useClientStore();
  const {
    public: { saltServiceUrl },
  } = useRuntimeConfig();
  const paymasterAddress = contractsByChain[defaultChain!.id].accountPaymaster;

  const getOidcAccountsInProgress = ref(false);
  const getOidcAccountsError = ref<Error | null>(null);
  const getOidcAccountsData = ref<readonly ParsedOidcData[] | null>(null);

  async function buildOidcDigest(jwt: JWT): Promise<Hex> {
    const response = await fetch(saltServiceUrl, {
      method: "GET",
      headers: {
        Authorization: `Bearer ${jwt.raw}`,
      },
    })
      .then((res) => res.json());

    const salt = response.salt;
    return new OidcDigest(jwt.iss, jwt.aud, jwt.sub, ByteVector.fromHex(salt)).toHex();
  }

  async function getOidcAccounts(oidcAddress: Address) {
    getOidcAccountsInProgress.value = true;
    getOidcAccountsError.value = null;

    try {
      const client = getPublicClient({ chainId: defaultChain.id });
      const data = await client.readContract({
        address: contractsByChain[defaultChain.id].recoveryOidc,
        abi: OidcRecoveryModuleAbi,
        functionName: "oidcDataForAddress",
        args: [oidcAddress],
      });
      getOidcAccountsData.value = data.map(parseOidcData);
      return;
    } catch (err) {
      getOidcAccountsError.value = err as Error;
      return;
    } finally {
      getOidcAccountsInProgress.value = false;
    }
  }

  const {
    inProgress: addOidcAccountIsLoading,
    error: addOidcAccountError,
    execute: addOidcAccount,
  } = useAsync(async (oidcData: OidcData) => {
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
    buildOidcDigest,
    addOidcAccount,
    addOidcAccountIsLoading,
    addOidcAccountError,
  };
};
