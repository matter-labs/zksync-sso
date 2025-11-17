import type { Address, Chain, Client, Hash, Hex, Transport } from "viem";
import { type SmartAccount } from "viem/account-abstraction";
import { encode_create_session_call_data } from "zksync-sso-web-sdk/bundler";

import { type SessionSpec } from "../session/types.js";
import { sessionSpecToJSON } from "../session/utils.js";

/**
 * Parameters for creating a session on a smart account
 */
export type CreateSessionParams = {
  /**
   * The session specification defining permissions and limits
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
 * Return type for createSession action
 */
export type CreateSessionReturnType = {
  /**
   * The hash of the UserOperation sent to create the session
   */
  userOpHash: Hash;
};

/**
 * SessionKeyValidator ABI for createSession function
 */
// Encoding for createSession is delegated to the wasm helper in web-sdk.

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
  const { sessionSpec, contracts } = params;

  if (!client.account) {
    throw new Error("Client must have an account");
  }

  // Convert SessionSpec into JSON string expected by wasm helper
  const sessionSpecJSON = sessionSpecToJSON(sessionSpec);
  const callData = encode_create_session_call_data(sessionSpecJSON) as Hex;
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
