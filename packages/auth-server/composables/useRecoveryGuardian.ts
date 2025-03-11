import type { Account, Address, Chain, Client, Hex, Transport } from "viem";
import { encodeFunctionData, keccak256, pad, toHex } from "viem";
import { waitForTransactionReceipt } from "viem/actions";
import { getGeneralPaymasterInput, sendTransaction } from "viem/zksync";
import { GuardianRecoveryModuleAbi } from "zksync-sso/abi";
import { confirmGuardian as sdkConfirmGuardian } from "zksync-sso/client";
import { base64UrlToUint8Array, getPublicKeyBytesFromPasskeySignature } from "zksync-sso/utils";

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
        args: [keccak256(toHex(window.location.origin)), guardianAddress],
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
        args: [keccak256(toHex(window.location.origin)), guardedAccount],
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
        functionName: "getPendingRecoveryData",
        args: [keccak256(toHex(window.location.origin)), account],
      });
    } catch (err) {
      getRecoveryError.value = err as Error;
      return null;
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
    await waitForTransactionReceipt(client, { hash: tx.transactionReceipt.transactionHash, confirmations: 1 });
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
    await waitForTransactionReceipt(client, { hash: tx.transactionReceipt.transactionHash, confirmations: 1 });
    getGuardians(client.account.address);
    return tx;
  });

  const { inProgress: confirmGuardianInProgress, error: confirmGuardianError, execute: confirmGuardian } = useAsync(async <transport extends Transport, chain extends Chain, account extends Account>({ client, accountToGuard }: { client: Client<transport, chain, account>; accountToGuard: Address }) => {
    const { transactionReceipt } = await sdkConfirmGuardian(client, {
      accountToGuard,
      contracts: {
        recovery: contractsByChain[defaultChain.id].recovery,
      },
      paymaster: {
        address: paymasterAddress,
      },
    });
    await waitForTransactionReceipt(client, { hash: transactionReceipt.transactionHash, confirmations: 1 });
    return { transactionReceipt };
  });

  const { inProgress: discardRecoveryInProgress, error: discardRecoveryError, execute: discardRecovery } = useAsync(async () => {
    const client = getClient({ chainId: defaultChain.id });
    const tx = await client.writeContract({
      address: contractsByChain[defaultChain.id].recovery,
      abi: GuardianRecoveryModuleAbi,
      functionName: "discardRecovery",
      args: [keccak256(toHex(window.location.origin))],
    });

    const transactionReceipt = await waitForTransactionReceipt(client, { hash: tx });
    if (transactionReceipt.status !== "success") {
      throw new Error("Account recovery transaction reverted");
    };
  });

  const { inProgress: initRecoveryInProgress, error: initRecoveryError, execute: initRecovery } = useAsync(async <transport extends Transport, chain extends Chain, account extends Account>({ accountToRecover, credentialPublicKey, accountId, client }: { accountToRecover: Address; credentialPublicKey: Uint8Array<ArrayBufferLike>; accountId: string; client: Client<transport, chain, account> }) => {
    const publicKeyBytes = getPublicKeyBytesFromPasskeySignature(credentialPublicKey);
    const publicKeyHex = [
      pad(`0x${publicKeyBytes[0].toString("hex")}`),
      pad(`0x${publicKeyBytes[1].toString("hex")}`),
    ] as const;

    const calldata = encodeFunctionData({
      abi: GuardianRecoveryModuleAbi,
      functionName: "initRecovery",
      args: [
        accountToRecover,
        keccak256(toHex(base64UrlToUint8Array(accountId))),
        publicKeyHex,
        keccak256(toHex(window.location.origin)),
      ],
    });

    const sendTransactionArgs = {
      account: client.account,
      to: contractsByChain[defaultChain.id].recovery,
      paymaster: contractsByChain[defaultChain!.id].accountPaymaster,
      paymasterInput: getGeneralPaymasterInput({ innerInput: "0x" }),
      data: calldata,
      gas: 10_000_000n, // TODO: Remove when gas estimation is fixed
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    } as any;
    const tx = await sendTransaction(client, sendTransactionArgs);
    await waitForTransactionReceipt(client, { hash: tx });
    return tx;
  });

  const { inProgress: checkRecoveryRequestInProgress, error: checkRecoveryRequestError, execute: checkRecoveryRequest } = useAsync(async ({ credentialId, address }: { credentialId?: string; address?: Address }) => {
    const client = getPublicClient({ chainId: defaultChain.id });
    const [requestValidityTime, requestDelayTime] = await Promise.all([
      client.readContract({
        address: contractsByChain[defaultChain.id].recovery,
        abi: GuardianRecoveryModuleAbi,
        functionName: "REQUEST_VALIDITY_TIME",
        args: [],
      }),
      client.readContract({
        address: contractsByChain[defaultChain.id].recovery,
        abi: GuardianRecoveryModuleAbi,
        functionName: "REQUEST_DELAY_TIME",
        args: [],
      }),
    ]);

    // Calculate the delay and validity times in blocks
    const blockTime = chainParameters[defaultChain.id].blockTime;
    const delayBlocks = requestDelayTime / BigInt(blockTime);
    const validityBlocks = requestValidityTime / BigInt(blockTime);

    // Blocks that mark the start of the recovery request after which the request
    // is valid and after which the request can not be executed yet
    const currentBlock = await client.getBlockNumber();
    const calculatedValidFromBlock = currentBlock - validityBlocks;
    const validFromBlock = calculatedValidFromBlock < 0n ? 0n : calculatedValidFromBlock;

    const args: { account?: Address; hashedCredentialId?: Hex; hashedOriginDomain: Hex } = {
      hashedOriginDomain: keccak256(toHex(window.location.origin)),
    };
    if (address) {
      args.account = address;
    }
    if (credentialId) {
      args.hashedCredentialId = keccak256(toHex(base64UrlToUint8Array(credentialId)));
    }

    const eventsFilter = {
      address: contractsByChain[defaultChain.id].recovery,
      abi: GuardianRecoveryModuleAbi,
      args,
      fromBlock: validFromBlock,
      toBlock: "latest",
      strict: true,
    } as const;

    const [initiatedEvents, finishedEvents, discardedEvents] = await Promise.all([
      client.getContractEvents({
        ...eventsFilter,
        eventName: "RecoveryInitiated",
      }),
      client.getContractEvents({
        ...eventsFilter,
        eventName: "RecoveryFinished",
      }),
      client.getContractEvents({
        ...eventsFilter,
        eventName: "RecoveryDiscarded",
      }),
    ]);

    if (initiatedEvents.length === 0) {
      return { pendingRecovery: false } as const;
    }

    const activeRecoveryEvents = initiatedEvents.filter((initEvent) => {
      const isFinished = finishedEvents.some((finishEvent) =>
        finishEvent.args.account === initEvent.args.account
        && finishEvent.args.hashedOriginDomain === initEvent.args.hashedOriginDomain
        && finishEvent.args.hashedCredentialId === initEvent.args.hashedCredentialId
        && finishEvent.blockNumber >= initEvent.blockNumber,
      );

      const isDiscarded = discardedEvents.some((discardEvent) =>
        discardEvent.args.account === initEvent.args.account
        && discardEvent.args.hashedOriginDomain === initEvent.args.hashedOriginDomain
        && discardEvent.args.hashedCredentialId === initEvent.args.hashedCredentialId
        && discardEvent.blockNumber >= initEvent.blockNumber,
      );

      return !isFinished && !isDiscarded;
    });

    if (activeRecoveryEvents.length === 0) {
      return { pendingRecovery: false } as const;
    }

    // From here on, we assume there's only one valid event, the last one.
    // This is because recovery is overwritten and only one recovery can be active at a time.
    const event = activeRecoveryEvents[activeRecoveryEvents.length - 1];
    const recoveryDelayFromBlock = event.blockNumber + delayBlocks; // Block from which the recovery can be executed
    const recoveryValidityFromBlock = event.blockNumber + validityBlocks; // Block from which the recovery can no longer be executed
    const remainingBlocks = recoveryDelayFromBlock - currentBlock;
    const remainingTime = remainingBlocks * BigInt(blockTime);

    if (currentBlock > recoveryValidityFromBlock) {
      return { pendingRecovery: false } as const;
    }

    return {
      pendingRecovery: true,
      ready: currentBlock >= recoveryDelayFromBlock,
      remainingTime: remainingTime < 0 ? 0n : remainingTime,
      accountAddress: event.args.account,
      guardianAddress: event.args.guardian,
    } as const;
  });

  const { inProgress: executeRecoveryInProgress, error: executeRecoveryError, execute: executeRecovery } = useAsync(async ({ accountAddress, credentialId, rawPublicKey }: { accountAddress: Address; credentialId: string; rawPublicKey: readonly [Hex, Hex] }) => {
    const recoveryClient = getRecoveryClient({ chainId: defaultChain.id, address: accountAddress });
    return await recoveryClient.addAccountOwnerPasskey({
      credentialId,
      rawPublicKey,
      origin: window.location.origin,
      paymaster: {
        address: paymasterAddress,
      },
    });
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
