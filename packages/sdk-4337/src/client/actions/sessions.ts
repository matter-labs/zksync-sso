/**
 * Actions for creating and querying sessions on smart accounts
 */

import type { Address, Chain, Client, Hash, Hex, PublicActions, Transport } from "viem";
import { decodeAbiParameters } from "viem";
import { type SmartAccount } from "viem/account-abstraction";
import { encode_create_session_call_data, encode_session_state_call_data } from "zksync-sso-web-sdk/bundler";

import { type SessionSpec } from "../session/types.js";
import { sessionSpecToJSON } from "../session/utils.js";

// ============================================================================
// Create Session
// ============================================================================

/**
 * Parameters for creating a session on a smart account
 */
export type CreateSessionParams = {
  /**
   * The session specification defining permissions and limits
   */
  sessionSpec: SessionSpec;

  /**
   * Proof of session key ownership (signature of session hash by session key)
   */
  proof: Hex;

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
 * Return type for createSession action
 */
export type CreateSessionReturnType = {
  /**
   * The hash of the UserOperation sent to create the session
   */
  userOpHash: Hash;
};

/**
 * Create a new session on a smart account
 *
 * This action sends a UserOperation to the SessionKeyValidator module to create
 * a new session with the specified permissions and limits. The session can then
 * be used to sign transactions using the session key.
 *
 * @param client - Viem client with smart account
 * @param params - Session creation parameters
 * @returns Promise resolving to the UserOperation hash
 *
 * @example
 * ```typescript
 * import { createSession } from "@zksync-sso/sdk-4337";
 * import { createPublicClient, createWalletClient } from "viem";
 * import { zkSyncSepoliaTestnet } from "viem/chains";
 *
 * const walletClient = createWalletClient({
 *   chain: zkSyncSepoliaTestnet,
 *   account: mySmartAccount,
 * });
 *
 * const { userOpHash } = await createSession(walletClient, {
 *   sessionSpec: {
 *     signer: "0x...", // Session key address
 *     expiresAt: 1735689600n, // Unix timestamp
 *     feeLimit: { limitType: 0, limit: 0n, period: 0n }, // Unlimited
 *     callPolicies: [
 *       {
 *         target: "0x...", // Contract address
 *         selector: "0xa9059cbb", // transfer(address,uint256)
 *         maxValuePerUse: 0n,
 *         valueLimit: { limitType: 0, limit: 0n, period: 0n },
 *         constraints: [],
 *       },
 *     ],
 *     transferPolicies: [],
 *   },
 *   proof: "0x...", // Signature of session hash by session key
 *   contracts: {
 *     sessionValidator: "0x...", // SessionKeyValidator module address
 *   },
 * });
 * ```
 */
export async function createSession<
  TTransport extends Transport = Transport,
  TChain extends Chain | undefined = Chain | undefined,
  TAccount extends SmartAccount | undefined = SmartAccount | undefined,
>(
  client: Client<TTransport, TChain, TAccount>,
  params: CreateSessionParams,
): Promise<CreateSessionReturnType> {
  const { sessionSpec, proof, contracts } = params;

  if (!client.account) {
    throw new Error("Client must have an account");
  }

  // Convert SessionSpec into JSON string expected by wasm helper
  const sessionSpecJSON = sessionSpecToJSON(sessionSpec);
  const callData = encode_create_session_call_data(sessionSpecJSON, proof) as Hex;
  // Send the UserOperation to create the session using the bundler client method.
  // Note: We explicitly pass the account for broader viem compatibility.
  const bundler = client as unknown as {
    sendUserOperation: (args: {
      account: SmartAccount;
      calls: { to: Address; data: Hex }[];
    }) => Promise<Hash>;
    account: SmartAccount;
  };

  const userOpHash = await bundler.sendUserOperation({
    account: bundler.account,
    calls: [
      {
        to: contracts.sessionValidator,
        data: callData as Hex,
      },
    ],
  });

  return {
    userOpHash,
  };
}

// ============================================================================
// Get Session State
// ============================================================================

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

  // Encode using wasm helper and perform a single eth_call to get the raw result
  const sessionSpecJSON = sessionSpecToJSON(sessionSpec);
  const calldata = encode_session_state_call_data(account, sessionSpecJSON) as Hex;
  const { data } = await client.call({
    to: contracts.sessionValidator,
    data: calldata,
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
  } as any);

  // Decode the result using viem's ABI decoder (re-uses the ABI for decoding only)
  // Use viem's ABI decoder to decode the session state output
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const outputs = (SESSION_KEY_VALIDATOR_ABI[0] as any).outputs;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const decoded = decodeAbiParameters(outputs, data as Hex) as any;

  // The top-level return is a single tuple; decoded[0] will be the struct
  const top = decoded[0];

  return {
    sessionState: top as SessionState,
  };
}

