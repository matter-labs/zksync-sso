import { type Account, type Address, type Chain, type Client, encodeFunctionData, type Hash, type Hex, type Prettify, type TransactionReceipt, type Transport } from "viem";
import { readContract, waitForTransactionReceipt } from "viem/actions";
import { getGeneralPaymasterInput, sendTransaction } from "viem/zksync";

import { SessionKeyValidatorAbi } from "../../../abi/SessionKeyValidator.js";
import { type CustomPaymasterHandler, getTransactionWithPaymasterData } from "../../../paymaster/index.js";
import { encodeSession } from "../../../utils/encoding.js";
import { noThrow } from "../../../utils/helpers.js";
import type { SessionConfig, SessionState, SessionStateEventCallback } from "../../../utils/session.js";
import { SessionEventType } from "../../../utils/session.js";

export type CreateSessionArgs = {
  sessionConfig: SessionConfig;
  contracts: {
    session: Address; // session module
  };
  paymaster?: {
    address: Address;
    paymasterInput?: Hex;
  };
  onTransactionSent?: (hash: Hash) => void;
  paymasterHandler?: CustomPaymasterHandler;
};
export type CreateSessionReturnType = {
  transactionReceipt: TransactionReceipt;
};
export const createSession = async <
  transport extends Transport,
  chain extends Chain,
  account extends Account,
>(client: Client<transport, chain, account>, args: Prettify<CreateSessionArgs>): Promise<Prettify<CreateSessionReturnType>> => {
  const callData = encodeFunctionData({
    abi: SessionKeyValidatorAbi,
    functionName: "createSession",
    args: [args.sessionConfig],
  });

  const sendTransactionArgs = {
    account: client.account,
    to: args.contracts.session,
    paymaster: args.paymaster?.address,
    paymasterInput: args.paymaster?.address ? (args.paymaster?.paymasterInput || getGeneralPaymasterInput({ innerInput: "0x" })) : undefined,
    data: callData,
    gas: 10_000_000n, // TODO: Remove when gas estimation is fixed
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
  } as any;

  const transactionWithPaymasterData: any = await getTransactionWithPaymasterData(
    client.chain.id,
    client.account.address,
    sendTransactionArgs,
    args.paymasterHandler,
  );

  const transactionHash = await sendTransaction(client, transactionWithPaymasterData);
  if (args.onTransactionSent) {
    noThrow(() => args.onTransactionSent?.(transactionHash));
  }

  const transactionReceipt = await waitForTransactionReceipt(client, { hash: transactionHash });
  if (transactionReceipt.status !== "success") throw new Error("createSession transaction reverted");

  return {
    transactionReceipt,
  };
};

export type RevokeSessionArgs = {
  sessionId: Hash;
  contracts: {
    session: Address; // session module
  };
  paymaster?: {
    address: Address;
    paymasterInput?: Hex;
  };
  onTransactionSent?: (hash: Hash) => void;
  paymasterHandler?: CustomPaymasterHandler;
};
export type RevokeSessionReturnType = {
  transactionReceipt: TransactionReceipt;
};
export const revokeSession = async <
  transport extends Transport,
  chain extends Chain,
  account extends Account,
>(client: Client<transport, chain, account>, args: Prettify<RevokeSessionArgs>): Promise<Prettify<RevokeSessionReturnType>> => {
  const callData = encodeFunctionData({
    abi: SessionKeyValidatorAbi,
    functionName: "revokeKey",
    args: [args.sessionId],
  });

  const sendTransactionArgs = {
    account: client.account,
    to: args.contracts.session,
    paymaster: args.paymaster?.address,
    paymasterInput: args.paymaster?.address ? (args.paymaster?.paymasterInput || getGeneralPaymasterInput({ innerInput: "0x" })) : undefined,
    data: callData,
    gas: 10_000_000n, // TODO: Remove when gas estimation is fixed
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
  } as any;

  const transactionWithPaymasterData: any = await getTransactionWithPaymasterData(
    client.chain.id,
    client.account.address,
    sendTransactionArgs,
    args.paymasterHandler,
  );

  const transactionHash = await sendTransaction(client, transactionWithPaymasterData);

  if (args.onTransactionSent) {
    noThrow(() => args.onTransactionSent?.(transactionHash));
  }

  const transactionReceipt = await waitForTransactionReceipt(client, { hash: transactionHash });
  if (transactionReceipt.status !== "success") throw new Error("createSession transaction reverted");

  return {
    transactionReceipt,
  };
};

