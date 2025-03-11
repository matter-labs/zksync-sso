import { describe, expect, test } from "vitest";

import { encodeModuleData, encodePasskeyModuleParameters } from "./encoding";

describe("encoding utils", () => {
  describe("encodePasskeyModuleParameters", () => {
    test("correctly encodes passkey parameters", () => {
      const passkey0 = "1234567890123456789012345678901234567890123456789012345678901234";
      const passkey1 = "abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd";
      const passkey = {
        credentialId: "unique-base64encoded-string",
        passkeyPublicKey: [
          Buffer.from(passkey0, "hex"),
          Buffer.from(passkey1, "hex"),
        ],
        expectedOrigin: "https://example.com",
      };

      const encoded = encodePasskeyModuleParameters(passkey);

      // The encoding should be a hex string
      expect(encoded).toMatch(/^0x[0-9a-f]+$/i);

      // Should contain both public key components and the origin
      expect(encoded).toContain(passkey0);
      expect(encoded).toContain(passkey1);
      expect(encoded).toContain(Buffer.from(passkey.expectedOrigin).toString("hex"));
      expect(encoded).toContain(Buffer.from(passkey.credentialId, "base64url").toString("hex"));
      expect(encoded).toEqual("0x00000000000000000000000000000000000000000000000000000000000000801234567890123456789012345678901234567890123456789012345678901234abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd00000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000014ba78aab9ef9b6ac7bae1e9dca1d79dfacb6b8a78000000000000000000000000000000000000000000000000000000000000000000000000000000000000001368747470733a2f2f6578616d706c652e636f6d00000000000000000000000000");
    });
  });

  describe("encodeModuleData", () => {
    test("correctly encodes module data", () => {
      const moduleData = {
        address: "0x1234567890123456789012345678901234567890" as const,
        parameters: "0xabcdef" as const,
      };

      const encoded = encodeModuleData(moduleData);

      // The encoding should be a hex string
      expect(encoded).toMatch(/^0x[0-9a-f]+$/i);

      // Should contain both the address and parameters
      expect(encoded.toLowerCase()).toContain(moduleData.address.slice(2).toLowerCase());
      expect(encoded.toLowerCase()).toContain(moduleData.parameters.slice(2).toLowerCase());
      expect(encoded).toEqual("0x000000000000000000000000123456789012345678901234567890123456789000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000003abcdef0000000000000000000000000000000000000000000000000000000000");
    });
  });
});
