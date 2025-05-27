import type { Hex } from "viem";
import { describe, expect, test } from "vitest";

import {
  getPasskeySignatureFromPublicKeyBytes,
  getPublicKeyBytesFromPasskeySignature,
} from "./passkey.js";

describe("passkey utils", () => {
  describe("getPublicKeyBytesFromPasskeySignature", () => {
    test("correctly decodes CBOR-encoded COSE key", () => {
      // This is a sample CBOR-encoded COSE key with known x,y coordinates
      // Format: map with 5 entries:
      // 1: 2 (kty: EC2)
      // 3: -7 (alg: ES256)
      // -1: 1 (crv: P-256)
      // -2: x coordinate (32 bytes)
      // -3: y coordinate (32 bytes)
      const samplePublicKey = new Uint8Array([
        0xa5, // map of 5 pairs
        0x01, // key 1 (kty)
        0x02, // value 2 (EC2)
        0x03, // key 3 (alg)
        0x26, // value -7 (ES256)
        0x20, // key -1 (crv)
        0x01, // value 1 (P-256)
        0x21, // key -2 (x coordinate)
        0x58,
        0x20, // bytes(32)
        ...new Uint8Array(32).fill(0x01), // x coordinate filled with 0x01
        0x22, // key -3 (y coordinate)
        0x58,
        0x20, // bytes(32)
        ...new Uint8Array(32).fill(0x02), // y coordinate filled with 0x02
      ]);

      const [x, y] = getPublicKeyBytesFromPasskeySignature(samplePublicKey);

      // Check that x coordinate is all 0x01
      expect(Buffer.from(x).every((byte) => byte === 0x01)).toBe(true);
      // Check that y coordinate is all 0x02
      expect(Buffer.from(y).every((byte) => byte === 0x02)).toBe(true);
      // Check lengths
      expect(x.length).toBe(32);
      expect(y.length).toBe(32);
    });

    test("roundtrip conversion works", () => {
      // Create sample x,y coordinates as hex strings
      const xHex = "0x" + "01".repeat(32) as Hex;
      const yHex = "0x" + "02".repeat(32) as Hex;

      // Convert to COSE format
      const coseKey = getPasskeySignatureFromPublicKeyBytes([xHex, yHex]);

      // Convert back to coordinates
      const [x, y] = getPublicKeyBytesFromPasskeySignature(coseKey);

      // Check that we got back our original values
      expect(Buffer.from(x).toString("hex")).toBe(xHex.slice(2));
      expect(Buffer.from(y).toString("hex")).toBe(yHex.slice(2));
    });

    test("throws on invalid CBOR data", () => {
      const invalidCBOR = new Uint8Array([0xff, 0xff, 0xff]); // Invalid CBOR bytes

      expect(() => {
        getPublicKeyBytesFromPasskeySignature(invalidCBOR);
      }).toThrow();
    });

    test("throws if x or y coordinates are missing", () => {
      // CBOR map with only kty, alg, and crv (missing x,y)
      const incompleteCOSE = new Uint8Array([
        0xa3, // map of 3 pairs
        0x01, // key 1 (kty)
        0x02, // value 2 (EC2)
        0x03, // key 3 (alg)
        0x26, // value -7 (ES256)
        0x20, // key -1 (crv)
        0x01, // value 1 (P-256)
      ]);

      expect(() => {
        getPublicKeyBytesFromPasskeySignature(incompleteCOSE);
      }).toThrow();
    });
  });
});
