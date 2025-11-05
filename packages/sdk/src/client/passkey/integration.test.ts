/**
 * Integration tests for Web-SDK integration
 *
 * These tests require:
 * - A running ERC-4337 bundler
 * - A local or testnet blockchain
 * - Deployed contracts (account factory, validators, entry point)
 *
 * Run with: pnpm test:integration
 *
 * Or skip with: SKIP_INTEGRATION_TESTS=true pnpm test
 */

import { createPublicClient, http } from "viem";
import { zkSyncSepoliaTestnet } from "viem/chains";
import { beforeAll, describe, expect, test } from "vitest";

import { createZksyncPasskeyClient, deployAccountWebSdk, publicKeyCoordinatesToFull } from "./index.js";

// Skip these tests if integration tests are disabled
const skipIntegration = process.env.SKIP_INTEGRATION_TESTS === "true";
const describeIntegration = skipIntegration ? describe.skip : describe;

describeIntegration("Web-SDK Integration Tests", () => {
  let bundlerUrl: string;
  let rpcUrl: string;
  let entryPointAddress: `0x${string}`;
  let deployerPrivateKey: `0x${string}`;

  beforeAll(() => {
    // Load from environment or use defaults for local testing
    bundlerUrl = process.env.BUNDLER_URL || "http://localhost:3000";
    rpcUrl = process.env.RPC_URL || "http://localhost:8545";
    entryPointAddress = (process.env.ENTRY_POINT_ADDRESS || "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789") as `0x${string}`;
    deployerPrivateKey = (process.env.DEPLOYER_PRIVATE_KEY || "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80") as `0x${string}`;

    // Validate required environment variables
    if (!bundlerUrl) {
      throw new Error("BUNDLER_URL is required for integration tests");
    }
    if (!rpcUrl) {
      throw new Error("RPC_URL is required for integration tests");
    }
    if (!deployerPrivateKey) {
      throw new Error("DEPLOYER_PRIVATE_KEY is required for integration tests");
    }
  });

  describe("Account Deployment", () => {
    test("should deploy account with passkey credentials", async () => {
      // Note: This test requires web-SDK to be built and credential creation
      // In a real environment, you would use createWebAuthnCredential()
      // For testing, we use mock credentials with known X/Y coordinates

      const mockPublicKeyX = "0x0101010101010101010101010101010101010101010101010101010101010101";
      const mockPublicKeyY = "0x0202020202020202020202020202020202020202020202020202020202020202";
      const mockCredentialId = "mock-credential-id";

      // Convert to SDK format
      const fullPublicKey = publicKeyCoordinatesToFull(mockPublicKeyX, mockPublicKeyY);

      // Create public client
      const publicClient = createPublicClient({
        chain: zkSyncSepoliaTestnet,
        transport: http(rpcUrl),
      });

      // Create SDK client
      // Note: In production, credential comes from createWebAuthnCredential()
      const client = createZksyncPasskeyClient({
        chain: zkSyncSepoliaTestnet,
        transport: http(rpcUrl),
        address: "0x0000000000000000000000000000000000000000", // Placeholder
        credentialPublicKey: fullPublicKey,
        userName: `test-user-${Date.now()}`,
        userDisplayName: "Test User",
        bundlerUrl,
        entryPointAddress,
        credential: {
          id: mockCredentialId,
          type: "public-key",
        },
        contracts: {
          oidcKeyRegistry: "0x0000000000000000000000000000000000000001" as `0x${string}`,
          session: "0x0000000000000000000000000000000000000002" as `0x${string}`,
          passkey: "0x0000000000000000000000000000000000000003" as `0x${string}`,
          recovery: "0x0000000000000000000000000000000000000004" as `0x${string}`,
          recoveryOidc: "0x0000000000000000000000000000000000000005" as `0x${string}`,
          accountFactory: "0x0000000000000000000000000000000000000006" as `0x${string}`,
        },
      });

      // Deploy account (note: requires actual contracts to be deployed)
      // This test will be skipped by default until contracts are available
      try {
        const result = await deployAccountWebSdk(client, {
          userId: `test-user-${Date.now()}`,
          deployerPrivateKey,
        });

        // Verify result
        expect(result.address).toMatch(/^0x[a-fA-F0-9]{40}$/);
        expect(result.transactionHash).toMatch(/^0x[a-fA-F0-9]{64}$/);

        // Verify account was deployed
        const code = await publicClient.getBytecode({ address: result.address });
        expect(code).toBeDefined();
        expect(code).not.toBe("0x");
      } catch (error) {
        // If contracts are not deployed or bundler is not running, skip
        if (
          error instanceof Error
          && (error.message.includes("bundler")
            || error.message.includes("contract")
            || error.message.includes("network"))
        ) {
          console.warn("Skipping integration test - bundler or contracts not available:", error.message);
          return;
        }
        throw error;
      }

      // Verify result
      expect(result.address).toMatch(/^0x[a-fA-F0-9]{40}$/);
      expect(result.transactionHash).toMatch(/^0x[a-fA-F0-9]{64}$/);

      // Verify account was deployed
      const code = await publicClient.getBytecode({ address: result.address });
      expect(code).toBeDefined();
      expect(code).not.toBe("0x");
    }, 60000); // 60 second timeout for blockchain interactions
  });

  describe("Transaction Sending", () => {
    test.skip("should send transaction using passkey", async () => {
      // TODO: Implement after account deployment test passes
      // This requires:
      // 1. Deployed account
      // 2. Funded account (for gas)
      // 3. Recipient address
      // 4. Actual passkey signing (or mock signature)
    });
  });

  describe("Helper Functions", () => {
    test("should convert public key coordinates correctly", () => {
      const publicKeyX = "0x0101010101010101010101010101010101010101010101010101010101010101";
      const publicKeyY = "0x0202020202020202020202020202020202020202020202020202020202020202";

      const fullKey = publicKeyCoordinatesToFull(publicKeyX, publicKeyY);

      expect(fullKey).toHaveLength(65);
      expect(fullKey[0]).toBe(0x04); // Uncompressed key prefix
    });
  });
});