export type GetSessionStateArgs = {
  account: Address;
  sessionConfig: SessionConfig;
  contracts: {
    session: Address; // session module
  };
};
export type GetSessionStateReturnType = {
  sessionState: SessionState;
};
export const getSessionState = async <
  transport extends Transport,
  chain extends Chain,
>(client: Client<transport, chain>, args: Prettify<GetSessionStateArgs>): Promise<Prettify<GetSessionStateReturnType>> => {
  const sessionState = await readContract(client, {
    address: args.contracts.session,
    abi: SessionKeyValidatorAbi,
    functionName: "sessionState",
    args: [args.account, args.sessionConfig],
  });

  return {
    sessionState: sessionState as SessionState,
  };
};

export type CheckSessionStateArgs = {
  account: Address;
  sessionConfig: SessionConfig;
  contracts: {
    session: Address; // session module
  };
  onSessionStateChange: SessionStateEventCallback;
};

// Store timeouts by sessionId to allow cleanup
const sessionTimeouts = new Map<string, NodeJS.Timeout[]>();

/**
 * Checks the current session state and sets up expiry notification.
 * This function will trigger the callback with the session state.
 */
export const checkSessionState = async <
  transport extends Transport,
  chain extends Chain,
>(client: Client<transport, chain>, args: Prettify<CheckSessionStateArgs>): Promise<void> => {
  // Generate a session ID for tracking timeouts
  const sessionId = `${args.account}_${encodeSession(args.sessionConfig)}`;

  // Clear any existing timeouts for this session
  if (sessionTimeouts.has(sessionId)) {
    sessionTimeouts.get(sessionId)?.forEach((timeout) => clearTimeout(timeout));
    sessionTimeouts.delete(sessionId);
  }

  try {
    // Get current session state
    const { sessionState } = await getSessionState(client, {
      account: args.account,
      sessionConfig: args.sessionConfig,
      contracts: args.contracts,
    });

    const now = BigInt(Math.floor(Date.now() / 1000));

    // Check session status
    if (sessionState.status === 0) { // Not initialized
      args.onSessionStateChange({
        type: SessionEventType.Inactive,
        message: "Session is not initialized",
      });
    } else if (sessionState.status === 2) { // Closed/Revoked
      args.onSessionStateChange({
        type: SessionEventType.Revoked,
        message: "Session has been revoked",
      });
    } else if (args.sessionConfig.expiresAt <= now) {
      // Session has expired
      args.onSessionStateChange({
        type: SessionEventType.Expired,
        expiresAt: args.sessionConfig.expiresAt,
        message: "Session has expired",
      });
    } else {
      // Session is active, set up expiry notification
      const timeToExpiry = Number(args.sessionConfig.expiresAt - now) * 1000; // Convert to milliseconds

      // Set up a timeout for session expiry
      const expiryTimeout = setTimeout(() => {
        args.onSessionStateChange({
          type: SessionEventType.Expired,
          expiresAt: args.sessionConfig.expiresAt,
          message: "Session has expired",
        });
      }, timeToExpiry);

      // Track the timeout for potential cleanup
      sessionTimeouts.set(sessionId, [expiryTimeout]);
    }
  } catch (error) {
    // In case of error fetching session state
    args.onSessionStateChange({
      type: SessionEventType.Inactive,
      message: `Error checking session state: ${(error as Error).message}`,
    });
  }
};
