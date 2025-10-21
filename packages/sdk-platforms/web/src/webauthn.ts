/**
 * WebAuthn Helper Module
 *
 * This module provides helper functions for creating and managing WebAuthn credentials
 * using SimpleWebAuthn library. It extracts the public key coordinates needed for
 * on-chain verification.
 *
 * @module webauthn
 */

import type { PasskeyPayload } from "../pkg-bundler/zksync_sso_erc4337_web_ffi.js";

/**
 * Configuration options for creating a WebAuthn credential
 */
export interface CreateCredentialOptions {
  /**
   * Relying Party (RP) name - typically your application name
   */
  rpName?: string;

  /**
   * Relying Party ID - typically your domain (e.g., "example.com")
   * If not provided, defaults to window.location.hostname
   */
  rpId?: string;

  /**
   * User display name
   */
  userName?: string;

  /**
   * User email or identifier
   */
  userEmail?: string;

  /**
   * Timeout in milliseconds (default: 60000)
   */
  timeout?: number;

  /**
   * Authenticator attachment preference
   * - "platform": Use built-in authenticators (Touch ID, Face ID, Windows Hello)
   * - "cross-platform": Use external authenticators (security keys)
   */
  authenticatorAttachment?: "platform" | "cross-platform";
}

/**
 * Result of WebAuthn credential creation
 */
export interface WebAuthnCredential {
  /**
   * Raw credential ID (hex string with 0x prefix)
   */
  credentialId: string;

  /**
   * X coordinate of P-256 public key (32 bytes, hex string with 0x prefix)
   */
  publicKeyX: string;

  /**
   * Y coordinate of P-256 public key (32 bytes, hex string with 0x prefix)
   */
  publicKeyY: string;

  /**
   * Origin where the credential was created
   */
  origin: string;
}

/**
 * Convert a Uint8Array to a hex string with 0x prefix
 */
function bytesToHex(bytes: Uint8Array): string {
  return "0x" + Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

/**
 * Convert a base64url string to Uint8Array
 */
function base64urlToBuffer(base64url: string): Uint8Array {
  // Replace base64url characters with base64 equivalents
  const base64 = base64url.replace(/-/g, "+").replace(/_/g, "/");

  // Pad with '=' if necessary
  const padded = base64.padEnd(base64.length + (4 - (base64.length % 4)) % 4, "=");

  // Decode base64 to binary string
  const binary = atob(padded);

  // Convert binary string to Uint8Array
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }

  return bytes;
}

/**
 * Create a new WebAuthn credential and extract public key coordinates
 *
 * This function uses SimpleWebAuthn to create a passkey and extracts the
 * P-256 public key coordinates needed for on-chain verification.
 *
 * @param options - Configuration options for credential creation
 * @returns Promise resolving to the credential details
 * @throws Error if WebAuthn is not supported or credential creation fails
 */
export async function createWebAuthnCredential(
  options: CreateCredentialOptions = {},
): Promise<WebAuthnCredential> {
  // Check if SimpleWebAuthn is available
  let startRegistration: any;
  try {
    // @ts-expect-error - Optional peer dependency, may not be installed
    const simpleWebAuthn = await import("@simplewebauthn/browser");
    startRegistration = simpleWebAuthn.startRegistration;
  } catch {
    throw new Error(
      "SimpleWebAuthn is not installed. Please install @simplewebauthn/browser: "
      + "npm install @simplewebauthn/browser",
    );
  }

  // Check if WebAuthn is supported
  if (!window.PublicKeyCredential) {
    throw new Error("WebAuthn is not supported in this browser");
  }

  // Generate random challenge (32 bytes)
  const challenge = new Uint8Array(32);
  window.crypto.getRandomValues(challenge);

  // Generate random user ID (16 bytes)
  const userId = new Uint8Array(16);
  window.crypto.getRandomValues(userId);

  // Prepare registration options compatible with SimpleWebAuthn
  const registrationOptions = {
    challenge: bytesToHex(challenge).slice(2), // Remove 0x prefix for SimpleWebAuthn
    rp: {
      name: options.rpName || "zkSync SSO",
      id: options.rpId || window.location.hostname,
    },
    user: {
      id: bytesToHex(userId).slice(2), // Remove 0x prefix
      name: options.userEmail || "user@example.com",
      displayName: options.userName || "Demo User",
    },
    pubKeyCredParams: [
      {
        type: "public-key" as const,
        alg: -7, // ES256 (P-256)
      },
    ],
    authenticatorSelection: {
      authenticatorAttachment: options.authenticatorAttachment || "platform",
      requireResidentKey: false,
      residentKey: "preferred" as const,
      userVerification: "required" as const,
    },
    timeout: options.timeout || 60000,
    attestation: "none" as const,
  };

  // Create the credential using SimpleWebAuthn
  const credential = await startRegistration(registrationOptions);

  // Parse the response to extract public key coordinates
  const credentialIdBytes = base64urlToBuffer(credential.id);
  const credentialId = bytesToHex(credentialIdBytes);

  // The response includes the public key in the attestationObject
  // SimpleWebAuthn already handles CBOR parsing for us
  // We need to extract the public key coordinates from the response

  // Parse the authenticator data which contains the credential public key
  const authenticatorData = base64urlToBuffer(credential.response.authenticatorData);

  // The public key is encoded in COSE format after the authenticator data header
  // Structure: RP ID Hash (32) + Flags (1) + Counter (4) + Attested Cred Data
  // Attested Cred Data: AAGUID (16) + Cred ID Length (2) + Cred ID + COSE Key

  let offset = 37; // Skip: RP ID hash (32) + flags (1) + counter (4)
  offset += 16; // Skip AAGUID

  // Read credential ID length (2 bytes, big endian)
  const credIdLength = (authenticatorData[offset] << 8) | authenticatorData[offset + 1];
  offset += 2;

  // Skip credential ID
  offset += credIdLength;

  // Now we're at the COSE key
  const coseKey = authenticatorData.slice(offset);

  // Parse COSE key to extract x and y coordinates
  const { x, y } = parseCOSEKey(coseKey);

  return {
    credentialId,
    publicKeyX: bytesToHex(x),
    publicKeyY: bytesToHex(y),
    origin: window.location.origin,
  };
}

