/**
 * WASM integration layer for Rust-based encoding functions
 *
 * This module provides TypeScript-friendly wrappers around the WASM functions
 * exported from the Rust SDK. It handles initialization and provides type-safe
 * interfaces for encoding operations.
 */

import type { Address, Hex } from "viem";

// Import WASM functions from the bundler build
// These functions are compiled from Rust and exposed via wasm-bindgen
let wasmModule: any | null = null;

/**
 * Lazy-load the WASM module
 * This ensures the module is only loaded when actually needed
 */
async function getWasmModule() {
  if (!wasmModule) {
    // Use dynamic import to load the WASM module
    // @ts-expect-error - TypeScript doesn't understand package.json exports with node module resolution
    wasmModule = await import("zksync-sso-web-sdk/bundler");
  }
  return wasmModule;
}

/**
 * Session specification for creating a new session
 * This type mirrors the Rust SessionSpec type
 */
export type SessionSpec = {
  signer: Address;
  expiresAt: string; // U48 as string
  feeLimit: UsageLimit;
  callPolicies: CallSpec[];
  transferPolicies: TransferSpec[];
};

export type UsageLimit = {
  limitType: "Unlimited" | "Lifetime" | "Allowance";
  limit: string; // U256 as string
  period: string; // U48 as string
};

export type TransferSpec = {
  target: Address;
  maxValuePerUse: string; // U256 as string
  valueLimit: UsageLimit;
};

export type CallSpec = {
  target: Address;
  selector: Hex; // 4-byte function selector
  maxValuePerUse: string; // U256 as string
  valueLimit: UsageLimit;
  constraints: WasmConstraint[];
};

export type WasmConstraint = {
  condition: "Unconstrained" | "Equal" | "Greater" | "Less" | "GreaterEqual" | "LessEqual" | "NotEqual";
  index: number;
  refValue: Hex; // 32-byte value
  limit: UsageLimit;
};

/**
 * Passkey payload for WebAuthn operations
 */
export type PasskeyPayload = {
  credentialId: Uint8Array;
  passkeyX: Uint8Array;
  passkeyY: Uint8Array;
  originDomain: string;
};

// ===== SESSION MANAGEMENT =====

/**
 * Encode call data for creating a session
 *
 * @param sessionSpec - The session specification
 * @param sessionValidatorAddress - Address of the session validator contract
 * @returns Hex-encoded call data
 */
export async function encodeCreateSessionCallData(
  sessionSpec: SessionSpec,
  sessionValidatorAddress: Address,
): Promise<Hex> {
  const wasm = await getWasmModule();
  const sessionSpecJson = JSON.stringify(sessionSpec);
  return wasm.encode_create_session_call_data(
    sessionSpecJson,
    sessionValidatorAddress,
  ) as Hex;
}

/**
 * Encode call data for revoking a session
 *
 * @param sessionHash - Hash of the session to revoke
 * @param sessionValidatorAddress - Address of the session validator contract
 * @returns Hex-encoded call data
 */
export async function encodeRevokeSessionCallData(
  sessionHash: Hex,
  sessionValidatorAddress: Address,
): Promise<Hex> {
  const wasm = await getWasmModule();
  return wasm.encode_revoke_session_call_data(
    sessionHash,
    sessionValidatorAddress,
  ) as Hex;
}

/**
 * Compute the hash of a session spec
 *
 * @param sessionSpec - The session specification
 * @returns Hex-encoded session hash
 */
export async function computeSessionHash(sessionSpec: SessionSpec): Promise<Hex> {
  const wasm = await getWasmModule();
  const sessionSpecJson = JSON.stringify(sessionSpec);
  return wasm.compute_session_hash(sessionSpecJson) as Hex;
}

// ===== PASSKEY MANAGEMENT =====

/**
 * Encode call data for adding a passkey to an account
 *
 * @param passkeyPayload - The passkey credentials
 * @param webauthnValidatorAddress - Address of the WebAuthn validator contract
 * @returns Hex-encoded call data
 */
export async function encodeAddPasskeyCallData(
  passkeyPayload: PasskeyPayload,
  webauthnValidatorAddress: Address,
): Promise<Hex> {
  const wasm = await getWasmModule();

  // Create WASM PasskeyPayload using the constructor
  const wasmPasskeyPayload = new wasm.PasskeyPayload(
    passkeyPayload.credentialId,
    passkeyPayload.passkeyX,
    passkeyPayload.passkeyY,
    passkeyPayload.originDomain,
  );

  return wasm.encode_add_passkey_call_data(
    wasmPasskeyPayload,
    webauthnValidatorAddress,
  ) as Hex;
}

/**
 * Encode call data for adding a validation key (passkey) to the WebAuthn validator
 *
 * @param passkeyPayload - The passkey credentials
 * @returns Hex-encoded call data
 */
export async function encodeAddValidationKeyCallData(
  passkeyPayload: PasskeyPayload,
): Promise<Hex> {
  const wasm = await getWasmModule();

  // Create WASM PasskeyPayload using the constructor
  const wasmPasskeyPayload = new wasm.PasskeyPayload(
    passkeyPayload.credentialId,
    passkeyPayload.passkeyX,
    passkeyPayload.passkeyY,
    passkeyPayload.originDomain,
  );

  return wasm.encode_add_validation_key_call_data(wasmPasskeyPayload) as Hex;
}

