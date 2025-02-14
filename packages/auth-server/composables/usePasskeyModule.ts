import type { Address } from "viem";
import { WebAuthModuleAbi } from "zksync-sso/abi";

const getPasskeysInProgress = ref(false);
const getPasskeysError = ref<Error | null>(null);
const getPasskeysData = ref<readonly `0x${string}`[] | null>(null);

export const usePasskeyModule = () => {
  const { getPublicClient, getClient, defaultChain } = useClientStore();
  const paymasterAddress = contractsByChain[defaultChain!.id].accountPaymaster;

  async function getPasskeys(origin: string, accountAddress: Address) {
    getPasskeysInProgress.value = true;
    getPasskeysError.value = null;

    try {
      const client = getPublicClient({ chainId: defaultChain.id });
      const data = await client.readContract({
        address: contractsByChain[defaultChain.id].passkey,
        abi: WebAuthModuleAbi,
        functionName: "getDomainAccountKeys",
        args: [origin, accountAddress],
      });
      getPasskeysData.value = data;
    } catch (err) {
      getPasskeysError.value = err as Error;
      return [];
    } finally {
      getPasskeysInProgress.value = false;
    }
  }

  const { inProgress: removePasskeyInProgress, error: removePasskeyError, execute: removePasskey } = useAsync(async (origin: string, credentialId: `0x${string}`) => {
    const client = getClient({ chainId: defaultChain.id });
    const tx = await client.removePasskey({
      domain: origin,
      credentialId,
      paymaster: {
        address: paymasterAddress,
      },
    });
    getPasskeys(origin, client.account?.address);
    return tx;
  });

  return {
    getPasskeys,
    getPasskeysData,
    getPasskeysInProgress,
    getPasskeysError,
    removePasskey,
    removePasskeyInProgress,
    removePasskeyError,
  };
};
