import type { Abi, Address, Chain, Client, Hash, Hex, Transport } from "viem";
import { encodeFunctionData } from "viem";
import { type SmartAccount } from "viem/account-abstraction";

import { type SessionSpec } from "../session/types.js";

/**
 * Note: LimitType and ConstraintCondition are now numeric enums,
 * so they can be used directly in ABI encoding without conversion.
 */

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
const SESSION_KEY_VALIDATOR_ABI = [
  {
    type: "function",
    name: "createSession",
    inputs: [
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
                  { name: "index", type: "uint64" },
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
    outputs: [],
    stateMutability: "nonpayable",
  },
] as const satisfies Abi;

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

  // Encode the createSession call
  const callData = encodeFunctionData({
    abi: SESSION_KEY_VALIDATOR_ABI,
    functionName: "createSession",
    args: [sessionSpecForAbi],
  });

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
