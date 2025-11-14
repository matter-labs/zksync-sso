/**
 * Actions for querying and monitoring session state from the SessionKeyValidator contract
 */

import type { Address, Chain, Client, PublicActions, Transport } from "viem";
import { readContract } from "viem/actions";

import type { SessionSpec } from "../session/types.js";

/**
 * Session status as stored on-chain
 */
export enum SessionStatus {
  NotInitialized = 0,
  Active = 1,
  Closed = 2,
}

/**
 * On-chain session state returned by SessionKeyValidator
 */
export type SessionState = {
  /**
   * Current status of the session
   */
  status: SessionStatus;

  /**
   * Remaining fee budget for this session
   */
  feesRemaining: bigint;

  /**
   * Value limits for transfer policies
   */
  transferValue: {
    remaining: bigint;
    target: Address;
    selector: `0x${string}`;
    index: bigint;
  }[];

  /**
   * Value limits for call policies
   */
  callValue: {
    remaining: bigint;
    target: Address;
    selector: `0x${string}`;
    index: bigint;
  }[];

  /**
   * Parameter constraint limits for call policies
   */
  callParams: {
    remaining: bigint;
    target: Address;
    selector: `0x${string}`;
    index: bigint;
  }[];
};

/**
 * Parameters for querying session state
 */
export type GetSessionStateParams = {
  /**
   * The smart account address
   */
  account: Address;

  /**
   * The session specification to query state for
   */
  sessionSpec: SessionSpec;

  /**
   * Contract addresses
   */
  contracts: {
    /**
     * The SessionKeyValidator module address
     */
    sessionValidator: Address;
  };
};

/**
 * Return type for getSessionState action
 */
export type GetSessionStateReturnType = {
  /**
   * The current on-chain state of the session
   */
  sessionState: SessionState;
};

/**
 * SessionKeyValidator ABI for sessionState view function
 */
const SESSION_KEY_VALIDATOR_ABI = [
  {
    type: "function",
    name: "sessionState",
    stateMutability: "view",
    inputs: [
      { name: "account", type: "address" },
      {
        name: "sessionSpec",
        type: "tuple",
        components: [
          { name: "signer", type: "address" },
          { name: "expiresAt", type: "uint48" },
          {
            name: "feeLimit",
            type: "tuple",
            components: [
              { name: "limitType", type: "uint8" },
              { name: "limit", type: "uint256" },
              { name: "period", type: "uint48" },
            ],
          },
          {
            name: "callPolicies",
            type: "tuple[]",
            components: [
              { name: "target", type: "address" },
              { name: "selector", type: "bytes4" },
              { name: "maxValuePerUse", type: "uint256" },
              {
                name: "valueLimit",
                type: "tuple",
                components: [
                  { name: "limitType", type: "uint8" },
                  { name: "limit", type: "uint256" },
                  { name: "period", type: "uint48" },
                ],
              },
              {
                name: "constraints",
                type: "tuple[]",
                components: [
                  { name: "condition", type: "uint8" },
                  { name: "index", type: "uint256" },
                  { name: "refValue", type: "bytes32" },
                  {
                    name: "limit",
                    type: "tuple",
                    components: [
                      { name: "limitType", type: "uint8" },
                      { name: "limit", type: "uint256" },
                      { name: "period", type: "uint48" },
                    ],
                  },
                ],
              },
            ],
          },
          {
            name: "transferPolicies",
            type: "tuple[]",
            components: [
              { name: "target", type: "address" },
              { name: "maxValuePerUse", type: "uint256" },
              {
                name: "valueLimit",
                type: "tuple",
                components: [
                  { name: "limitType", type: "uint8" },
                  { name: "limit", type: "uint256" },
                  { name: "period", type: "uint48" },
                ],
              },
            ],
          },
        ],
      },
    ],
    outputs: [
      {
        name: "",
        type: "tuple",
        components: [
          { name: "status", type: "uint8" },
          { name: "feesRemaining", type: "uint256" },
          {
            name: "transferValue",
            type: "tuple[]",
            components: [
              { name: "remaining", type: "uint256" },
              { name: "target", type: "address" },
              { name: "selector", type: "bytes4" },
              { name: "index", type: "uint256" },
            ],
          },
          {
            name: "callValue",
            type: "tuple[]",
            components: [
              { name: "remaining", type: "uint256" },
              { name: "target", type: "address" },
              { name: "selector", type: "bytes4" },
              { name: "index", type: "uint256" },
            ],
          },
          {
            name: "callParams",
            type: "tuple[]",
            components: [
              { name: "remaining", type: "uint256" },
              { name: "target", type: "address" },
              { name: "selector", type: "bytes4" },
              { name: "index", type: "uint256" },
            ],
          },
        ],
      },
    ],
  },
] as const;

