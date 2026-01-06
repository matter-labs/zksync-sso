import type { Account, Address, Chain, Hex, Transport, WalletClient } from "viem";
import { encodeAbiParameters, keccak256, parseAbiParameters, toHex } from "viem";
import { waitForTransactionReceipt } from "viem/actions";
import { base64urlToUint8Array, getPublicKeyBytesFromPasskeySignature } from "zksync-sso-4337/utils";

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

    const client = getClient({ chainId: defaultChain.id, usePaymaster: true });
    const accountAddress = client.account.address;

    // Check if GuardianExecutor module is installed
    const publicClient = getPublicClient({ chainId: defaultChain.id });
    const isModuleInstalled = await publicClient.readContract({
      address: accountAddress,
      abi: [{
        type: "function",
        name: "isModuleInstalled",
        inputs: [
          { name: "moduleTypeId", type: "uint256" },
          { name: "module", type: "address" },
          { name: "additionalContext", type: "bytes" },
        ],
        outputs: [{ type: "bool" }],
        stateMutability: "view",
      }],
      functionName: "isModuleInstalled",
      args: [2n, contracts.guardianExecutor, "0x"], // 2 = MODULE_TYPE_EXECUTOR
    });

    // Module should be installed during account deployment
    if (!isModuleInstalled) {
      throw new Error(
        `GuardianExecutor module is not installed for account ${accountAddress}. `
        + "The module should be installed during account deployment. "
        + `GuardianExecutor address: ${contracts.guardianExecutor}`,
      );
    }

    // eslint-disable-next-line no-console
    console.log("✅ GuardianExecutor module is installed");

    // eslint-disable-next-line no-console
    console.log("🔍 Proposing guardian:", {
      guardianExecutor: contracts.guardianExecutor,
      guardianAddress: address,
      accountAddress,
    });

    // Call GuardianExecutor.proposeGuardian() directly
    // The SDK will automatically wrap this in account.execute() via encode_execute_call_data
    const tx = await client.writeContract({
      address: contracts.guardianExecutor,
      abi: GuardianExecutorAbi,
      functionName: "proposeGuardian",
      args: [address],
    });

    // eslint-disable-next-line no-console
    console.log("✅ Guardian proposal transaction sent:", tx);

    // Get the full transaction receipt for event parsing
    const receipt = await publicClient.waitForTransactionReceipt({ hash: tx });

    // eslint-disable-next-line no-console
    console.log("Guardian proposal transaction receipt:", {
      hash: tx,
      status: receipt.status,
      gasUsed: receipt.gasUsed,
      blockNumber: receipt.blockNumber,
      logsCount: receipt.logs.length,
    });

    // eslint-disable-next-line no-console
    console.log("🔍 All emitted events:");
    receipt.logs.forEach((log, i) => {
      // eslint-disable-next-line no-console
      console.log(`  Event ${i + 1}:`, {
        address: log.address,
        topic0: log.topics[0],
        topics: log.topics,
        data: log.data,
      });
    });

    if (receipt.status != "success") {
      throw new Error(`Failed to propose guardian ${address} for account ${accountAddress}`);
    }

    // Verify the guardian was actually proposed by checking for the GuardianProposed event
    const guardianProposedEventTopic = "0xf1284770232ce131bb37044a576145e0b24fd6b24f5710624b11255d5ac61a81"; // keccak256("GuardianProposed(address,address)")
    const proposalEvent = receipt.logs.find((log) =>
      log.topics[0] === guardianProposedEventTopic
      && log.topics[1]?.toLowerCase() === `0x000000000000000000000000${accountAddress.slice(2).toLowerCase()}`
      && log.topics[2]?.toLowerCase() === `0x000000000000000000000000${address.slice(2).toLowerCase()}`,
    );

    if (!proposalEvent) {
      // eslint-disable-next-line no-console
      console.error("GuardianProposed event not found in transaction logs.");
      // eslint-disable-next-line no-console
      console.error("Expected event topic:", guardianProposedEventTopic);
      // eslint-disable-next-line no-console
      console.error("Expected account:", accountAddress);
      // eslint-disable-next-line no-console
      console.error("Expected guardian:", address);
      // eslint-disable-next-line no-console
      console.error("All event topics:", receipt.logs.map((l) => l.topics[0]));
      throw new Error("Guardian proposal transaction succeeded but GuardianProposed event was not emitted.");
    }

    // eslint-disable-next-line no-console
    console.log("✅ GuardianProposed event found, proposal successful");

    return receipt;
  });

  /**
   * Remove an existing guardian from the caller's account
   * This is called by the smart account owner via ERC-4337 user operation
   */
  const { inProgress: removeGuardianInProgress, error: removeGuardianError, execute: removeGuardian } = useAsync(async (address: Address) => {
    if (!contracts.guardianExecutor) throw new Error("GuardianExecutor contract address not configured");

    const client = getClient({ chainId: defaultChain.id, usePaymaster: true });

    const tx = await client.writeContract({
      address: contracts.guardianExecutor,
      abi: GuardianExecutorAbi,
      functionName: "removeGuardian",
      args: [address],
    });

    const receipt = await client.waitForTransactionReceipt({ hash: tx });

    // Refresh guardians list
    getGuardians(client.account.address);
    return receipt;
  });

  /**
   * Accept/confirm a guardian proposal for a given account
   * This is called by the guardian (from their EOA or smart account) to accept the guardian role
   */
  const { inProgress: confirmGuardianInProgress, error: confirmGuardianError, execute: confirmGuardian } = useAsync(async <transport extends Transport, chain extends Chain, account extends Account>({ client, accountToGuard }: { client: WalletClient<transport, chain, account>; accountToGuard: Address }) => {
    if (!contracts.guardianExecutor) throw new Error("GuardianExecutor contract address not configured");

    const guardianAddress = client.account.address;

    // First, verify the guardian was actually proposed
    const guardians = await getGuardians(accountToGuard);
    const guardianStatus = guardians.find((g) => g.addr.toLowerCase() === guardianAddress.toLowerCase());

    if (!guardianStatus) {
      throw new Error(`Guardian ${guardianAddress} was never proposed for account ${accountToGuard}. The account owner must propose this guardian first before you can accept.`);
    }

    if (guardianStatus.isReady) {
      return { alreadyActive: true };
    }

    // Call acceptGuardian from the guardian's wallet
    const tx = await client.writeContract({
      address: contracts.guardianExecutor,
      abi: GuardianExecutorAbi,
      functionName: "acceptGuardian",
      args: [accountToGuard],
      chain: null,
    });

    const transactionReceipt = await waitForTransactionReceipt(client, { hash: tx, confirmations: 1 });

    // Check if transaction was successful
    if (transactionReceipt.status === "reverted") {
      throw new Error(`Transaction reverted: Guardian confirmation failed. Guardian: ${guardianAddress}, Account: ${accountToGuard}. This usually means the guardian was not properly proposed or the GuardianExecutor module is not installed.`);
    }

    return { transactionReceipt };
  });

  /**
   * Discard/cancel any ongoing recovery for the caller's account
   * This is called by the smart account owner via ERC-4337 user operation
   */
  const { inProgress: discardRecoveryInProgress, error: discardRecoveryError, execute: discardRecovery } = useAsync(async () => {
    if (!contracts.guardianExecutor) throw new Error("GuardianExecutor contract address not configured");

    const client = getClient({ chainId: defaultChain.id });

    const tx = await client.writeContract({
      address: contracts.guardianExecutor,
      abi: GuardianExecutorAbi,
      functionName: "discardRecovery",
      args: [],
    });

    const receipt = await client.waitForTransactionReceipt({ hash: tx });
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
    client: WalletClient<transport, chain, account>;
    recoveryType?: RecoveryType;
  }) => {
    if (!contracts.guardianExecutor) throw new Error("GuardianExecutor contract address not configured");

    // For WebAuthn recovery, encode the credential data
    let recoveryData: Hex;

    if (recoveryType === RecoveryType.WebAuthn) {
      // Validate inputs before encoding
      if (!credentialId || credentialId.trim().length === 0) {
        throw new Error("credentialId cannot be empty");
      }

      const publicKeyBytes = getPublicKeyBytesFromPasskeySignature(credentialPublicKey);

      // Validate that public key coordinates are valid 32-byte values
      if (publicKeyBytes[0].length !== 32 || publicKeyBytes[1].length !== 32) {
        throw new Error("Invalid public key coordinates: must be 32 bytes each");
      }

      const publicKeyHex = [
        pad(`0x${publicKeyBytes[0].toString("hex")}`),
        pad(`0x${publicKeyBytes[1].toString("hex")}`),
      ] as const;

      // Encode the recovery data for WebAuthn
      recoveryData = encodeAbiParameters(
        parseAbiParameters("bytes32 credentialIdHash, bytes32[2] publicKey"),
        [
          keccak256(toHex(base64urlToUint8Array(credentialId))),
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
      chain: null,
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
        keccak256(toHex(base64urlToUint8Array(credentialId))),
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
