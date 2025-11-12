import type { Address, Hex } from "viem";
import { hexToBytes } from "viem";
import {
  encode_add_passkey_call_data,
  PasskeyPayload,
  // @ts-expect-error - TypeScript doesn't understand package.json exports
} from "zksync-sso-web-sdk/bundler";

/**
 * Parameters for adding a passkey to a smart account
 */
export type AddPasskeyParams = {
  /** Smart account address */
  account: Address;

  /** Contract addresses */
  contracts: {
    /** WebAuthn validator address */
    webauthnValidator: Address;
  };

  /** Passkey signer to add */
  passkeySigner: {
    /** Hex-encoded credential ID */
    credentialId: Hex;
    /** Public key coordinates */
    publicKey: { x: Hex; y: Hex };
    /** Origin domain (e.g., "https://example.com" or window.location.origin) */
    originDomain: string;
  };
};

/**
 * Result from addPasskey containing transaction data
 */
export type AddPasskeyResult = {
  /** Transaction to send to add the passkey */
  transaction: {
    /** Smart account address */
    to: Address;
    /** Encoded execute() call data */
    data: Hex;
  };
};

/**
 * Create a transaction to add a passkey to a smart account.
 * This function does NOT send the transaction - it returns the transaction data
 * that must be signed by an existing signer on the account and sent via any means
 * (EOA, bundler, etc).
 *
 * Uses Rust WASM SDK for encoding (no RPC calls).
 *
 * @param params - Parameters including account address, passkey details, and validator
 * @returns Transaction data to add the passkey
 *
 * @example
 * ```typescript
 * import { addPasskey } from "zksync-sso/client-new/actions";
 *
 * const { transaction } = addPasskey({
 *   account: "0x...", // Your smart account address
 *   contracts: {
 *     webauthnValidator: "0x...",
 *   },
 *   passkeySigner: {
 *     credentialId: "0x...",
 *     publicKey: { x: "0x...", y: "0x..." },
 *     originDomain: window.location.origin,
 *   },
 * });
 *
 * // Send transaction via your preferred method
 * // Must be signed by an existing signer on the account
 * const hash = await smartAccount.sendTransaction({
 *   to: transaction.to,
 *   data: transaction.data,
 * });
 * ```
 */
export function addPasskey(params: AddPasskeyParams): AddPasskeyResult {
  const { account, contracts, passkeySigner } = params;

  // Convert passkey signer to PasskeyPayload format for Rust SDK
  // Convert hex strings to Uint8Array for WASM
  const credentialIdBytes = hexToBytes(passkeySigner.credentialId);
  const passkeyXBytes = hexToBytes(passkeySigner.publicKey.x);
  const passkeyYBytes = hexToBytes(passkeySigner.publicKey.y);

  const passkeyPayload = new PasskeyPayload(
    credentialIdBytes,
    passkeyXBytes,
    passkeyYBytes,
    passkeySigner.originDomain,
  );

  // Encode the call data using Rust SDK
  const encodedCallData = encode_add_passkey_call_data(
    passkeyPayload,
    contracts.webauthnValidator,
  ) as Hex;

  return {
    transaction: {
      to: account,
      data: encodedCallData,
    },
  };
}