/**
 * Query the current on-chain state of a session
 *
 * This action reads the SessionKeyValidator contract to get the current state
 * of a session, including remaining fee budget and usage limits.
 *
 * @param client - A viem public client with chain and transport
 * @param params - Parameters including account address, session spec, and contract addresses
 * @returns The current on-chain session state
 *
 * @example
 * ```typescript
 * const { sessionState } = await getSessionState(publicClient, {
 *   account: smartAccountAddress,
 *   sessionSpec: mySessionSpec,
 *   contracts: {
 *     sessionValidator: sessionValidatorAddress,
 *   },
 * });
 *
 * console.log("Session status:", sessionState.status);
 * console.log("Fees remaining:", sessionState.feesRemaining);
 *
 * // Check if session is active
 * if (sessionState.status === SessionStatus.Active) {
 *   console.log("Session is active!");
 * } else if (sessionState.status === SessionStatus.Closed) {
 *   console.log("Session has been closed/revoked");
 * }
 * ```
 */
export async function getSessionState<
  TTransport extends Transport = Transport,
  TChain extends Chain | undefined = Chain | undefined,
>(
  client: Client<TTransport, TChain> & PublicActions,
  params: GetSessionStateParams,
): Promise<GetSessionStateReturnType> {
  const { account, sessionSpec, contracts } = params;

  // Convert SessionSpec to match ABI types
  const sessionSpecForAbi = {
    signer: sessionSpec.signer,
    expiresAt: Number(sessionSpec.expiresAt),
    feeLimit: {
      limitType: sessionSpec.feeLimit.limitType, // Already numeric
      limit: sessionSpec.feeLimit.limit,
      period: Number(sessionSpec.feeLimit.period),
    },
    callPolicies: sessionSpec.callPolicies.map((policy) => ({
      target: policy.target,
      selector: policy.selector,
      maxValuePerUse: policy.maxValuePerUse,
      valueLimit: {
        limitType: policy.valueLimit.limitType, // Already numeric
        limit: policy.valueLimit.limit,
        period: Number(policy.valueLimit.period),
      },
      constraints: policy.constraints.map((constraint) => ({
        condition: constraint.condition, // Already numeric
        index: constraint.index,
        refValue: constraint.refValue,
        limit: {
          limitType: constraint.limit.limitType, // Already numeric
          limit: constraint.limit.limit,
          period: Number(constraint.limit.period),
        },
      })),
    })),
    transferPolicies: sessionSpec.transferPolicies.map((policy) => ({
      target: policy.target,
      maxValuePerUse: policy.maxValuePerUse,
      valueLimit: {
        limitType: policy.valueLimit.limitType, // Already numeric
        limit: policy.valueLimit.limit,
        period: Number(policy.valueLimit.period),
      },
    })),
  };

  const sessionState = await readContract(client, {
    address: contracts.sessionValidator,
    abi: SESSION_KEY_VALIDATOR_ABI,
    functionName: "sessionState",
    args: [account, sessionSpecForAbi],
  });

  return {
    sessionState: sessionState as SessionState,
  };
}

/**
 * Event types for session state changes
 */
export enum SessionEventType {
  Expired = "session_expired",
  Revoked = "session_revoked",
  Inactive = "session_inactive",
  Warning = "session_warning",
}

/**
 * Session state event
 */
export type SessionStateEvent = {
  /**
   * Type of event
   */
  type: SessionEventType;

  /**
   * Human-readable message describing the event
   */
  message: string;

  /**
   * Optional session state snapshot when event was triggered
   */
  sessionState?: SessionState;
};

/**
 * Callback function for session state change events
 */
export type SessionStateEventCallback = (event: SessionStateEvent) => void;
