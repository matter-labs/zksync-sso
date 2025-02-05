import { type Address, createClient, encodeAbiParameters, encodeFunctionData, hashMessage, hashTypedData, http, publicActions, walletActions } from "viem";
import { toAccount } from "viem/accounts";
import { eip712WalletActions, getGeneralPaymasterInput, sendTransaction, serializeTransaction, type ZksyncTransactionSerializableEIP712 } from "viem/zksync";
import { GuardianRecoveryModuleAbi } from "zksync-sso/abi";
import { getEip712Domain } from "zksync-sso/client";

const getGuardiansInProgress = ref(false);
const getGuardiansError = ref<Error | null>(null);
const getGuardiansData = ref<readonly { addr: Address; isReady: boolean }[] | null>(null);

export const useRecoveryGuardian = () => {
  const { getClient, getPublicClient, getWalletClient, getThrowAwayClient, defaultChain } = useClientStore();
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

    console.log(passKey);
    const tx = await client.writeContract({
      account: address,
      address: contractsByChain[defaultChain.id].recovery,
      abi: GuardianRecoveryModuleAbi,
      functionName: "initRecovery",
      args: [account, passKey, accountId],
    });
    return tx;
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
    const throwAwayClient = await getThrowAwayClient({ chainId: defaultChain.id });
    const pendingRecovery = await getPendingRecoveryData(address);

    const sign = async ({ hash: digest }: { hash: `0x${string}` }) => {
      return encodeAbiParameters(
        [{ type: "bytes" }, { type: "address" }, { type: "bytes" }],
        [
          await throwAwayClient.account.sign({ hash: digest }),
          contractsByChain[defaultChain.id].recovery,
          encodeAbiParameters(
            [{ type: "uint256" }],
            [123n],
          ),
        ],
      );
    };
    const account = toAccount({
      address,
      type: "local",
      async signTransaction(transaction) {
        const signableTransaction = {
          ...transaction,
          from: address!,
          type: "eip712",
        } as ZksyncTransactionSerializableEIP712;

        const eip712DomainAndMessage = getEip712Domain(signableTransaction);
        const digest = hashTypedData(eip712DomainAndMessage);

        const data = await sign({ hash: digest });
        return serializeTransaction({
          ...signableTransaction,
          customSignature: data,
        });
      },

      sign,
      async signMessage({ message }) {
        return sign({
          hash: hashMessage(message),
        });
      },
      async signTypedData(typedData) {
        return sign({
          hash: hashTypedData(typedData),
        });
      },
    });

    const client = await createClient({
      chain: throwAwayClient.chain,
      transport: http(),
      account,
      type: "local",
    })
      .extend(publicActions)
      .extend(walletActions)
      .extend(eip712WalletActions());

    const callData = encodeFunctionData({
      abi: GuardianRecoveryModuleAbi,
      functionName: "addValidationKey",
      args: [pendingRecovery![0]!],
    });
    const tx = await sendTransaction(client, {
      account,
      paymaster: contractsByChain[defaultChain.id].accountPaymaster,
      paymasterInput: getGeneralPaymasterInput({ innerInput: "0x" }),
      to: contractsByChain[defaultChain.id].passkey,
      data: callData,
      type: "eip712",
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
