import { describe, expect, test } from "vitest";

import { encodeModuleData, encodePasskeyModuleParameters } from "./encoding";

describe("encoding utils", () => {
  describe("encodePasskeyModuleParameters", () => {
    test("correctly encodes passkey parameters", () => {
      const passkey = {
        passkeyPublicKey: [
          Buffer.from("1234567890123456789012345678901234567890123456789012345678901234", "hex"),
          Buffer.from("abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd", "hex"),
        ],
        expectedOrigin: "https://example.com",
      };

      const encoded = encodePasskeyModuleParameters(passkey);

      // The encoding should be a hex string
      expect(encoded).toMatch(/^0x[0-9a-f]+$/i);

      // Should contain both public key components and the origin
      expect(encoded).toContain("1234567890123456789012345678901234567890123456789012345678901234");
      expect(encoded).toContain("abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd");
      expect(encoded).toContain(Buffer.from("https://example.com").toString("hex"));
      expect(encoded).toEqual("0x1234567890123456789012345678901234567890123456789012345678901234abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd0000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000001368747470733a2f2f6578616d706c652e636f6d00000000000000000000000000");
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
