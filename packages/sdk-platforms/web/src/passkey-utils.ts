/**
 * Simplified utilities for working with passkeys in the ZKSync SSO SDK
 *
 * This module provides high-level helpers that abstract away the complexity
 * of hex conversions, coordinate validation, and WASM function calls.
 */

import type { PasskeyPayload } from "../pkg-bundler/zksync_sso_erc4337_web_ffi";

/**
 * Passkey coordinates from WebAuthn registration
 */
export interface PasskeyCoordinates {
  credentialId: string; // hex string with 0x prefix
  x: string; // 32-byte hex string with 0x prefix
  y: string; // 32-byte hex string with 0x prefix
}

/**
 * Convert a hex string to Uint8Array
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

/**
 * Validate that coordinates are 32 bytes each
 */
function validateCoordinates(x: Uint8Array, y: Uint8Array): void {
  if (x.length !== 32) {
    throw new Error(`Passkey X coordinate must be 32 bytes, got ${x.length}`);
  }
  if (y.length !== 32) {
    throw new Error(`Passkey Y coordinate must be 32 bytes, got ${y.length}`);
  }
}

/**
 * Create a PasskeyPayload from hex string coordinates (simplified helper)
 *
 * @param credentials - The passkey coordinates from WebAuthn
 * @param origin - The origin domain (e.g., "https://example.com" or window.location.origin)
 * @returns A PasskeyPayload ready to use with SDK functions
 *
 * @example
 * ```typescript
 * const payload = createPasskeyPayloadFromHex({
 *   credentialId: "0x2868baa08431052f6c7541392a458f64",
 *   x: "0xe0a43b9c64a2357ea7f66a0551f57442fbd32031162d9be762800864168fae40",
 *   y: "0x450875e2c28222e81eb25ae58d095a3e7ca295faa3fc26fb0e558a0b571da501"
 * }, window.location.origin);
 * ```
 */
export async function createPasskeyPayloadFromHex(
  credentials: PasskeyCoordinates,
  origin: string,
): Promise<PasskeyPayload> {
  // Import PasskeyPayload dynamically
  const { PasskeyPayload } = await import("../pkg-bundler/zksync_sso_erc4337_web_ffi");

  // Convert hex strings to bytes
  const credentialId = hexToBytes(credentials.credentialId);
  const x = hexToBytes(credentials.x);
  const y = hexToBytes(credentials.y);

  // Validate coordinates
  validateCoordinates(x, y);

  // Create and return the payload
  return new PasskeyPayload(credentialId, x, y, origin);
}

/**
 * Quick passkey payload creation with automatic origin detection (simplified helper)
 *
 * @param credentials - The passkey coordinates
 * @returns A PasskeyPayload with automatically detected origin
 */
export async function createPasskeyPayloadFromHexAuto(
  credentials: PasskeyCoordinates,
): Promise<PasskeyPayload> {
  if (typeof window === "undefined") {
    throw new Error("createPasskeyPayloadFromHexAuto can only be used in browser environments");
  }
  return createPasskeyPayloadFromHex(credentials, window.location.origin);
}

/**
 * Load contract addresses from a contracts.json file
 *
 * @param url - URL to contracts.json (default: "/contracts.json")
 * @returns Contract addresses
 */
export async function loadContractAddresses(url = "/contracts.json") {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Failed to load contracts from ${url}: ${response.statusText}`);
  }
  return response.json();
}
