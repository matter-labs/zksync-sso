import { type Account, type Address, type Chain, type Client, type Hash, type Prettify, type TransactionReceipt, type Transport } from "viem";
import { readContract, sendTransaction, waitForTransactionReceipt } from "viem/actions";

import { SessionKeyValidatorAbi } from "../../../abi/SessionKeyValidator.js";
import { noThrow } from "../../../utils/helpers.js";
import type { SessionConfig, SessionState, SessionStateEventCallback } from "../../../utils/session.js";
import { SessionEventType, SessionStatus } from "../../../utils/session.js";
import { encodeCreateSessionCallData, encodeRevokeSessionCallData, type SessionSpec } from "../../../utils/wasm.js";

/**
 * Convert SessionConfig (TypeScript with bigint) to SessionSpec (WASM with strings)
 */
function sessionConfigToSpec(config: SessionConfig): SessionSpec {
  return {
    signer: config.signer,
    expiresAt: config.expiresAt.toString(),
    feeLimit: {
      limitType: config.feeLimit.limitType as any,
      limit: config.feeLimit.limit.toString(),
      period: config.feeLimit.period.toString(),
    },
    callPolicies: config.callPolicies.map((policy) => ({
      target: policy.target,
      selector: policy.selector,
      maxValuePerUse: policy.maxValuePerUse.toString(),
      valueLimit: {
        limitType: policy.valueLimit.limitType as any,
        limit: policy.valueLimit.limit.toString(),
        period: policy.valueLimit.period.toString(),
      },
      constraints: policy.constraints.map((constraint) => ({
        condition: constraint.condition as any,
        index: Number(constraint.index),
        refValue: constraint.refValue,
        limit: {
          limitType: constraint.limit.limitType as any,
          limit: constraint.limit.limit.toString(),
          period: constraint.limit.period.toString(),
        },
      })),
    })),
    transferPolicies: config.transferPolicies.map((policy) => ({
      target: policy.target,
      maxValuePerUse: policy.maxValuePerUse.toString(),
      valueLimit: {
        limitType: policy.valueLimit.limitType as any,
        limit: policy.valueLimit.limit.toString(),
        period: policy.valueLimit.period.toString(),
      },
    })),
  };
}

export type CreateSessionArgs = {
  sessionConfig: SessionConfig;
  contracts: {
    session: Address; // session module
  };
  onTransactionSent?: (hash: Hash) => void;
};
export type CreateSessionReturnType = {
  transactionReceipt: TransactionReceipt;
};
export const createSession = async <
  transport extends Transport,
  chain extends Chain,
  account extends Account,
>(client: Client<transport, chain, account>, args: Prettify<CreateSessionArgs>): Promise<Prettify<CreateSessionReturnType>> => {
  // Convert SessionConfig to SessionSpec and encode using WASM
  const sessionSpec = sessionConfigToSpec(args.sessionConfig);
  const callData = await encodeCreateSessionCallData(sessionSpec, args.contracts.session);

  const transactionHash = await sendTransaction(client, {
    to: args.contracts.session,
    data: callData,
  } as any);
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
  onTransactionSent?: (hash: Hash) => void;
};
export type RevokeSessionReturnType = {
  transactionReceipt: TransactionReceipt;
};
export const revokeSession = async <
  transport extends Transport,
  chain extends Chain,
  account extends Account,
>(client: Client<transport, chain, account>, args: Prettify<RevokeSessionArgs>): Promise<Prettify<RevokeSessionReturnType>> => {
  // Encode using WASM
  const callData = await encodeRevokeSessionCallData(args.sessionId, args.contracts.session);

  const transactionHash = await sendTransaction(client, {
    to: args.contracts.session,
    data: callData,
  } as any);

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
  sessionConfig: SessionConfig;
  sessionState: SessionState;
  onSessionStateChange: SessionStateEventCallback;
  sessionNotifyTimeout?: NodeJS.Timeout;
};
export type CheckSessionStateReturnType = {
  newTimeout?: NodeJS.Timeout;
};

/**
 * Checks the current session state and sets up expiry notification.
 * This function will trigger the callback with the session state.
 */
export const sessionStateNotify = (args: Prettify<CheckSessionStateArgs>): CheckSessionStateReturnType => {
  // Generate a session ID for tracking timeouts
  const { sessionState } = args;
  const now = BigInt(Math.floor(Date.now() / 1000));

  // Check session status
  if (sessionState.status === SessionStatus.NotInitialized) { // Not initialized
    args.onSessionStateChange({
      type: SessionEventType.Inactive,
      message: "Session is not initialized",
    });
  } else if (sessionState.status === SessionStatus.Closed) { // Closed/Revoked
    args.onSessionStateChange({
      type: SessionEventType.Revoked,
      message: "Session has been revoked",
    });
  } else if (args.sessionConfig.expiresAt <= now) {
    // Session has expired
    args.onSessionStateChange({
      type: SessionEventType.Expired,
      message: "Session has expired",
    });
  } else {
    // Session is active, set up expiry notification
    const timeToExpiry = Number(args.sessionConfig.expiresAt - now) * 1000;
    if (args.sessionNotifyTimeout) {
      clearTimeout(args.sessionNotifyTimeout);
    }
    const newTimeout = setTimeout(() => {
      args.onSessionStateChange({
        type: SessionEventType.Expired,
        message: "Session has expired",
      });
    }, timeToExpiry);
    return {
      newTimeout,
    };
  }

  return {};
};
