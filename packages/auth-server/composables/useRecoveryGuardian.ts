import { type Address, encodeAbiParameters } from "viem";
import { GuardianRecoveryModuleAbi } from "zksync-sso/abi";

const getGuardiansInProgress = ref(false);
const getGuardiansError = ref<Error | null>(null);
const getGuardiansData = ref<readonly { addr: Address; isReady: boolean }[] | null>(null);

export const useRecoveryGuardian = () => {
  const { getClient, getPublicClient, getWalletClient, defaultChain } = useClientStore();
  const paymasterAddress = contractsByChain[defaultChain!.id].accountPaymaster;

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

  async function getGuardians(guardedAccount: Address) {
    getGuardiansInProgress.value = true;
    getGuardiansError.value = null;

    try {
      const client = getPublicClient({ chainId: defaultChain.id });
      const data = await client.readContract({
        address: contractsByChain[defaultChain.id].recovery,
        abi: GuardianRecoveryModuleAbi,
        functionName: "guardiansFor",
        args: [guardedAccount],
      });
      getGuardiansData.value = data;
      return;
    } catch (err) {
      getGuardiansError.value = err as Error;
      return [];
    } finally {
      getGuardiansInProgress.value = false;
    }
  }

  const { inProgress: proposeGuardianInProgress, error: proposeGuardianError, execute: proposeGuardian } = useAsync(async (address: Address) => {
    const client = getClient({ chainId: defaultChain.id });
    const tx = await client.proposeGuardian({
      newGuardian: address,
      paymaster: {
        address: paymasterAddress,
      },
    });
    await getGuardians(client.account.address);
    return tx;
  });

  const { inProgress: removeGuardianInProgress, error: removeGuardianError, execute: removeGuardian } = useAsync(async (address: Address) => {
    const client = getClient({ chainId: defaultChain.id });
    const tx = await client.removeGuardian({
      guardian: address,
      paymaster: {
        address: paymasterAddress,
      },
    });
    getGuardians(client.account.address);
    return tx;
  });

  const { inProgress: confirmGuardianInProgress, error: confirmGuardianError, execute: confirmGuardian } = useAsync(async (account: Address) => {
    const client = await getWalletClient({ chainId: defaultChain.id });
    const [address] = await client.getAddresses();
    const tx = await client.writeContract({
      account: address,
      address: contractsByChain[defaultChain.id].recovery,
      abi: GuardianRecoveryModuleAbi,
      functionName: "addValidationKey",
      args: [encodeAbiParameters([{ type: "address" }], [account])],
    });
    return tx;
  });

  return {
    confirmGuardianInProgress,
    confirmGuardianError,
    confirmGuardian,
    proposeGuardianInProgress,
    proposeGuardianError,
    proposeGuardian,
    removeGuardianInProgress,
    removeGuardianError,
    removeGuardian,
    getGuardedAccountsInProgress,
    getGuardedAccountsError,
    getGuardedAccounts,
    getGuardiansInProgress,
    getGuardiansError,
    getGuardiansData,
    getGuardians,
  };
};
