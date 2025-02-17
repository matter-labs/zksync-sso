import type { Account, Address, Chain, Client, Transport } from "viem";
import { hexToBytes } from "viem";
import { waitForTransactionReceipt } from "viem/actions";
import { GuardianRecoveryModuleAbi } from "zksync-sso/abi";
import { confirmGuardian as sdkConfirmGuardian, initRecovery as sdkInitRecovery } from "zksync-sso/client";

const getGuardiansInProgress = ref(false);
const getGuardiansError = ref<Error | null>(null);
const getGuardiansData = ref<readonly { addr: Address; isReady: boolean }[] | null>(null);

export const useRecoveryGuardian = () => {
  const { getClient, getPublicClient, getRecoveryClient, defaultChain } = useClientStore();
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
      return data;
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
    return await client.proposeGuardian({
      newGuardian: address,
      paymaster: {
        address: paymasterAddress,
      },
    });
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

  const { inProgress: confirmGuardianInProgress, error: confirmGuardianError, execute: confirmGuardian } = useAsync(async <transport extends Transport, chain extends Chain, account extends Account>({ client, accountToGuard }: { client: Client<transport, chain, account>; accountToGuard: Address }) => {
    return await sdkConfirmGuardian(client, {
      accountToGuard,
      contracts: {
        recovery: contractsByChain[defaultChain.id].recovery,
      },
      paymaster: {
        address: paymasterAddress,
      },
    });
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
    const tx = await client.writeContract({
      address: contractsByChain[defaultChain.id].recovery,
      abi: GuardianRecoveryModuleAbi,
      functionName: "discardRecovery",
    });

    const transactionReceipt = await waitForTransactionReceipt(client, { hash: tx });
    if (transactionReceipt.status !== "success") {
      throw new Error("Account recovery transaction reverted");
    };
  });

  const { inProgress: initRecoveryInProgress, error: initRecoveryError, execute: initRecovery } = useAsync(async <transport extends Transport, chain extends Chain, account extends Account>({ accountToRecover, credentialPublicKey, accountId, client }: { accountToRecover: Address; credentialPublicKey: Uint8Array<ArrayBufferLike>; accountId: string; client: Client<transport, chain, account> }) => {
    return await sdkInitRecovery(client, {
      accountId,
      expectedOrigin: window.location.origin,
      credentialPublicKey,
      contracts: {
        recovery: contractsByChain[defaultChain.id].recovery,
      },
      account: accountToRecover,
      paymaster: {
        address: paymasterAddress,
      },
    });
  });

  const { inProgress: checkRecoveryRequestInProgress, error: checkRecoveryRequestError, execute: checkRecoveryRequest } = useAsync(async (accountId: string) => {
    const client = getPublicClient({ chainId: defaultChain.id });
    const tx = await client.readContract({
      address: contractsByChain[defaultChain.id].recovery,
      abi: GuardianRecoveryModuleAbi,
      functionName: "checkRecoveryRequest",
      args: [accountId],
    });
    return tx;
  });

  const { inProgress: executeRecoveryInProgress, error: executeRecoveryError, execute: executeRecovery } = useAsync(async (address: Address) => {
    const recoveryClient = getRecoveryClient({ chainId: defaultChain.id, address });
    const pendingRecovery = await getPendingRecoveryData(address);

    const tx = await recoveryClient.addAccountOwnerPasskey({
      passkeyPublicKey: hexToBytes(pendingRecovery![0]!),
      paymaster: {
        address: paymasterAddress,
      },
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
    checkRecoveryRequestInProgress,
    checkRecoveryRequestError,
    checkRecoveryRequest,
    executeRecoveryInProgress,
    executeRecoveryError,
    executeRecovery,
  };
};
