/**
 * WebAuthn helper for viem-based passkey smart accounts
 * Extracted from packages/sdk-platforms/web/src/webauthn-helpers.ts
 */

import { startAuthentication } from "@simplewebauthn/browser";
import { type Hex, hexToBytes, pad, toHex } from "viem";
// @ts-expect-error - TypeScript doesn't understand package.json exports with node module resolution
import { encode_passkey_signature } from "zksync-sso-web-sdk/bundler";

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

  // Normalize s to low-s form for EVM compatibility
  // secp256r1 curve order (n): 0xFFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551
  const secp256r1_n = BigInt("0xFFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551");
  const secp256r1_n_half = secp256r1_n / 2n;

  // Convert s to BigInt
  let sBigInt = 0n;
  for (let i = 0; i < s.length; i++) {
    sBigInt = (sBigInt << 8n) | BigInt(s[i]);
  }

  // If s > n/2, use n - s instead (low-s form)
  if (sBigInt > secp256r1_n_half) {
    sBigInt = secp256r1_n - sBigInt;

    // Convert back to Uint8Array
    const sBytes: number[] = [];
    let temp = sBigInt;
    while (temp > 0n) {
      sBytes.unshift(Number(temp & 0xFFn));
      temp = temp >> 8n;
    }
    s = new Uint8Array(sBytes);
  }

  return { r, s };
}

/**
 * Pad signature component to 32 bytes (right-align, pad left with zeros)
 */
function padTo32Bytes(value: Uint8Array): Uint8Array {
  return hexToBytes(pad(toHex(value), { size: 32 }));
}

export interface SignWithPasskeyOptions {
  /** The hash to sign (as hex string with 0x prefix) */
  hash: Hex;
  /** The credential ID (as hex string with 0x prefix) */
  credentialId: Hex;
  /** The validator contract address (will be prepended to signature) */
  validatorAddress: Hex;
  /** The origin/domain where the credential was created */
  rpId: string;
  /** The origin URL */
  origin: string;
}

/**
 * Sign a hash with a WebAuthn passkey and return complete signature
 *
 * This function handles:
 * 1. Converting the hash to a WebAuthn challenge (base64url)
 * 2. Requesting signature from the authenticator
 * 3. Parsing the DER-encoded signature
 * 4. ABI-encoding the signature in the format expected by WebAuthnValidator
 * 5. Prepending the validator address (ModularSmartAccount format)
 *
 * @param options - Configuration for the signature request
 * @returns Promise resolving to the complete signature (validator + ABI-encoded passkey sig)
 */
export async function signWithPasskey(
  options: SignWithPasskeyOptions,
): Promise<Hex> {
  const { hash, credentialId, validatorAddress, rpId } = options;

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

  // ABI encode the passkey signature using Rust function
  // Format: (bytes authenticatorData, string clientDataJSON, bytes32[2] rs, bytes credentialId)
  const passkeySignature = encode_passkey_signature(
    authenticatorData,
    clientDataJSON,
    rPadded,
    sPadded,
    credentialIdBytes,
  );

  // Prepend validator address (ModularSmartAccount format: 20 bytes validator + signature)
  const validatorBytes = hexToBytes(validatorAddress);
  const signatureBytes_final = hexToBytes(passkeySignature);
  const fullSignature = new Uint8Array(validatorBytes.length + signatureBytes_final.length);
  fullSignature.set(validatorBytes, 0);
  fullSignature.set(signatureBytes_final, validatorBytes.length);

  return `0x${Array.from(fullSignature).map((b) => b.toString(16).padStart(2, "0")).join("")}` as Hex;
}
