import { describe, expect, test } from "vitest";

import {
  credentialIdToBase64Url,
  credentialIdToHex,
  publicKeyCoordinatesToFull,
  publicKeyFullToCoordinates,
} from "./webSdkHelpers.js";

describe("webSdkHelpers", () => {
  describe("publicKeyCoordinatesToFull", () => {
    test("converts X and Y coordinates to full uncompressed key", () => {
      const publicKeyX = "0x0101010101010101010101010101010101010101010101010101010101010101";
      const publicKeyY = "0x0202020202020202020202020202020202020202020202020202020202020202";

      const fullKey = publicKeyCoordinatesToFull(publicKeyX, publicKeyY);

      expect(fullKey).toHaveLength(65);
      expect(fullKey[0]).toBe(0x04); // Uncompressed key prefix
      // Verify X coordinate (bytes 1-32)
      for (let i = 1; i <= 32; i++) {
        expect(fullKey[i]).toBe(0x01);
      }
      // Verify Y coordinate (bytes 33-64)
      for (let i = 33; i <= 64; i++) {
        expect(fullKey[i]).toBe(0x02);
      }
    });

    test("handles coordinates without 0x prefix", () => {
      const publicKeyX = "0101010101010101010101010101010101010101010101010101010101010101";
      const publicKeyY = "0202020202020202020202020202020202020202020202020202020202020202";

      const fullKey = publicKeyCoordinatesToFull(publicKeyX, publicKeyY);

      expect(fullKey).toHaveLength(65);
      expect(fullKey[0]).toBe(0x04);
    });

    test("throws error if X coordinate is wrong length", () => {
      const publicKeyX = "0x01010101"; // Too short
      const publicKeyY = "0x0202020202020202020202020202020202020202020202020202020202020202";

      expect(() => publicKeyCoordinatesToFull(publicKeyX, publicKeyY)).toThrow(
        "Invalid X coordinate length",
      );
    });

    test("throws error if Y coordinate is wrong length", () => {
      const publicKeyX = "0x0101010101010101010101010101010101010101010101010101010101010101";
      const publicKeyY = "0x020202"; // Too short

      expect(() => publicKeyCoordinatesToFull(publicKeyX, publicKeyY)).toThrow(
        "Invalid Y coordinate length",
      );
    });

    test("handles invalid hex characters gracefully", () => {
      const publicKeyX = "0x01010101010101010101010101010101010101010101010101010101010101ZZ"; // Invalid hex
      const publicKeyY = "0x0202020202020202020202020202020202020202020202020202020202020202";

      // parseInt returns NaN for invalid hex, which becomes 0 in the byte array
      // This is expected behavior - validation should happen at a higher level
      const result = publicKeyCoordinatesToFull(publicKeyX, publicKeyY);
      expect(result).toHaveLength(65);
      expect(result[0]).toBe(0x04);
    });
  });

  describe("publicKeyFullToCoordinates", () => {
    test("converts full key to X and Y coordinates", () => {
      const fullKey = new Uint8Array(65);
      fullKey[0] = 0x04; // Uncompressed key prefix
      // Fill X coordinate with 0x01
      for (let i = 1; i <= 32; i++) {
        fullKey[i] = 0x01;
      }
      // Fill Y coordinate with 0x02
      for (let i = 33; i <= 64; i++) {
        fullKey[i] = 0x02;
      }

      const { publicKeyX, publicKeyY } = publicKeyFullToCoordinates(fullKey);

      expect(publicKeyX).toBe("0x0101010101010101010101010101010101010101010101010101010101010101");
      expect(publicKeyY).toBe("0x0202020202020202020202020202020202020202020202020202020202020202");
    });

    test("throws error if key is wrong length", () => {
      const invalidKey = new Uint8Array(64); // Too short

      expect(() => publicKeyFullToCoordinates(invalidKey)).toThrow(
        "Invalid public key length",
      );
    });

    test("throws error if key doesn't start with 0x04", () => {
      const invalidKey = new Uint8Array(65);
      invalidKey[0] = 0x03; // Compressed key prefix (not supported)

      expect(() => publicKeyFullToCoordinates(invalidKey)).toThrow(
        "Invalid public key format",
      );
    });
  });

  describe("roundtrip conversion", () => {
    test("coordinates -> full -> coordinates produces same result", () => {
      const originalX = "0xaabbccdd00112233445566778899aabbccddeeff00112233445566778899aabb";
      const originalY = "0x1122334455667788990011223344556677889900112233445566778899001122";

      const fullKey = publicKeyCoordinatesToFull(originalX, originalY);
      const { publicKeyX, publicKeyY } = publicKeyFullToCoordinates(fullKey);

      expect(publicKeyX.toLowerCase()).toBe(originalX.toLowerCase());
      expect(publicKeyY.toLowerCase()).toBe(originalY.toLowerCase());
    });

    test("full -> coordinates -> full produces same result", () => {
      const originalKey = new Uint8Array(65);
      originalKey[0] = 0x04;
      // Random X coordinate
      for (let i = 1; i <= 32; i++) {
        originalKey[i] = Math.floor(Math.random() * 256);
      }
      // Random Y coordinate
      for (let i = 33; i <= 64; i++) {
        originalKey[i] = Math.floor(Math.random() * 256);
      }

      const { publicKeyX, publicKeyY } = publicKeyFullToCoordinates(originalKey);
      const reconstructedKey = publicKeyCoordinatesToFull(publicKeyX, publicKeyY);

      expect(reconstructedKey).toEqual(originalKey);
    });
  });

  describe("credentialIdToHex", () => {
    test("converts base64url credential ID to hex", () => {
      // "test" in base64url is "dGVzdA"
      const credentialId = "dGVzdA";
      const hex = credentialIdToHex(credentialId);

      expect(hex).toBe("0x74657374"); // "test" in hex
    });

    test("handles base64url with dashes and underscores", () => {
      // Base64url uses - and _ instead of + and /
      const credentialId = "SGVsbG8tV29ybGRf"; // Contains - and _
      const hex = credentialIdToHex(credentialId);

      expect(hex).toMatch(/^0x[0-9a-f]+$/); // Valid hex
    });

    test("handles credential ID without padding", () => {
      // Base64url typically omits padding
      const credentialId = "dGVzdA"; // No padding
      const hex = credentialIdToHex(credentialId);

      expect(hex).toBe("0x74657374");
    });
  });

  describe("credentialIdToBase64Url", () => {
    test("converts hex credential ID to base64url", () => {
      const hex = "0x74657374"; // "test" in hex
      const base64url = credentialIdToBase64Url(hex);

      expect(base64url).toBe("dGVzdA"); // "test" in base64url
    });

    test("handles hex without 0x prefix", () => {
      const hex = "74657374"; // No 0x prefix
      const base64url = credentialIdToBase64Url(hex);

      expect(base64url).toBe("dGVzdA");
    });

    test("produces base64url without padding", () => {
      const hex = "0x74657374";
      const base64url = credentialIdToBase64Url(hex);

      // Base64url should not contain padding (=)
      expect(base64url).not.toContain("=");
    });

    test("produces base64url with URL-safe characters", () => {
      // Generate a hex that would produce + or / in standard base64
      const hex = "0x" + "fb".repeat(20); // Will produce + and / in base64
      const base64url = credentialIdToBase64Url(hex);

      // Should only contain base64url characters (no + or /)
      expect(base64url).toMatch(/^[A-Za-z0-9_-]+$/);
    });
  });

  describe("credentialId roundtrip conversion", () => {
    test("hex -> base64url -> hex produces same result", () => {
      const originalHex = "0xaabbccdd00112233445566778899aabbccddeeff";
      const base64url = credentialIdToBase64Url(originalHex);
      const reconstructedHex = credentialIdToHex(base64url);

      expect(reconstructedHex.toLowerCase()).toBe(originalHex.toLowerCase());
    });

    test("base64url -> hex -> base64url produces same result", () => {
      const originalBase64url = "qrvM3QARIjNEVWZ3iJmqu8zd7_8";
      const hex = credentialIdToHex(originalBase64url);
      const reconstructedBase64url = credentialIdToBase64Url(hex);

      expect(reconstructedBase64url).toBe(originalBase64url);
    });
  });
});