/**
 * Parse COSE key format to extract P-256 public key coordinates
 *
 * COSE key format for P-256:
 * - kty (1): 2 (EC2)
 * - alg (3): -7 (ES256)
 * - crv (-1): 1 (P-256)
 * - x (-2): 32 bytes
 * - y (-3): 32 bytes
 */
function parseCOSEKey(coseKey: Uint8Array): { x: Uint8Array; y: Uint8Array } {
  const dataView = new DataView(coseKey.buffer, coseKey.byteOffset, coseKey.byteLength);
  let offset = 0;

  // Read the first byte (should be a map)
  const firstByte = dataView.getUint8(offset++);
  if ((firstByte & 0xE0) !== 0xA0) {
    throw new Error("Expected CBOR map for COSE key");
  }

  const mapSize = firstByte & 0x1F;
  const keyData: { [key: number]: Uint8Array | number } = {};

  for (let i = 0; i < mapSize; i++) {
    // Read key (integer)
    let key = dataView.getInt8(offset);
    offset++;

    // Handle negative integers
    if (key >= 0x20 && key <= 0x37) {
      key = -1 - (key - 0x20);
    }

    // Read value
    const valueType = dataView.getUint8(offset);
    offset++;

    if ((valueType & 0xE0) === 0x40) {
      // Byte string
      let length = valueType & 0x1F;

      // Handle extended length encoding
      if (length === 24) {
        length = dataView.getUint8(offset);
        offset++;
      }

      const value = new Uint8Array(coseKey.buffer, coseKey.byteOffset + offset, length);
      keyData[key] = value;
      offset += length;
    } else if (valueType <= 0x17) {
      // Small positive integer
      keyData[key] = valueType;
    } else if (valueType >= 0x20 && valueType <= 0x37) {
      // Small negative integer
      keyData[key] = -1 - (valueType - 0x20);
    }
  }

  // Extract x and y coordinates
  const x = keyData[-2] as Uint8Array;
  const y = keyData[-3] as Uint8Array;

  if (!x || !y) {
    throw new Error("Failed to extract public key coordinates from COSE key");
  }

  if (x.length !== 32 || y.length !== 32) {
    throw new Error(`Invalid coordinate length: x=${x.length}, y=${y.length} (expected 32 bytes each)`);
  }

  return { x, y };
}

/**
 * Create a PasskeyPayload from WebAuthn credential
 *
 * This is a convenience function that creates a WebAuthn credential and
 * immediately converts it to a PasskeyPayload for use with the deploy_account
 * function.
 *
 * @param options - Configuration options for credential creation
 * @returns Promise resolving to PasskeyPayload instance
 */
export async function createPasskeyPayload(
  options: CreateCredentialOptions = {},
): Promise<PasskeyPayload> {
  // Import PasskeyPayload constructor from WASM
  const { PasskeyPayload } = await import("../pkg-bundler/zksync_sso_erc4337_web_ffi.js");

  // Create the credential
  const credential = await createWebAuthnCredential(options);

  // Convert hex strings to Uint8Array
  const credentialId = hexToBytes(credential.credentialId);
  const passkeyX = hexToBytes(credential.publicKeyX);
  const passkeyY = hexToBytes(credential.publicKeyY);

  // Create and return PasskeyPayload
  return new PasskeyPayload(
    credentialId,
    passkeyX,
    passkeyY,
    credential.origin,
  );
}

/**
 * Convert hex string to Uint8Array
 */
function hexToBytes(hex: string): Uint8Array {
  const cleanHex = hex.startsWith("0x") ? hex.slice(2) : hex;

  if (cleanHex.length % 2 !== 0) {
    throw new Error(`Invalid hex string length: ${cleanHex.length}`);
  }

  const bytes = new Uint8Array(cleanHex.length / 2);
  for (let i = 0; i < cleanHex.length; i += 2) {
    bytes[i / 2] = parseInt(cleanHex.substr(i, 2), 16);
  }

  return bytes;
}
