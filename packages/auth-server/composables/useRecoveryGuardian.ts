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

  const getRecoveryInProgress = ref(false);
  const getRecoveryError = ref<Error | null>(null);

  async function getRecovery(account: Address) {
    getRecoveryInProgress.value = true;
    getRecoveryError.value = null;

    try {
      const client = getPublicClient({ chainId: defaultChain.id });
      return await client.readContract({
        address: contractsByChain[defaultChain.id].recovery,
        abi: GuardianRecoveryModuleAbi,
        functionName: "pendingRecoveryData",
        args: [account],
      });
    } catch (err) {
      getRecoveryError.value = err as Error;
      return [];
    } finally {
      getRecoveryInProgress.value = false;
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
    const client = getWalletClient({ chainId: defaultChain.id });
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

  const { inProgress: getPendingRecoveryDataInProgress, error: getPendingRecoveryDataError, execute: getPendingRecoveryData, result: getPendingRecoveryDataResult } = useAsync(async (account: Address) => {
    const client = getPublicClient({ chainId: defaultChain.id });
    return await client.readContract({
      address: contractsByChain[defaultChain.id].recovery,
      abi: GuardianRecoveryModuleAbi,
      functionName: "pendingRecoveryData",
      args: [account],
    });
  });

  const { inProgress: discardRecoveryInProgress, error: discardRecoveryError, execute: discardRecovery } = useAsync(async () => {
    const client = getClient({ chainId: defaultChain.id });
    return await client.writeContract({
      address: contractsByChain[defaultChain.id].recovery,
      abi: GuardianRecoveryModuleAbi,
      functionName: "discardRecovery",
    });
  });

  const { inProgress: initRecoveryInProgress, error: initRecoveryError, execute: initRecovery } = useAsync(async (account: Address, passKey: `0x${string}`, accountId: string) => {
    const client = await getWalletClient({ chainId: defaultChain.id });
    const [address] = await client.getAddresses();

    const tx = await client.writeContract({
      account: address,
      address: contractsByChain[defaultChain.id].recovery,
      abi: GuardianRecoveryModuleAbi,
      functionName: "initRecovery",
      args: [account, passKey, accountId],
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
    initRecoveryInProgress,
    initRecoveryError,
    initRecovery,
    getGuardedAccountsInProgress,
    getGuardedAccountsError,
    getGuardedAccounts,
    getGuardiansInProgress,
    getGuardiansError,
    getGuardiansData,
    getGuardians,
    getPendingRecoveryDataInProgress,
    getPendingRecoveryDataError,
    getPendingRecoveryData,
    getPendingRecoveryDataResult,
    discardRecoveryInProgress,
    discardRecoveryError,
    discardRecovery,
    getRecoveryInProgress,
    getRecoveryError,
    getRecovery,
  };
};
