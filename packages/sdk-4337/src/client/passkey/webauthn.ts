/**
 * WebAuthn helper for viem-based passkey smart accounts
 * Includes credential creation and signing functions
 */

import { type PublicKeyCredentialCreationOptionsJSON, startAuthentication, startRegistration } from "@simplewebauthn/browser";
import { bytesToHex, type Hex, hexToBytes, pad, toHex } from "viem";
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
export function base64urlToUint8Array(base64url: string): Uint8Array {
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
  const signatureBytes_final = hexToBytes(passkeySignature as Hex);
  const fullSignature = new Uint8Array(validatorBytes.length + signatureBytes_final.length);
  fullSignature.set(validatorBytes, 0);
  fullSignature.set(signatureBytes_final, validatorBytes.length);

  const hexString = bytesToHex(fullSignature);
  return hexString as Hex;
}

// ============================================================================
// WebAuthn Credential Creation
// ============================================================================

/**
 * Configuration options for creating a WebAuthn credential
 */
export interface CreateCredentialOptions {
  /**
   * Relying Party (RP) name - typically your application name
   */
  rpName: string;

  /**
   * Relying Party ID - typically your domain (e.g., "example.com")
   */
  rpId: string;

  displayName: string;
  name: string;

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
  credentialId: Hex;

  /**
   * Raw credential ID (base64url encoded string)
   */
  credentialIdBase64url: string;

  publicKey: {
    /**
     * X coordinate of P-256 public key (32 bytes, hex string with 0x prefix)
     */
    x: Hex;

    /**
     * Y coordinate of P-256 public key (32 bytes, hex string with 0x prefix)
     */
    y: Hex;
  };

  /**
   * Origin where the credential was created
   */
  origin: string;
}

/**
 * Convert Uint8Array to base64url encoding (for SimpleWebAuthn registration)
 */