// ============================================================================
// Session State Events
// ============================================================================

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

// ============================================================================
// List Active Sessions
// ============================================================================

/**
 * Parameters for listing active sessions
 */
export type ListActiveSessionsParams = {
  /**
   * The smart account address to query sessions for
   */
  account: Address;

  /**
   * RPC URL for the blockchain network
   */
  rpcUrl: string;

  /**
   * Contract addresses
   */
  contracts: {
    /**
     * The SessionKeyValidator module address
     */
    sessionValidator: Address;
    /**
     * The EntryPoint contract address
     */
    entryPoint: Address;
    /**
     * The AccountFactory contract address
     */
    accountFactory: Address;
    /**
     * The WebAuthnValidator module address
     */
    webauthnValidator: Address;
    /**
     * The EOAValidator module address
     */
    eoaValidator: Address;
    /**
     * The GuardianExecutor module address
     */
    guardianExecutor: Address;
  };
};

/**
 * Return type for listActiveSessions action
 */
export type ListActiveSessionsReturnType = {
  /**
   * Array of active sessions
   */
  sessions: Array<{
    /**
     * The hash of the session
     */
    sessionHash: Hex;
    /**
     * The session specification
     */
    sessionSpec: SessionSpec;
  }>;
};

/**
 * List all active sessions for a smart account
 *
 * This action queries the SessionKeyValidator contract events to find all sessions
 * that have been created but not yet revoked for a given account.
 *
 * @param params - Parameters including account address, RPC URL, and contract addresses
 * @returns Promise resolving to an array of active sessions
 *
 * @example
 * ```typescript
 * import { listActiveSessions } from "@zksync-sso/sdk-4337";
 *
 * const { sessions } = await listActiveSessions({
 *   account: "0x...", // Smart account address
 *   rpcUrl: "https://sepolia.era.zksync.dev",
 *   contracts: {
 *     sessionValidator: "0x...",
 *     entryPoint: "0x...",
 *     accountFactory: "0x...",
 *   },
 * });
 *
 * console.log(`Found ${sessions.length} active sessions`);
 * sessions.forEach(({ sessionHash, sessionSpec }) => {
 *   console.log("Session hash:", sessionHash);
 *   console.log("Session signer:", sessionSpec.signer);
 *   console.log("Expires at:", sessionSpec.expiresAt);
 * });
 * ```
 */
export async function listActiveSessions(
  params: ListActiveSessionsParams,
): Promise<ListActiveSessionsReturnType> {
  const { account, rpcUrl, contracts } = params;

  // Import the WASM function
  const { get_active_sessions_wasm } = await import("zksync-sso-web-sdk/bundler");

  // Prepare contracts JSON
  const contractsJson = JSON.stringify({
    session_validator: contracts.sessionValidator,
    entry_point: contracts.entryPoint,
    account_factory: contracts.accountFactory,
    webauthn_validator: contracts.webauthnValidator,
    eoa_validator: contracts.eoaValidator,
    guardian_executor: contracts.guardianExecutor,
  });

  // Call the WASM function
  const resultJson = await get_active_sessions_wasm(
    rpcUrl,
    account,
    contractsJson,
  );

  // Parse the JSON result
  const sessions = JSON.parse(resultJson as string);

  return {
    sessions,
  };
}
