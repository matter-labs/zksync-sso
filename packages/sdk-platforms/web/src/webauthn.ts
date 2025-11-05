/**
 * WebAuthn Helper Module
 *
 * This module provides helper functions for creating and managing WebAuthn credentials
 * using SimpleWebAuthn library. It extracts the public key coordinates needed for
 * on-chain verification.
 *
 * @module webauthn
 */

// Lazy-loaded SimpleWebAuthn - only loaded when needed
// eslint-disable-next-line @typescript-eslint/no-explicit-any
let simpleWebAuthnModule: any = null;
async function getSimpleWebAuthn() {
  if (!simpleWebAuthnModule) {
    try {
      simpleWebAuthnModule = await import("@simplewebauthn/browser");
    } catch {
      throw new Error(
        "SimpleWebAuthn is not installed. Please install @simplewebauthn/browser: npm install @simplewebauthn/browser",
      );
    }
  }
  return simpleWebAuthnModule;
}

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
 * Convert Uint8Array to base64url encoding (for SimpleWebAuthn)
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
 * Convert base64url string to Uint8Array
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
  // Get SimpleWebAuthn (lazy loaded and cached)
  const simpleWebAuthn = await getSimpleWebAuthn();
  const startRegistration = simpleWebAuthn.startRegistration;

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
  const registrationOptions = {
    challenge: arrayBufferToBase64url(challenge),
    rp: {
      name: options.rpName || "zkSync SSO",
      id: options.rpId || window.location.hostname,
    },
    user: {
      id: arrayBufferToBase64url(userId),
      name: options.userEmail || "user@example.com",
      displayName: options.userName || "Demo User",
    },
    pubKeyCredParams: [
      {
        type: "public-key",
        alg: -7, // ES256 (P-256)
      },
    ],
    authenticatorSelection: {
      authenticatorAttachment: options.authenticatorAttachment || "platform",
      residentKey: "required",
      userVerification: "discouraged",
    },
    timeout: options.timeout || 60000,
    attestation: "none",
  };

  // Create the credential using SimpleWebAuthn
  const credential = await startRegistration(registrationOptions);

  // Parse the authenticator data to extract the public key
  // SimpleWebAuthn returns base64url-encoded strings
  const authenticatorDataBase64url = credential.response.authenticatorData;
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

  // eslint-disable-next-line no-console
  console.log("COSE key bytes:", Array.from(coseKey).map((b) => b.toString(16).padStart(2, "0")).join(" "));

  // Parse COSE key to extract public key coordinates
  const [xBuffer, yBuffer] = parseCOSEKey(coseKey);

  return {
    credentialId: bytesToHex(credId),
    publicKeyX: bytesToHex(xBuffer),
    publicKeyY: bytesToHex(yBuffer),
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
 * @param publicPasskey - CBOR-encoded COSE public key
 * @returns Tuple of [x, y] coordinates as Uint8Arrays
 */
function parseCOSEKey(publicPasskey: Uint8Array): [Uint8Array, Uint8Array] {
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