/**
 * Encode a passkey signature for on-chain verification
 *
 * @param authenticatorData - Raw authenticator data from WebAuthn
 * @param clientDataJSON - Client data JSON string from WebAuthn
 * @param r - R component of ECDSA signature (32 bytes)
 * @param s - S component of ECDSA signature (32 bytes)
 * @param credentialId - The credential ID bytes
 * @returns Hex-encoded ABI signature
 */
export async function encodePasskeySignature(
  authenticatorData: Uint8Array,
  clientDataJSON: string,
  r: Uint8Array,
  s: Uint8Array,
  credentialId: Uint8Array,
): Promise<Hex> {
  const wasm = await getWasmModule();
  return wasm.encode_passkey_signature(
    authenticatorData,
    clientDataJSON,
    r,
    s,
    credentialId,
  ) as Hex;
}

/**
 * Compute account ID from user ID
 * This is used to generate a unique identifier for smart accounts
 *
 * @param userId - Unique user identifier
 * @returns Hex-encoded account ID
 */
export async function computeAccountId(userId: string): Promise<Hex> {
  const wasm = await getWasmModule();
  return wasm.compute_account_id(userId) as Hex;
}

// ===== PASSKEY SIGNING =====

export type SignWithPasskeyOptions = {
  /** The hash to sign (as hex string with 0x prefix) */
  hash: Hex;
  /** The credential ID (as hex string with 0x prefix) */
  credentialId: Hex;
  /** The origin/domain where the credential was created */
  rpId: string;
  /** The origin URL */
  origin: string;
};

export type PasskeySignatureResult = {
  /** ABI-encoded signature */
  signature: Hex;
  /** Raw authenticator data (for debugging) */
  authenticatorData: Uint8Array;
  /** Client data JSON (for debugging) */
  clientDataJSON: string;
  /** Signature r component (for debugging) */
  r: Uint8Array;
  /** Signature s component (for debugging) */
  s: Uint8Array;
};

/**
 * Sign a hash with a WebAuthn passkey and return ABI-encoded signature
 *
 * This function handles:
 * 1. Converting the hash to a WebAuthn challenge (base64url)
 * 2. Requesting signature from the authenticator
 * 3. Parsing the DER-encoded signature
 * 4. ABI-encoding the signature in the format expected by WebAuthnValidator
 *
 * @param options - Configuration for the signature request
 * @returns Promise resolving to the ABI-encoded signature and components
 */
export async function signWithPasskey(
  options: SignWithPasskeyOptions,
): Promise<PasskeySignatureResult> {
  const wasm = await getWasmModule();
  return wasm.signWithPasskey(options);
}

// ===== TRANSACTION HELPERS =====

export type SendTransactionWithPasskeyOptions = {
  rpcUrl: string;
  bundlerUrl: string;
  entryPointAddress: Address;
  webauthnValidatorAddress: Address;
  accountAddress: Address;
  toAddress: Address;
  value: string;
  data?: Hex | null;
  credentialId: Hex;
  rpId: string;
  origin: string;
};

/**
 * Send a transaction from a smart account using passkey authentication
 *
 * This is a high-level convenience function that handles the complete flow:
 * 1. Prepare UserOperation (with stub signature)
 * 2. Request passkey signature from user
 * 3. Submit signed UserOperation
 *
 * For advanced use cases where you need more control over the signing process,
 * use the lower-level functions directly.
 *
 * @param options - Transaction configuration
 * @returns Promise resolving to transaction hash or result
 */
export async function sendTransactionWithPasskey(
  options: SendTransactionWithPasskeyOptions,
): Promise<string> {
  const wasm = await getWasmModule();
  return wasm.sendTransactionWithPasskey(options);
}

// ===== DEPLOYMENT ENCODING =====

export type EncodeDeployAccountCalldataParams = {
  /** Unique account ID (32 bytes) */
  accountId: Hex;
  /** Optional passkey payload for the account */
  passkeyPayload?: PasskeyPayload;
  /** Address of passkey validator (required if passkeyPayload is provided) */
  passkeyValidatorAddress?: Address;
  /** Address of session validator */
  sessionValidatorAddress: Address;
  /** Optional hex-encoded session initialization data */
  sessionData?: Hex;
  /** Address of guardian recovery validator */
  recoveryValidatorAddress: Address;
  /** Address of OIDC recovery validator */
  oidcRecoveryValidatorAddress: Address;
};

/**
 * Encode complete calldata for deployProxySsoAccount
 * This is the single function to use for account deployment - it returns
 * the complete calldata ready to send to the factory contract with viem.
 *
 * @param params - Deployment configuration
 * @returns Hex-encoded calldata for deployProxySsoAccount
 */
export async function encodeDeployAccountCalldata(
  params: EncodeDeployAccountCalldataParams,
): Promise<Hex> {
  const wasm = await getWasmModule();

  // Create WASM PasskeyPayload if provided
  const wasmPasskeyPayload = params.passkeyPayload
    ? new wasm.PasskeyPayload(
      params.passkeyPayload.credentialId,
      params.passkeyPayload.passkeyX,
      params.passkeyPayload.passkeyY,
      params.passkeyPayload.originDomain,
    )
    : undefined;

  return wasm.encode_deploy_account_calldata(
    params.accountId,
    wasmPasskeyPayload,
    params.passkeyValidatorAddress,
    params.sessionValidatorAddress,
    params.sessionData,
    params.recoveryValidatorAddress,
    params.oidcRecoveryValidatorAddress,
  ) as Hex;
}
