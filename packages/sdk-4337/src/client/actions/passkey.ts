import type { Address, Hex, PublicClient } from "viem";
import { hexToBytes, toHex } from "viem";
import { readContract } from "viem/actions";
import {
  decode_get_account_list_result,
  encode_add_passkey_call_data,
  encode_get_account_list_call_data,
  PasskeyPayload,
} from "zksync-sso-web-sdk/bundler";

import { WebAuthnValidatorAbi } from "../../abi/WebAuthnValidator.js";
import { base64urlToUint8Array } from "../passkey/webauthn.js";

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

/**
 * Parameters for finding addresses by passkey credential ID
 */
export type FindAddressesByPasskeyParams = {
  /** Public client for making RPC calls */
  client: PublicClient;

  /** Contract addresses */
  contracts: {
    /** WebAuthn validator address */
    webauthnValidator: Address;
  };

  /** Passkey credential details */
  passkey: {
    /** Hex-encoded credential ID */
    credentialId: Hex;
    /** Origin domain (e.g., "https://example.com" or window.location.origin) */
    originDomain: string;
  };
};

/**
 * Result from finding addresses by passkey
 */
export type FindAddressesByPasskeyResult = {
  /** Array of smart account addresses associated with this passkey */
  addresses: Address[];
};

/**
 * Find all smart account addresses associated with a passkey credential ID.
 * This queries the WebAuthnValidator contract to get the list of accounts
 * that have registered this specific passkey.
 *
 * Uses Rust WASM SDK for encoding/decoding with viem for RPC calls.
 *
 * @param params - Parameters including client, validator address, and passkey details
 * @returns Array of addresses associated with the passkey
 *
 * @example
 * ```typescript
 * import { createPublicClient, http } from "viem";
 * import { findAddressesByPasskey } from "zksync-sso-4337/client";
 *
 * const publicClient = createPublicClient({
 *   chain: zkSyncSepoliaTestnet,
 *   transport: http(),
 * });
 *
 * const { addresses } = await findAddressesByPasskey({
 *   client: publicClient,
 *   contracts: {
 *     webauthnValidator: "0x...",
 *   },
 *   passkey: {
 *     credentialId: "0x...",
 *     originDomain: window.location.origin,
 *   },
 * });
 *
 * console.log(`Found ${addresses.length} accounts for this passkey`);
 * ```
 */
export async function findAddressesByPasskey(
  params: FindAddressesByPasskeyParams,
): Promise<FindAddressesByPasskeyResult> {
  const { client, contracts, passkey } = params;

  // Encode the call data using Rust SDK
  const callData = encode_get_account_list_call_data(
    passkey.originDomain,
    passkey.credentialId,
  ) as Hex;

  // Make the RPC call to the WebAuthnValidator contract
  const result = await client.call({
    to: contracts.webauthnValidator,
    data: callData,
  });

  // Decode the result using Rust SDK
  // Result is a JSON string of address array
  const addressesJson = decode_get_account_list_result(result.data as Hex);
  const addresses = JSON.parse(addressesJson) as Address[];

  return { addresses };
}

/**
 * Parameters for fetching account details by passkey
 */
export type FetchAccountParams = {
  /** Public client for making RPC calls */
  client: PublicClient;

  /** Contract addresses */
  contracts: {
    /** WebAuthn validator address */
    webauthnValidator: Address;
  };

  /** Origin domain (e.g., "https://example.com" or window.location.origin) */
  originDomain: string;

  /** Optional: credential ID if known, otherwise will prompt user */
  credentialId?: string;
};

/**
 * Result from fetching account details
 */
export type FetchAccountResult = {
  /** Credential ID (username) */
  credentialId: string;
  /** Smart account address */
  address: Address;
  /** Passkey public key as Uint8Array (COSE format) */
  passkeyPublicKey: Uint8Array;
};

/**
 * Fetch account details for a passkey credential.
 * If credentialId is not provided, will prompt the user to select a passkey.
 * This queries the WebAuthnValidator contract to get the account address and public key.
 *
 * @param params - Parameters including client, validator address, and optional credential ID
 * @returns Account details including credentialId, address, and public key
 *
 * @example
 * ```typescript
 * import { createPublicClient, http } from "viem";
 * import { fetchAccount } from "zksync-sso-4337/actions";
 *
 * const publicClient = createPublicClient({
 *   chain: zkSyncSepoliaTestnet,
 *   transport: http(),
 * });
 *
 * const { credentialId, address, passkeyPublicKey } = await fetchAccount({
 *   client: publicClient,
 *   contracts: {
 *     webauthnValidator: "0x...",
 *   },
 *   originDomain: window.location.origin,
 * });
 *
 * console.log(`Account address: ${address}`);
 * ```
 */
export async function fetchAccount(
  params: FetchAccountParams,
): Promise<FetchAccountResult> {
  const { client, contracts, originDomain, credentialId: providedCredentialId } = params;

  let credentialId = providedCredentialId;

  // If no credential ID provided, prompt user to select a passkey
  if (!credentialId) {
    const credential = (await navigator.credentials.get({
      publicKey: {
        challenge: new Uint8Array(32), // Dummy challenge
        userVerification: "discouraged",
      },
    })) as PublicKeyCredential | null;

    if (!credential) {
      throw new Error("No passkey credential selected");
    }

    credentialId = credential.id;
  }

  // Convert credential ID to hex
  const credentialIdHex = toHex(base64urlToUint8Array(credentialId));

  // Get account list for this credential
  const { addresses } = await findAddressesByPasskey({
    client,
    contracts,
    passkey: {
      credentialId: credentialIdHex,
      originDomain,
    },
  });

  if (addresses.length === 0) {
    throw new Error(`No account found for credential ID: ${credentialId}`);
  }

  // Get the first account (most common case is one account per passkey)
  const address = addresses[0];

  // Get the public key for this account
  const publicKeyCoords = await readContract(client, {
    abi: WebAuthnValidatorAbi,
    address: contracts.webauthnValidator,
    functionName: "getAccountKey",
    args: [originDomain, credentialIdHex, address],
  });

  if (!publicKeyCoords || !publicKeyCoords[0] || !publicKeyCoords[1]) {
    throw new Error(`Passkey credentials not found in on-chain module for passkey ${credentialId}`);
  }

  // Convert the public key coordinates back to COSE format
  // The coordinates are returned as bytes32[2], we need to convert them to a COSE public key
  // For now, we'll return them as a simple concatenated Uint8Array
  // This matches what the legacy SDK's getPasskeySignatureFromPublicKeyBytes expects
  const xBytes = hexToBytes(publicKeyCoords[0]);
  const yBytes = hexToBytes(publicKeyCoords[1]);

  // Create COSE-encoded public key (simplified version)
  // This is a minimal CBOR map encoding for ES256 key
  const coseKey = new Uint8Array([
    0xa5, // Map with 5 items
    0x01, 0x02, // kty: 2 (EC2)
    0x03, 0x26, // alg: -7 (ES256)
    0x20, 0x01, // crv: 1 (P-256)
    0x21, 0x58, 0x20, ...xBytes, // x: 32-byte coordinate
    0x22, 0x58, 0x20, ...yBytes, // y: 32-byte coordinate
  ]);

  return {
    credentialId,
    address,
    passkeyPublicKey: coseKey,
  };
}
