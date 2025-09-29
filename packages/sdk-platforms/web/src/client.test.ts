// imports
import { describe, expect, it } from "vitest";

import { ZkSyncSsoClient, ZkSyncSsoUtils } from "./client";

describe("ZkSyncSsoUtils", () => {
  it("should validate addresses correctly", () => {
    expect(ZkSyncSsoUtils.isValidAddress("0x1234567890123456789012345678901234567890")).toBe(true);
    expect(ZkSyncSsoUtils.isValidAddress("invalid")).toBe(false);
  });

  it("should validate private keys correctly", () => {
    expect(ZkSyncSsoUtils.isValidPrivateKey("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef")).toBe(true);
    expect(ZkSyncSsoUtils.isValidPrivateKey("invalid")).toBe(false);
  });

  it("should convert bytes to hex", () => {
    const bytes = new Uint8Array([1, 2, 3, 255]);
    expect(ZkSyncSsoUtils.bytesToHex(bytes)).toBe("0x010203ff");
  });

  it("should convert hex to bytes", () => {
    const hex = "0x010203ff";
    const bytes = ZkSyncSsoUtils.hexToBytes(hex);
    expect(bytes).toEqual(new Uint8Array([1, 2, 3, 255]));
  });
});

describe("ZkSyncSsoClient", () => {
  const config = {
    rpcUrl: "https://sepolia.era.zksync.dev",
    bundlerUrl: "https://bundler.example.com",
    contracts: {
      entryPoint: "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789",
      accountFactory: "0x9406Cc6185a346906296840746125a0E44976454",
    },
  };

  const privateKey = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

  it("should create client instance", () => {
    expect(() => new ZkSyncSsoClient(config, privateKey)).not.toThrow();
  });
});
