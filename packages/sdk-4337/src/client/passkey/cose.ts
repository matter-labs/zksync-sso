import { Buffer } from "buffer";

enum COSEKEYS {
  kty = 1, // Key Type
  alg = 3, // Algorithm
  crv = -1, // Curve for EC keys
  x = -2, // X coordinate for EC keys
  y = -3, // Y coordinate for EC keys
}

type COSEPublicKeyMap = Map<COSEKEYS, number | Buffer>;

function decodeMap(buffer: Buffer): COSEPublicKeyMap {
  const map = new Map<COSEKEYS, number | Buffer>();
  let offset = 1; // Start after the map header

  const mapHeader = buffer[0];
  const mapSize = mapHeader & 0x1f; // Number of pairs

  for (let i = 0; i < mapSize; i++) {
    const [key, keyLength] = decodeInt(buffer, offset);
    offset += keyLength;

    const [value, valueLength] = decodeValue(buffer, offset);
    offset += valueLength;

    map.set(key as COSEKEYS, value);
  }

  return map;
}

function decodeInt(buffer: Buffer, offset: number): [number, number] {
  const intByte = buffer[offset];

  if (intByte < 24) {
    // Small positive integer (0–23)
    return [intByte, 1];
  } else if (intByte === 0x18) {
    // 1-byte unsigned integer
    return [buffer[offset + 1], 2];
  } else if (intByte === 0x19) {
    // 2-byte unsigned integer
    return [buffer.readUInt16BE(offset + 1), 3];
  } else if (intByte >= 0x20 && intByte <= 0x37) {
    // Small negative integer (-1 to -24)
    return [-(intByte - 0x20) - 1, 1];
  } else if (intByte === 0x38) {
    // 1-byte negative integer
    return [-1 - buffer[offset + 1], 2];
  } else if (intByte === 0x39) {
    // 2-byte negative integer
    return [-1 - buffer.readUInt16BE(offset + 1), 3];
  } else {
    throw new Error("Unsupported integer format");
  }
}

function decodeBytes(buffer: Buffer, offset: number): [Buffer, number] {
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

function decodeValue(buffer: Buffer, offset: number): [number | Buffer, number] {
  const type = buffer[offset];
  if (type >= 0x40 && type <= 0x5f) {
    // Byte array
    return decodeBytes(buffer, offset);
  } else {
    return decodeInt(buffer, offset);
  }
}

/**
 * Decodes a CBOR-encoded COSE public key (from WebAuthn credential) and returns the x,y coordinates.
 * @param publicPasskey - CBOR-encoded COSE public key as Uint8Array
 * @returns Tuple of [x, y] coordinates as Buffers
 */
export const getPublicKeyBytesFromPasskeySignature = (publicPasskey: Uint8Array): [Buffer, Buffer] => {
  const cosePublicKey = decodeMap(Buffer.from(publicPasskey)); // Decodes CBOR-encoded COSE key
  const x = cosePublicKey.get(COSEKEYS.x) as Buffer;
  const y = cosePublicKey.get(COSEKEYS.y) as Buffer;

  return [Buffer.from(x), Buffer.from(y)];
};
