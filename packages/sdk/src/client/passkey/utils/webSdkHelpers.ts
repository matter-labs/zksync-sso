/**
 * Helper utilities for Web-SDK integration
 *
 * These utilities help convert between the SDK's existing formats
 * and the web-SDK's expected formats.
 */

/**
 * Convert web-SDK public key coordinates to SDK format
 *
 * The web-SDK returns separate X and Y coordinates (32 bytes each).
 * The SDK expects a full uncompressed P-256 key (65 bytes: [0x04][X][Y]).
 *
 * @param publicKeyX - X coordinate as hex string (with or without 0x prefix)
 * @param publicKeyY - Y coordinate as hex string (with or without 0x prefix)
 * @returns Full uncompressed public key (65 bytes)
 *
 * @example
 * ```typescript
 * import { createWebAuthnCredential } from "zksync-sso-web-sdk/bundler";
 * import { publicKeyCoordinatesToFull } from "zksync-sso/client/passkey";
 *
 * const credential = await createWebAuthnCredential({ ... });
 *
 * const fullPublicKey = publicKeyCoordinatesToFull(
 *   credential.publicKeyX,
 *   credential.publicKeyY
 * );
 *
 * // Use with SDK
 * const client = createZksyncPasskeyClient({
 *   credentialPublicKey: fullPublicKey,
 *   // ...
 * });
 * ```
 */
export function publicKeyCoordinatesToFull(
  publicKeyX: string,
  publicKeyY: string,
): Uint8Array {
  const xBytes = hexToBytes(publicKeyX);
  const yBytes = hexToBytes(publicKeyY);

  if (xBytes.length !== 32) {
    throw new Error(`Invalid X coordinate length: ${xBytes.length} (expected 32)`);
  }
  if (yBytes.length !== 32) {
    throw new Error(`Invalid Y coordinate length: ${yBytes.length} (expected 32)`);
  }

  // Create uncompressed key: [0x04][X][Y]
  return new Uint8Array([0x04, ...xBytes, ...yBytes]);
}

/**
 * Convert full SDK public key to web-SDK coordinate format
 *
 * @param fullPublicKey - Full uncompressed public key (65 bytes)
 * @returns Object with X and Y coordinates as hex strings
 *
 * @example
 * ```typescript
 * const { publicKeyX, publicKeyY } = publicKeyFullToCoordinates(
 *   client.credentialPublicKey
 * );
 * ```
 */
export function publicKeyFullToCoordinates(
  fullPublicKey: Uint8Array,
): { publicKeyX: string; publicKeyY: string } {
  if (fullPublicKey.length !== 65) {
    throw new Error(
      `Invalid public key length: ${fullPublicKey.length} (expected 65)`,
    );
  }
  if (fullPublicKey[0] !== 0x04) {
    throw new Error("Invalid public key format (expected uncompressed key starting with 0x04)");
  }

  const xBytes = fullPublicKey.slice(1, 33);
  const yBytes = fullPublicKey.slice(33, 65);

  return {
    publicKeyX: bytesToHex(xBytes),
    publicKeyY: bytesToHex(yBytes),
  };
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
    bytes[i / 2] = parseInt(cleanHex.substring(i, i + 2), 16);
  }

  return bytes;
}

/**
 * Convert Uint8Array to hex string with 0x prefix
 */
function bytesToHex(bytes: Uint8Array): string {
  return `0x${Array.from(bytes).map((b) => b.toString(16).padStart(2, "0")).join("")}`;
}

/**
 * Convert base64url credential ID to hex string
 *
 * @param credentialId - Base64url encoded credential ID
 * @returns Hex string with 0x prefix
 *
 * @example
 * ```typescript
 * const credentialIdHex = credentialIdToHex(credential.id);
 * ```
 */
export function credentialIdToHex(credentialId: string): string {
  const bytes = base64UrlToBytes(credentialId);
  return bytesToHex(bytes);
}

/**
 * Convert hex credential ID to base64url
 *
 * @param credentialIdHex - Hex string (with or without 0x prefix)
 * @returns Base64url encoded string
 */
export function credentialIdToBase64Url(credentialIdHex: string): string {
  const bytes = hexToBytes(credentialIdHex);
  return bytesToBase64Url(bytes);
}

/**
 * Convert base64url to Uint8Array
 */
function base64UrlToBytes(base64url: string): Uint8Array {
  // Replace URL-safe characters
  const base64 = base64url.replace(/-/g, "+").replace(/_/g, "/");

  // Pad with '=' if necessary
  const padded = base64.padEnd(
    base64.length + ((4 - (base64.length % 4)) % 4),
    "=",
  );

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
 * Convert Uint8Array to base64url
 */
function bytesToBase64Url(bytes: Uint8Array): string {
  let binary = "";
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i]);
  }

  const base64 = btoa(binary);
  // Convert base64 to base64url
  return base64.replace(/\+/g, "-").replace(/\//g, "_").replace(/=/g, "");
}
