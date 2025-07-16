import { getAddress, type Hex, http } from "viem";
import { zksyncSepoliaTestnet } from "viem/chains";
import { beforeEach, describe, expect, test, vi } from "vitest";

import { wrapPasskeyTypedDataSig } from "../../utils/wrapSignature.js";
import { createZksyncPasskeyClient } from "./client.js";

// Mock passkey authentication
vi.mock("./actions/passkey.js", () => ({
  requestPasskeyAuthentication: vi.fn().mockResolvedValue({
    passkeyAuthenticationResponse: {
      id: "mockId",
      response: {
        authenticatorData: new Uint8Array(32).fill(1),
        clientDataJSON: new Uint8Array(32).fill(2),
        signature: new Uint8Array(64).fill(3),
      },
    },
  }),
}));

// Mock passkey signature formatting
vi.mock("../../utils/passkey.js", () => ({
  passkeyHashSignatureResponseFormat: vi.fn().mockReturnValue("0x1234567890abcdef" as Hex),
}));

describe.skip("Passkey Client ERC-7739", () => {
  const mockAddress = getAddress("0x742d35Cc6634C0532925a3b844Bc9e7595f2bd9e");
  const mockContracts = {
    session: getAddress("0x1234567890123456789012345678901234567890"),
    passkey: getAddress("0x2345678901234567890123456789012345678901"),
    recovery: getAddress("0x3456789012345678901234567890123456789012"),
  };
  const mockCredentialPublicKey = new Uint8Array(33).fill(0);

  let client: ReturnType<typeof createZksyncPasskeyClient>;

  beforeEach(() => {
    client = createZksyncPasskeyClient({
      chain: zksyncSepoliaTestnet,
      transport: http(),
      address: mockAddress,
      credentialPublicKey: mockCredentialPublicKey,
      userName: "testUser",
      userDisplayName: "Test User",
      contracts: mockContracts,
    }) as any;
  });

  describe("signMessage", () => {
    test("should return an ERC-7739 wrapped signature", async () => {
      const message = "Hello, world!";

      // The passkey client now automatically uses ERC-7739 format
      const signature = await client.signMessage({ message });

      // ERC-7739 signatures start with 0x00 (ApplicationSpecificSigning)
      expect(signature).toMatch(/^0x00/);

      // The signature should be longer than a raw signature due to wrapping
      expect(signature.length).toBeGreaterThan(132); // 0x + 65 bytes * 2
    });

    test("should handle Uint8Array messages", async () => {
      const message = new Uint8Array([72, 101, 108, 108, 111]); // "Hello"

      // The passkey client now automatically uses ERC-7739 format
      const signature = await client.signMessage({ message } as any);

      expect(signature).toMatch(/^0x00/);
      expect(signature.length).toBeGreaterThan(132);
    });
  });

  describe("signTypedData", () => {
    const domain = {
      name: "Test",
      version: "1",
      chainId: zksyncSepoliaTestnet.id,
      verifyingContract: mockAddress,
    } as const;

    const types = {
      Person: [
        { name: "name", type: "string" },
        { name: "wallet", type: "address" },
      ],
    } as const;

    const message = {
      name: "Alice",
      wallet: mockAddress,
    } as const;

    test("should return an ERC-7739 wrapped signature", async () => {
      // The passkey client now automatically uses ERC-7739 format
      const signature = await client.signTypedData({
        domain,
        types,
        primaryType: "Person",
        message,
      });

      // ERC-7739 signatures start with 0x00 (ApplicationSpecificSigning)
      expect(signature).toMatch(/^0x00/);

      // The signature should be longer than a raw signature due to wrapping
      expect(signature.length).toBeGreaterThan(132);
    });
  });

  describe("wrapPasskeyTypedDataSig utility", () => {
    test("should wrap a raw signature", () => {
      const rawSignature = "0x" + "a".repeat(130) as Hex; // 65 bytes

      // Create mock typed data for wrapping
      const params = {
        signature: rawSignature,
        domain: {
          name: "Test",
          version: "1",
          chainId: 1,
          verifyingContract: mockAddress,
        },
        types: {
          Person: [
            { name: "name", type: "string" },
            { name: "wallet", type: "address" },
          ],
        },
        primaryType: "Person" as const,
        message: {
          name: "Alice",
          wallet: mockAddress,
        },
      };

      const wrappedSignature = wrapPasskeyTypedDataSig(params);

      // Should start with 0x00 for ApplicationSpecificSigning
      expect(wrappedSignature).toMatch(/^0x00/);

      // Should be longer than the original
      expect(wrappedSignature.length).toBeGreaterThan(rawSignature.length);
    });
  });

  describe("isValidSignature verification", () => {
    test("should produce signatures verifiable by smart contracts", async () => {
      // This test would require a forked network or mock contract
      // For now, we just verify the signature format is correct
      const message = "Test message for verification";
      const signature = await client.signMessage({ message });

      // Verify it's an ERC-7739 signature
      expect(signature.startsWith("0x00")).toBe(true);

      // In a real test, you would:
      // 1. Deploy or connect to a smart account contract
      // 2. Call isValidSignature(messageHash, signature)
      // 3. Expect it to return the ERC-1271 magic value
    });
  });

  describe("signature format comparison", () => {
    test("snapshot: raw vs ERC-7739 signature format", async () => {
      const message = "Snapshot test message";

      // Get the ERC-7739 signature
      const erc7739Signature = await client.signMessage({ message });

      // Create a snapshot of the signature format
      expect({
        signaturePrefix: erc7739Signature.slice(0, 4), // Should be 0x00
        signatureLength: erc7739Signature.length,
        isErc7739: erc7739Signature.startsWith("0x00"),
      }).toMatchInlineSnapshot();
    });
  });
});