function arrayBufferToBase64url(buffer: Uint8Array): string {
  let binary = "";
  for (let i = 0; i < buffer.length; i++) {
    binary += String.fromCharCode(buffer[i]);
  }
  const base64 = btoa(binary);
  // Convert base64 to base64url
  return base64.replace(/\+/g, "-").replace(/\//g, "_").replace(/=/g, "");
}

/**
 * Convert base64url string to Uint8Array (for parsing registration response)
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
export async function createWebAuthnCredential(options: CreateCredentialOptions): Promise<WebAuthnCredential> {
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

  // Prepare registration options for SimpleWebAuthn
  const registrationOptions: PublicKeyCredentialCreationOptionsJSON = {
    challenge: arrayBufferToBase64url(challenge),
    rp: {
      name: options.rpName,
      id: options.rpId,
    },
    user: {
      id: arrayBufferToBase64url(userId),
      name: options.name,
      displayName: options.displayName,
    },
    pubKeyCredParams: [
      {
        type: "public-key" as const,
        alg: -7, // ES256 (P-256)
      },
    ],
    authenticatorSelection: {
      authenticatorAttachment: options.authenticatorAttachment,
      residentKey: "required" as const,
      userVerification: "discouraged" as const,
    },
    timeout: options.timeout || 60000,
    attestation: "none" as const,
  };

  // Create the credential using SimpleWebAuthn
  const credential = await startRegistration({ optionsJSON: registrationOptions });

  // Parse the authenticator data to extract the public key
  // SimpleWebAuthn returns base64url-encoded strings
  const authenticatorDataBase64url = credential.response.authenticatorData;
  if (!authenticatorDataBase64url) {
    throw new Error("No authenticator data in credential response");
  }
  const authenticatorData = base64urlToBuffer(authenticatorDataBase64url);

  // Authenticator data structure:
  // - 32 bytes: RP ID hash
  // - 1 byte: flags
  // - 4 bytes: signature counter
  // - 16 bytes: AAGUID
  // - 2 bytes: credential ID length
  // - N bytes: credential ID
  // - M bytes: COSE public key

  const credIdLengthOffset = 53; // 32 + 1 + 4 + 16
  const credIdLength = (authenticatorData[credIdLengthOffset] << 8)
    | authenticatorData[credIdLengthOffset + 1];

  const credIdOffset = credIdLengthOffset + 2;
  const credId = authenticatorData.slice(credIdOffset, credIdOffset + credIdLength);

  // Extract COSE public key
  const coseKeyOffset = credIdOffset + credIdLength;
  const coseKey = authenticatorData.slice(coseKeyOffset);

  // Parse COSE key to extract public key coordinates
  const [xBuffer, yBuffer] = getPublicKeyBytesFromPasskeySignature(coseKey);

  console.log({
    credential,
    credentialId: bytesToHex(credId),
  });

  return {
    credentialId: bytesToHex(credId),
    credentialIdBase64url: credential.id,
    publicKey: {
      x: bytesToHex(xBuffer),
      y: bytesToHex(yBuffer),
    },
    origin: window.location.origin,
  };
}

// ============================================================================
// COSE/CBOR Parsing Functions (browser-compatible, no Node Buffer dependency)
// ============================================================================

enum COSEKEYS {
  kty = 1, // Key Type
  alg = 3, // Algorithm
  crv = -1, // Curve for EC keys
  x = -2, // X coordinate for EC keys
  y = -3, // Y coordinate for EC keys
}

type COSEPublicKeyMap = Map<COSEKEYS, number | Uint8Array>;

// CBOR Decoding functions (only what we need for parsing COSE keys)

function decodeMap(buffer: Uint8Array): COSEPublicKeyMap {
  const map = new Map<COSEKEYS, number | Uint8Array>();
  let offset = 1; // Start after the map header

  const mapHeader = buffer[0];
  const mapSize = mapHeader & 0x1F; // Number of pairs

  for (let i = 0; i < mapSize; i++) {
    const [key, keyLength] = decodeInt(buffer, offset);
    offset += keyLength;

    const [value, valueLength] = decodeValue(buffer, offset);
    offset += valueLength;

    map.set(key as COSEKEYS, value);
  }

  return map;
}

function decodeInt(buffer: Uint8Array, offset: number): [number, number] {
  const intByte = buffer[offset];

  if (intByte < 24) {
    // Small positive integer (0–23)
    return [intByte, 1];
  } else if (intByte === 0x18) {
    // 1-byte unsigned integer
    return [buffer[offset + 1], 2];
  } else if (intByte === 0x19) {
    // 2-byte unsigned integer
    const value = (buffer[offset + 1] << 8) | buffer[offset + 2];
    return [value, 3];
  } else if (intByte >= 0x20 && intByte <= 0x37) {
    // Small negative integer (-1 to -24)
    return [-(intByte - 0x20) - 1, 1];
  } else if (intByte === 0x38) {
    // 1-byte negative integer
    return [-1 - buffer[offset + 1], 2];
  } else if (intByte === 0x39) {
    // 2-byte negative integer
    const value = (buffer[offset + 1] << 8) | buffer[offset + 2];
    return [-1 - value, 3];
  } else {
    throw new Error(`Unsupported integer format: ${intByte}`);
  }
}

function decodeBytes(buffer: Uint8Array, offset: number): [Uint8Array, number] {
  const lengthByte = buffer[offset];
  if (lengthByte >= 0x40 && lengthByte <= 0x57) {
    const length = lengthByte - 0x40;
    return [buffer.slice(offset + 1, offset + 1 + length), length + 1];
  } else if (lengthByte === 0x58) {
    // Byte array with 1-byte length prefix
    const length = buffer[offset + 1];
    return [buffer.slice(offset + 2, offset + 2 + length), length + 2];
  } else {
    throw new Error("Unsupported byte format");
  }
}

function decodeValue(buffer: Uint8Array, offset: number): [number | Uint8Array, number] {
  const type = buffer[offset];
  if (type >= 0x40 && type <= 0x5F) {
    // Byte array
    return decodeBytes(buffer, offset);
  } else {
    return decodeInt(buffer, offset);
  }
}

/**
 * Parse COSE key to extract P-256 public key coordinates
 * Browser-compatible version using Uint8Array (no Node Buffer dependency)
 * @param publicPasskey - CBOR-encoded COSE public key
 * @returns Tuple of [x, y] coordinates as Uint8Arrays
 */
export function getPublicKeyBytesFromPasskeySignature(publicPasskey: Uint8Array): [Uint8Array, Uint8Array] {
  const cosePublicKey = decodeMap(publicPasskey);
  const x = cosePublicKey.get(COSEKEYS.x) as Uint8Array;
  const y = cosePublicKey.get(COSEKEYS.y) as Uint8Array;

  if (!x || !y) {
    throw new Error(`Failed to extract x and y coordinates from COSE key ${publicPasskey}`);
  }

  if (x.length !== 32 || y.length !== 32) {
    throw new Error(`Invalid coordinate length: x=${x.length}, y=${y.length} (expected 32 bytes each)`);
  }

  return [x, y];
}

// ============================================================================
// COSE/CBOR Encoding Functions
// ============================================================================

// Encode an integer in CBOR format
function encodeInt(int: number): Uint8Array {
  if (int >= 0 && int <= 23) {
    return new Uint8Array([int]);
  } else if (int >= 24 && int <= 255) {
    return new Uint8Array([0x18, int]);
  } else if (int >= 256 && int <= 65535) {
    const buf = new Uint8Array(3);
    buf[0] = 0x19;
    buf[1] = (int >> 8) & 0xFF;
    buf[2] = int & 0xFF;
    return buf;
  } else if (int < 0 && int >= -24) {
    return new Uint8Array([0x20 - (int + 1)]);
  } else if (int < -24 && int >= -256) {
    return new Uint8Array([0x38, -int - 1]);
  } else if (int < -256 && int >= -65536) {
    const buf = new Uint8Array(3);
    buf[0] = 0x39;
    const value = -int - 1;
    buf[1] = (value >> 8) & 0xFF;
    buf[2] = value & 0xFF;
    return buf;
  } else {
    throw new Error("Unsupported integer range");
  }
}

// Encode a byte array in CBOR format
function encodeBytes(bytes: Uint8Array): Uint8Array {
  if (bytes.length <= 23) {
    const result = new Uint8Array(1 + bytes.length);
    result[0] = 0x40 + bytes.length;
    result.set(bytes, 1);
    return result;
  } else if (bytes.length < 256) {
    const result = new Uint8Array(2 + bytes.length);
    result[0] = 0x58;
    result[1] = bytes.length;
    result.set(bytes, 2);
    return result;
  } else {
    throw new Error("Unsupported byte array length");
  }
}

// Encode a map in CBOR format
function encodeMap(map: COSEPublicKeyMap): Uint8Array {
  const encodedItems: Uint8Array[] = [];

  // CBOR map header
  const mapHeader = 0xA0 | map.size;
  encodedItems.push(new Uint8Array([mapHeader]));

  map.forEach((value, key) => {
    // Encode the key
    encodedItems.push(encodeInt(key));

    // Encode the value based on its type
    if (value instanceof Uint8Array) {
      encodedItems.push(encodeBytes(value));
    } else {
      encodedItems.push(encodeInt(value));
    }
  });

  // Concatenate all encoded items
  const totalLength = encodedItems.reduce((sum, item) => sum + item.length, 0);
  const result = new Uint8Array(totalLength);
  let offset = 0;
  for (const item of encodedItems) {
    result.set(item, offset);
    offset += item.length;
  }
  return result;
}

/**
 * Encodes x,y hex coordinates into a CBOR-encoded COSE public key format.
 * Browser-compatible version using Uint8Array (no Node Buffer dependency)
 * This is the inverse of getPublicKeyBytesFromPasskeySignature.
 * @param coordinates - Tuple of [x, y] coordinates as hex strings
 * @returns CBOR-encoded COSE public key as Uint8Array
 */
export function getPasskeySignatureFromPublicKeyBytes(coordinates: readonly [Hex, Hex]): Uint8Array {
  const [xHex, yHex] = coordinates;
  const x = hexToBytes(xHex);
  const y = hexToBytes(yHex);

  const cosePublicKey: COSEPublicKeyMap = new Map();
  cosePublicKey.set(COSEKEYS.kty, 2); // Type 2 for EC keys
  cosePublicKey.set(COSEKEYS.alg, -7); // -7 for ES256 algorithm
  cosePublicKey.set(COSEKEYS.crv, 1); // Curve ID (1 for P-256)
  cosePublicKey.set(COSEKEYS.x, x);
  cosePublicKey.set(COSEKEYS.y, y);

  return encodeMap(cosePublicKey);
}

export async function getPasskeyCredential() {
  const credential = await navigator.credentials.get({
    publicKey: {
      challenge: new Uint8Array(32),
      userVerification: "discouraged",
    },
  });
  if (!credential) return null;
  if (credential.type !== "public-key") throw new Error("Invalid credential type");
  const credentialIdHex = bytesToHex(new Uint8Array((credential as PublicKeyCredential).rawId));
  console.log({
    credentialIdHex,
  });
  return {
    credentialIdHex,
    ...credential as PublicKeyCredential,
  };
}
