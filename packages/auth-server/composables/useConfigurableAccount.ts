import type { Address } from "viem";
import { FactoryAbi } from "zksync-sso/abi";
import { fetchAccount } from "zksync-sso/client";

export const useConfigurableAccount = () => {
  const { getPublicClient, getConfigurableClient, defaultChain } = useClientStore();

  const { inProgress: getConfigurableAccountInProgress, error: getConfigurableAccountError, execute: getConfigurableAccount } = useAsync(async ({ address }: { address: Address }) => {
    const publicClient = getPublicClient({ chainId: defaultChain.id });
    const factoryAddress = contractsByChain[defaultChain.id].accountFactory;

    const accountId = await publicClient.readContract({
      address: factoryAddress,
      abi: FactoryAbi,
      functionName: "accountIds",
      args: [address],
    });
    if (!accountId) {
      throw new Error("Account not found");
    }

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const { username, passkeyPublicKey } = await fetchAccount(publicClient as any, {
      contracts: contractsByChain[defaultChain.id],
      uniqueAccountId: accountId,
    });

    const client = getConfigurableClient({
      chainId: defaultChain.id,
      address,
      credentialPublicKey: passkeyPublicKey,
      username,
    });
    return client;
  });

  return {
    getConfigurableAccountInProgress,
    getConfigurableAccountError,
    getConfigurableAccount,
  };
};
