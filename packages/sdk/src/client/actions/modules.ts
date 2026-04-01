/**
 * Module management actions for ERC-7579 smart accounts
 */

import type { Address, Hex, PublicClient } from "viem";

/**
 * ERC-7579 module type IDs
 */
export const ModuleType = {
  VALIDATOR: 1n,
  EXECUTOR: 2n,
  FALLBACK: 3n,
  HOOK: 4n,
} as const;

/**
 * Minimal ABI for isModuleInstalled function on ERC-7579 accounts
 */
const IERC7579AccountAbi = [
  {
    type: "function",
    name: "isModuleInstalled",
    inputs: [
      { name: "moduleTypeId", type: "uint256", internalType: "uint256" },
      { name: "module", type: "address", internalType: "address" },
      { name: "additionalContext", type: "bytes", internalType: "bytes" },
    ],
    outputs: [{ name: "", type: "bool", internalType: "bool" }],
    stateMutability: "view",
  },
] as const;

export type IsModuleInstalledParams = {
  /** Public client for reading contract state */
  client: PublicClient;
  /** The smart account address to check */
  accountAddress: Address;
  /** The module address to check for installation */
  moduleAddress: Address;
  /** The module type ID (use ModuleType constants) */
  moduleTypeId: bigint;
  /** Optional additional context for module lookup (usually empty) */
  additionalContext?: Hex;
};

export type IsModuleInstalledResult = {
  /** Whether the module is installed on the account */
  isInstalled: boolean;
};

/**
 * Check if a module is installed on an ERC-7579 smart account
 *
 * @example
 * ```typescript
 * const result = await isModuleInstalled({
 *   client: publicClient,
 *   accountAddress: "0x...",
 *   moduleAddress: "0x...",
 *   moduleTypeId: ModuleType.EXECUTOR,
 * });
 * console.log("Guardian module installed:", result.isInstalled);
 * ```
 */
export async function isModuleInstalled(
  params: IsModuleInstalledParams,
): Promise<IsModuleInstalledResult> {
  const {
    client,
    accountAddress,
    moduleAddress,
    moduleTypeId,
    additionalContext = "0x",
  } = params;

  const isInstalled = await client.readContract({
    address: accountAddress,
    abi: IERC7579AccountAbi,
    functionName: "isModuleInstalled",
    args: [moduleTypeId, moduleAddress, additionalContext],
  });

  return { isInstalled };
}

/**
 * Check if the guardian executor module is installed on an account
 *
 * @example
 * ```typescript
 * const result = await isGuardianModuleInstalled({
 *   client: publicClient,
 *   accountAddress: "0x...",
 *   guardianExecutorAddress: "0x...",
 * });
 * if (!result.isInstalled) {
 *   console.warn("Guardian module not installed on account");
 * }
 * ```
 */
export async function isGuardianModuleInstalled(params: {
  client: PublicClient;
  accountAddress: Address;
  guardianExecutorAddress: Address;
}): Promise<IsModuleInstalledResult> {
  return isModuleInstalled({
    ...params,
    moduleAddress: params.guardianExecutorAddress,
    moduleTypeId: ModuleType.EXECUTOR,
  });
}
