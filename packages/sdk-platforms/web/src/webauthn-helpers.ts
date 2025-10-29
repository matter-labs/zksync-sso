/**
 * WebAuthn helper functions for passkey operations
 * Handles the complexity of WebAuthn signature encoding and formatting
 */

import { startAuthentication } from "@simplewebauthn/browser";

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

/**
 * Convert Uint8Array to base64url string
 */
function uint8ArrayToBase64url(bytes: Uint8Array): string {
  let binary = "";
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i]);
  }

  const base64 = btoa(binary);
  return base64.replace(/\+/g, "-").replace(/\//g, "_").replace(/=/g, "");
}

/**
 * Convert base64url string to Uint8Array
 */
function base64urlToUint8Array(base64url: string): Uint8Array {
  const base64 = base64url.replace(/-/g, "+").replace(/_/g, "/");
  const padded = base64.padEnd(base64.length + (4 - (base64.length % 4)) % 4, "=");
  const binary = atob(padded);

  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }

  return bytes;
}

/**
 * Parse DER-encoded ECDSA signature to extract r and s components
 * DER format: 0x30 [total-length] 0x02 [r-length] [r] 0x02 [s-length] [s]
 */
function parseDerSignature(signatureBytes: Uint8Array): { r: Uint8Array; s: Uint8Array } {
  let offset = 0;

  if (signatureBytes[offset++] !== 0x30) {
    throw new Error("Invalid DER signature format");
  }
  offset++; // Skip total length

  if (signatureBytes[offset++] !== 0x02) {
    throw new Error("Invalid DER signature format - missing r marker");
  }
  const rLength = signatureBytes[offset++];
  let r = signatureBytes.slice(offset, offset + rLength);
  offset += rLength;

  if (signatureBytes[offset++] !== 0x02) {
    throw new Error("Invalid DER signature format - missing s marker");
  }
  const sLength = signatureBytes[offset++];
  let s = signatureBytes.slice(offset, offset + sLength);

  // Strip leading 0x00 bytes (DER adds these when high bit is set)
  while (r.length > 32 && r[0] === 0x00) {
    r = r.slice(1);
  }
  while (s.length > 32 && s[0] === 0x00) {
    s = s.slice(1);
  }

  // Ensure values fit in 32 bytes
  if (r.length > 32 || s.length > 32) {
    throw new Error(`Invalid signature component length: r=${r.length}, s=${s.length}`);
  }

  return { r, s };
}

/**
 * Pad signature component to 32 bytes (right-align, pad left with zeros)
 */
function padTo32Bytes(value: Uint8Array): Uint8Array {
  const padded = new Uint8Array(32);
  padded.set(value, 32 - value.length);
  return padded;
}

export interface SignWithPasskeyOptions {
  /** The hash to sign (as hex string with 0x prefix) */
  hash: string;
  /** The credential ID (as hex string with 0x prefix) */
  credentialId: string;
  /** The origin/domain where the credential was created */
  rpId: string;
  /** The origin URL */
  origin: string;
}

export interface PasskeySignatureResult {
  /** ABI-encoded signature: (bytes authenticatorData, string clientDataJSON, bytes32[2] rs, bytes credentialId) */
  signature: string;
  /** Raw authenticator data (for debugging) */
  authenticatorData: Uint8Array;
  /** Client data JSON (for debugging) */
  clientDataJSON: string;
  /** Signature r component (for debugging) */
  r: Uint8Array;
  /** Signature s component (for debugging) */
  s: Uint8Array;
}

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
  const { hash, credentialId, rpId } = options;

  // Convert hash to challenge bytes
  const challengeBytes = hexToBytes(hash);

  if (challengeBytes.length !== 32) {
    throw new Error(`Hash must be 32 bytes, got ${challengeBytes.length}`);
  }

  // Convert to base64url for WebAuthn
  const challengeBase64url = uint8ArrayToBase64url(challengeBytes);

  // Convert credential ID to base64url
  const credentialIdBytes = hexToBytes(credentialId);
  const credentialIdBase64url = uint8ArrayToBase64url(credentialIdBytes);

  // Request signature from authenticator
  const authResponse = await startAuthentication({
    optionsJSON: {
      challenge: challengeBase64url,
      rpId,
      userVerification: "required",
      allowCredentials: [
        {
          id: credentialIdBase64url,
          type: "public-key",
        },
      ],
    },
  });

  // Extract and decode response components
  const authenticatorData = base64urlToUint8Array(authResponse.response.authenticatorData);
  const clientDataJSONBytes = base64urlToUint8Array(authResponse.response.clientDataJSON);
  const clientDataJSON = new TextDecoder().decode(clientDataJSONBytes);
  const signatureBytes = base64urlToUint8Array(authResponse.response.signature);

  // Parse DER signature
  const { r, s } = parseDerSignature(signatureBytes);

  // Pad to 32 bytes
  const rPadded = padTo32Bytes(r);
  const sPadded = padTo32Bytes(s);

  // Convert to hex strings for WASM function
  const authenticatorDataHex = "0x" + Array.from(authenticatorData).map((b) => b.toString(16).padStart(2, "0")).join("");
  const rHex = "0x" + Array.from(rPadded).map((b) => b.toString(16).padStart(2, "0")).join("");
  const sHex = "0x" + Array.from(sPadded).map((b) => b.toString(16).padStart(2, "0")).join("");
  const credentialIdHex = "0x" + Array.from(credentialIdBytes).map((b) => b.toString(16).padStart(2, "0")).join("");

  // ABI encode the signature using WASM function
  // Format: (bytes authenticatorData, string clientDataJSON, bytes32[2] rs, bytes credentialId)
  const wasm = await import("../pkg-bundler/zksync_sso_erc4337_web_ffi");

  let signature: string;
  try {
    signature = wasm.abi_encode_passkey_signature(
      authenticatorDataHex,
      clientDataJSON,
      rHex,
      sHex,
      credentialIdHex,
    );
  } catch (error) {
    throw new Error(`Failed to ABI encode passkey signature: ${error}`);
  }

  return {
    signature,
    authenticatorData,
    clientDataJSON,
    r: rPadded,
    s: sPadded,
  };
}

/**
 * Create a minimal stub signature for gas estimation
 * This creates a properly formatted but non-functional signature with empty/zero values
 *
 * @param validatorAddress - The WebAuthn validator address (as hex string with 0x prefix)
 * @returns Hex-encoded stub signature (validator address + ABI-encoded empty signature)
 */
export async function createStubSignature(validatorAddress: string): Promise<string> {
  // Use WASM function for ABI encoding
  const wasm = await import("../pkg-bundler/zksync_sso_erc4337_web_ffi");
  return wasm.abi_encode_stub_passkey_signature(validatorAddress);
}
