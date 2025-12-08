import type { Account, Address, Chain, Client, Hex, Transport } from "viem";
import { encodeAbiParameters, encodeFunctionData, keccak256, pad, parseAbiParameters, toHex } from "viem";
import { waitForTransactionReceipt } from "viem/actions";
import { base64UrlToUint8Array, getPublicKeyBytesFromPasskeySignature } from "zksync-sso-4337/utils";

import { GuardianExecutorAbi, RecoveryType } from "~/abi/GuardianExecutorAbi";

const getGuardiansInProgress = ref(false);
const getGuardiansError = ref<Error | null>(null);
const getGuardiansData = ref<readonly { addr: Address; isReady: boolean }[] | null>(null);

export const useRecoveryGuardian = () => {
  const { getClient, getPublicClient, getThrowAwayClient, defaultChain, contractsByChain } = useClientStore();
  const contracts = contractsByChain[defaultChain!.id];

  const getGuardedAccountsInProgress = ref(false);
  const getGuardedAccountsError = ref<Error | null>(null);

  /**
   * Get all accounts that a guardian is guarding by querying GuardianAdded events
   */
  async function getGuardedAccounts(guardianAddress: Address): Promise<Address[]> {
    getGuardedAccountsInProgress.value = true;
    getGuardedAccountsError.value = null;

    try {
      if (!contracts.guardianExecutor) throw new Error("GuardianExecutor contract address not configured");
      const client = getPublicClient({ chainId: defaultChain.id });

      // Query GuardianAdded events where this guardian was added
      const addedEvents = await client.getContractEvents({
        address: contracts.guardianExecutor,
        abi: GuardianExecutorAbi,
        eventName: "GuardianAdded",
        args: {
          guardian: guardianAddress,
        },
        fromBlock: 0n,
        toBlock: "latest",
      });

      // Query GuardianRemoved events where this guardian was removed
      const removedEvents = await client.getContractEvents({
        address: contracts.guardianExecutor,
        abi: GuardianExecutorAbi,
        eventName: "GuardianRemoved",
        args: {
          guardian: guardianAddress,
        },
        fromBlock: 0n,
        toBlock: "latest",
      });

      // Build set of accounts still guarded (added but not removed)
      const accountsMap = new Map<Address, boolean>();

      // Process added events first
      for (const event of addedEvents) {
        if (event.args.account) {
          accountsMap.set(event.args.account, true);
        }
      }

      // Remove accounts where guardian was removed after being added
      for (const event of removedEvents) {
        if (event.args.account) {
          const addedEvent = addedEvents.find(
            (ae) => ae.args.account === event.args.account && ae.blockNumber <= event.blockNumber,
          );
          if (addedEvent) {
            accountsMap.delete(event.args.account);
          }
        }
      }

      return Array.from(accountsMap.keys());
    } catch (err) {
      getGuardedAccountsError.value = err as Error;
      return [];
    } finally {
      getGuardedAccountsInProgress.value = false;
    }
  }

  /**
   * Get all guardians for a given account
   */
  async function getGuardians(guardedAccount: Address) {
    getGuardiansInProgress.value = true;
    getGuardiansError.value = null;

    try {
      if (!contracts.guardianExecutor) throw new Error("GuardianExecutor contract address not configured");
      const client = getPublicClient({ chainId: defaultChain.id });

      // Get list of guardian addresses
      const guardians = await client.readContract({
        address: contracts.guardianExecutor,
        abi: GuardianExecutorAbi,
        functionName: "guardiansFor",
        args: [guardedAccount],
      });

      // For each guardian, get their status to determine if active
      const guardiansWithStatus = await Promise.all(
        guardians.map(async (addr) => {
          const [isPresent, isActive] = await client.readContract({
            address: contracts.guardianExecutor!,
            abi: GuardianExecutorAbi,
            functionName: "guardianStatusFor",
            args: [guardedAccount, addr],
          });
          return { addr, isReady: isPresent && isActive };
        }),
      );

      getGuardiansData.value = guardiansWithStatus;
      return guardiansWithStatus;
    } catch (err) {
      getGuardiansError.value = err as Error;
      return [];
    } finally {
      getGuardiansInProgress.value = false;
    }
  }

  const getRecoveryInProgress = ref(false);
  const getRecoveryError = ref<Error | null>(null);

  /**
   * Get pending recovery request for an account
   */
  async function getRecovery(account: Address) {
    getRecoveryInProgress.value = true;
    getRecoveryError.value = null;

    try {
      if (!contracts.guardianExecutor) throw new Error("GuardianExecutor contract address not configured");
      const client = getPublicClient({ chainId: defaultChain.id });

      const [recoveryType, hashedData, timestamp] = await client.readContract({
        address: contracts.guardianExecutor,
        abi: GuardianExecutorAbi,
        functionName: "pendingRecovery",
        args: [account],
      });

      // RecoveryType 0 means no recovery in progress
      if (recoveryType === RecoveryType.None) {
        return null;
      }

      return { recoveryType, hashedData, timestamp };
    } catch (err) {
      getRecoveryError.value = err as Error;
      return null;
    } finally {
      getRecoveryInProgress.value = false;
    }
  }

  /**
   * Propose a new guardian for the caller's account
   * This is called by the smart account owner via ERC-4337 user operation
   */
  const { inProgress: proposeGuardianInProgress, error: proposeGuardianError, execute: proposeGuardian } = useAsync(async (address: Address) => {
    if (!contracts.guardianExecutor) throw new Error("GuardianExecutor contract address not configured");

    const client = getClient({ chainId: defaultChain.id });

    // Call proposeGuardian on the GuardianExecutor module through the smart account
    const calldata = encodeFunctionData({
      abi: GuardianExecutorAbi,
      functionName: "proposeGuardian",
      args: [address],
    });

    const tx = await client.sendUserOperation({
      calls: [{
        to: contracts.guardianExecutor,
        data: calldata,
      }],
    });

    // Wait for user operation receipt
    const receipt = await client.waitForUserOperationReceipt({ hash: tx });
    return receipt;
  });

  /**
   * Remove an existing guardian from the caller's account
   * This is called by the smart account owner via ERC-4337 user operation
   */
  const { inProgress: removeGuardianInProgress, error: removeGuardianError, execute: removeGuardian } = useAsync(async (address: Address) => {
    if (!contracts.guardianExecutor) throw new Error("GuardianExecutor contract address not configured");

    const client = getClient({ chainId: defaultChain.id });

    const calldata = encodeFunctionData({
      abi: GuardianExecutorAbi,
      functionName: "removeGuardian",
      args: [address],
    });

    const tx = await client.sendUserOperation({
      calls: [{
        to: contracts.guardianExecutor,
        data: calldata,
      }],
    });

    const receipt = await client.waitForUserOperationReceipt({ hash: tx });

    // Refresh guardians list
    getGuardians(client.account.address);
    return receipt;
  });

  /**
   * Accept/confirm a guardian proposal for a given account
   * This is called by the guardian (from their EOA) to accept the guardian role
   */
  const { inProgress: confirmGuardianInProgress, error: confirmGuardianError, execute: confirmGuardian } = useAsync(async <transport extends Transport, chain extends Chain, account extends Account>({ client, accountToGuard }: { client: Client<transport, chain, account>; accountToGuard: Address }) => {
    if (!contracts.guardianExecutor) throw new Error("GuardianExecutor contract address not configured");

    // Call acceptGuardian directly from the guardian's wallet
    const tx = await client.writeContract({
      address: contracts.guardianExecutor,
      abi: GuardianExecutorAbi,
      functionName: "acceptGuardian",
      args: [accountToGuard],
    });

    const transactionReceipt = await waitForTransactionReceipt(client, { hash: tx, confirmations: 1 });
    return { transactionReceipt };
  });

  /**
   * Discard/cancel any ongoing recovery for the caller's account
   * This is called by the smart account owner via ERC-4337 user operation
   */
  const { inProgress: discardRecoveryInProgress, error: discardRecoveryError, execute: discardRecovery } = useAsync(async () => {
    if (!contracts.guardianExecutor) throw new Error("GuardianExecutor contract address not configured");

    const client = getClient({ chainId: defaultChain.id });

    const calldata = encodeFunctionData({
      abi: GuardianExecutorAbi,
      functionName: "discardRecovery",
      args: [],
    });

    const tx = await client.sendUserOperation({
      calls: [{
        to: contracts.guardianExecutor,
        data: calldata,
      }],
    });

    const receipt = await client.waitForUserOperationReceipt({ hash: tx });
    return receipt;
  });

  /**
   * Initialize recovery for an account
   * This is called by a guardian to start the recovery process
   */
  const { inProgress: initRecoveryInProgress, error: initRecoveryError, execute: initRecovery } = useAsync(async <transport extends Transport, chain extends Chain, account extends Account>({
    accountToRecover,
    credentialPublicKey,
    credentialId,
    client,
    recoveryType = RecoveryType.WebAuthn,
  }: {
    accountToRecover: Address;
    credentialPublicKey: Uint8Array<ArrayBufferLike>;
    credentialId: string;
    client: Client<transport, chain, account>;
    recoveryType?: RecoveryType;
  }) => {
    if (!contracts.guardianExecutor) throw new Error("GuardianExecutor contract address not configured");

    // For WebAuthn recovery, encode the credential data
    let recoveryData: Hex;

    if (recoveryType === RecoveryType.WebAuthn) {
      const publicKeyBytes = getPublicKeyBytesFromPasskeySignature(credentialPublicKey);
      const publicKeyHex = [
        pad(`0x${publicKeyBytes[0].toString("hex")}`),
        pad(`0x${publicKeyBytes[1].toString("hex")}`),
      ] as const;

      // Encode the recovery data for WebAuthn
      recoveryData = encodeAbiParameters(
        parseAbiParameters("bytes32 credentialIdHash, bytes32[2] publicKey"),
        [
          keccak256(toHex(base64UrlToUint8Array(credentialId))),
          publicKeyHex,
        ],
      );
    } else {
      throw new Error(`Unsupported recovery type: ${recoveryType}`);
    }

    // Call initializeRecovery from the guardian's wallet
    const tx = await client.writeContract({
      address: contracts.guardianExecutor,
      abi: GuardianExecutorAbi,
      functionName: "initializeRecovery",
      args: [accountToRecover, recoveryType, recoveryData],
    });

    await waitForTransactionReceipt(client, { hash: tx });
    return tx;
  });

  /**
   * Check for pending recovery requests for an account
   * Returns status and timing information
   */
  const { inProgress: checkRecoveryRequestInProgress, error: checkRecoveryRequestError, execute: checkRecoveryRequest } = useAsync(async ({ address }: { address: Address }) => {
    if (!contracts.guardianExecutor) throw new Error("GuardianExecutor contract address not configured");

    const client = getPublicClient({ chainId: defaultChain.id });

    // Get timing constants from contract
    const [requestValidityTime, requestDelayTime] = await Promise.all([
      client.readContract({
        address: contracts.guardianExecutor,
        abi: GuardianExecutorAbi,
        functionName: "REQUEST_VALIDITY_TIME",
        args: [],
      }),
      client.readContract({
        address: contracts.guardianExecutor,
        abi: GuardianExecutorAbi,
        functionName: "REQUEST_DELAY_TIME",
        args: [],
      }),
    ]);

    // Get pending recovery
    const [recoveryType, hashedData, timestamp] = await client.readContract({
      address: contracts.guardianExecutor,
      abi: GuardianExecutorAbi,
      functionName: "pendingRecovery",
      args: [address],
    });

    // No recovery in progress
    if (recoveryType === RecoveryType.None || timestamp === 0n) {
      return { pendingRecovery: false } as const;
    }

    const currentTime = BigInt(Math.floor(Date.now() / 1000));
    const recoveryReadyTime = BigInt(timestamp) + requestDelayTime;
    const recoveryExpiryTime = BigInt(timestamp) + requestValidityTime;

    // Check if recovery has expired
    if (currentTime > recoveryExpiryTime) {
      return { pendingRecovery: false } as const;
    }

    const isReady = currentTime >= recoveryReadyTime;
    const remainingTime = isReady ? 0n : recoveryReadyTime - currentTime;

    return {
      pendingRecovery: true,
      ready: isReady,
      remainingTime,
      accountAddress: address,
      recoveryType,
      hashedData,
      timestamp,
    } as const;
  });

  /**
   * Finalize a pending recovery after the delay has elapsed
   * This can be called by anyone once the delay has passed
   */
  const { inProgress: executeRecoveryInProgress, error: executeRecoveryError, execute: executeRecovery } = useAsync(async ({
    accountAddress,
    credentialId,
    rawPublicKey,
  }: {
    accountAddress: Address;
    credentialId: string;
    rawPublicKey: readonly [Hex, Hex];
  }) => {
    if (!contracts.guardianExecutor) throw new Error("GuardianExecutor contract address not configured");

    // Use throwaway client for finalization (anyone can call this)
    const client = getThrowAwayClient({ chainId: defaultChain.id });

    const recoveryData = encodeAbiParameters(
      parseAbiParameters("bytes32 credentialIdHash, bytes32[2] publicKey"),
      [
        keccak256(toHex(base64UrlToUint8Array(credentialId))),
        rawPublicKey,
      ],
    );

    // Call finalizeRecovery
    const tx = await client.writeContract({
      address: contracts.guardianExecutor,
      abi: GuardianExecutorAbi,
      functionName: "finalizeRecovery",
      args: [accountAddress, recoveryData],
    });

    const receipt = await waitForTransactionReceipt(client, { hash: tx });
    return receipt;
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
