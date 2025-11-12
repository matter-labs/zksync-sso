import type { Address, Hex, Log } from "viem";
import { hexToBytes, keccak256, toHex } from "viem";
import {
  encode_deploy_account_call_data,
  generate_account_id,
  PasskeyPayload,
  // @ts-expect-error - TypeScript doesn't understand package.json exports
} from "zksync-sso-web-sdk/bundler";

/**
 * Parameters for preparing a smart account deployment
 */
export type PrepareDeploySmartAccountParams = {
  /** Contract addresses */
  contracts: {
    /** MSAFactory contract address */
    factory: Address;
    /** EOA validator address (required if eoaSigners provided) */
    eoaValidator?: Address;
    /** WebAuthn validator address (required if passkeySigners provided) */
    webauthnValidator?: Address;
  };

  /** Optional array of EOA signer addresses to install */
  eoaSigners?: Address[];

  /** Optional array of passkey signers to install */
  passkeySigners?: Array<{
    /** Hex-encoded credential ID */
    credentialId: Hex;
    /** Public key coordinates */
    publicKey: { x: Hex; y: Hex };
    /** Origin domain (e.g., "https://example.com" or window.location.origin) */
    originDomain: string;
  }>;

  /** Optional user ID for deterministic account deployment. If provided, generates deterministic accountId from userId */
  userId?: string;

  /** Optional custom account ID (32-byte hex). If not provided, generated from userId (if provided) or random */
  accountId?: Hex;
};

/**
 * Result from prepareDeploySmartAccount containing transaction data
 */
export type PrepareDeploySmartAccountResult = {
  /** Transaction to send to deploy the account */
  transaction: {
    /** Factory contract address */
    to: Address;
    /** Encoded deployAccount call data */
    data: Hex;
  };
  /** Account ID used as salt */
  accountId: Hex;
};

/**
 * Prepare a deployment transaction for a smart account.
 * This function does NOT send the transaction - it returns the transaction data
 * that can be sent via any means (EOA, bundler, etc).
 *
 * Uses Rust WASM SDK for encoding (no RPC calls).
 *
 * @param params - Deployment parameters including contracts and signers
 * @returns Transaction data
 *
 * @example
 * ```typescript
 * import { prepareDeploySmartAccount } from "zksync-sso/client-new/actions";
 *
 * const { transaction, accountId } = prepareDeploySmartAccount({
 *   contracts: {
 *     factory: "0x...",
 *     eoaValidator: "0x...",
 *   },
 *   eoaSigners: ["0x..."],
 * });
 *
 * // Send transaction via your preferred method
 * const hash = await walletClient.sendTransaction(transaction);
 *
 * // Wait for receipt and extract deployed address from AccountCreated event
 * const receipt = await publicClient.waitForTransactionReceipt({ hash });
 * const deployedAddress = getAccountAddressFromLogs(receipt.logs);
 * ```
 */
export function prepareDeploySmartAccount(
  params: PrepareDeploySmartAccountParams,
): PrepareDeploySmartAccountResult {
  const { contracts, eoaSigners, passkeySigners, userId, accountId: customAccountId } = params;

  // Validation: Check that required validators are provided
  if (eoaSigners && eoaSigners.length > 0 && !contracts.eoaValidator) {
    throw new Error(
      "eoaValidator contract address is required when eoaSigners are provided",
    );
  }

  if (passkeySigners && passkeySigners.length > 0 && !contracts.webauthnValidator) {
    throw new Error(
      "webauthnValidator contract address is required when passkeySigners are provided",
    );
  }

  // Validation: At least one signer type must be provided
  const hasEoaSigners = eoaSigners && eoaSigners.length > 0;
  const hasPasskeySigners = passkeySigners && passkeySigners.length > 0;

  if (!hasEoaSigners && !hasPasskeySigners) {
    throw new Error(
      "At least one signer must be provided (eoaSigners or passkeySigners)",
    );
  }

  // Generate or use provided account ID
  // Priority: customAccountId > userId (deterministic) > random
  const accountId = customAccountId || (generate_account_id(userId || null) as Hex);

  // Convert passkey signers to PasskeyPayload format for Rust SDK
  let passkeyPayload: typeof PasskeyPayload | null = null;
  if (passkeySigners && passkeySigners.length > 0) {
    if (passkeySigners.length > 1) {
      throw new Error(
        "Currently only one passkey signer is supported during deployment",
      );
    }

    const passkey = passkeySigners[0];

    // Convert hex strings to Uint8Array for WASM
    const credentialIdBytes = hexToBytes(passkey.credentialId);
    const passkeyXBytes = hexToBytes(passkey.publicKey.x);
    const passkeyYBytes = hexToBytes(passkey.publicKey.y);

    passkeyPayload = new PasskeyPayload(
      credentialIdBytes,
      passkeyXBytes,
      passkeyYBytes,
      passkey.originDomain,
    );
  }

  const encodedCallData = encode_deploy_account_call_data(
    accountId,
    eoaSigners || null,
    contracts.eoaValidator || null,
    passkeyPayload,
    contracts.webauthnValidator || null,
  ) as Hex;

  return {
    transaction: {
      to: contracts.factory,
      data: encodedCallData,
    },
    accountId,
  };
}

/**
 * Extract the deployed account address from transaction logs.
 * Searches for the AccountCreated event emitted by MSAFactory.
 *
 * @param logs - Transaction logs from the deployment transaction
 * @returns The deployed account address
 * @throws Error if AccountCreated event is not found in the logs
 *
 * @example
 * ```typescript
 * import { prepareDeploySmartAccount, getAccountAddressFromLogs } from "zksync-sso/client-new/actions";
 *
 * const { transaction } = prepareDeploySmartAccount({ ... });
 * const hash = await walletClient.sendTransaction(transaction);
 * const receipt = await publicClient.waitForTransactionReceipt({ hash });
 *
 * const deployedAddress = getAccountAddressFromLogs(receipt.logs);
 * console.log("Account deployed at:", deployedAddress);
 * ```
 */
export function getAccountAddressFromLogs(logs: Log[]): Address {
  // Event signature: AccountCreated(address indexed account, address indexed deployer)
  const accountCreatedTopic = keccak256(toHex("AccountCreated(address,address)"));

  // Find the log with AccountCreated event
  const log = logs.find((log) => log.topics[0] === accountCreatedTopic);

  if (!log || !log.topics[1]) {
    throw new Error("AccountCreated event not found in transaction logs");
  }

  // The account address is the first indexed parameter (topics[1])
  // Convert from 32-byte hex topic to 20-byte address
  const deployedAddress = `0x${log.topics[1].slice(-40)}` as Address;

  return deployedAddress;
}
