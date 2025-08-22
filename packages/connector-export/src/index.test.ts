import { describe, expect, it } from "vitest";

import { eraMainnet, eraSepolia, eraTestNode, zksyncSsoConnector } from "./index.js";

describe("zksync-sso-connector-export", () => {
  it("should export eraTestNode with correct configuration", () => {
    expect(eraTestNode).toBeDefined();
    expect(eraTestNode.id).toBe(260);
    expect(eraTestNode.name).toBe("ZKsync InMemory Node");
    expect(eraTestNode.testnet).toBe(true);
    expect(eraTestNode.rpcUrls.default.http).toEqual(["http://localhost:8011"]);
  });

  it("should export eraSepolia with correct configuration", () => {
    expect(eraSepolia).toBeDefined();
    expect(eraSepolia.id).toBe(300);
    expect(eraSepolia.name).toBe("ZKsync Sepolia Testnet");
    expect(eraSepolia.testnet).toBe(true);
    expect(eraSepolia.rpcUrls.default.http).toEqual(["https://sepolia.era.zksync.dev"]);
  });

  it("should export eraMainnet with correct configuration", () => {
    expect(eraMainnet).toBeDefined();
    expect(eraMainnet.id).toBe(324);
    expect(eraMainnet.name).toBe("ZKsync Era");
    expect(eraMainnet.testnet).toBe(undefined); // wagmi chains don't have testnet: false, just omit it
    expect(eraMainnet.rpcUrls.default.http).toEqual(["https://mainnet.era.zksync.io"]);
  });

  it("should export zksyncSsoConnector function", () => {
    expect(zksyncSsoConnector).toBeDefined();
    expect(typeof zksyncSsoConnector).toBe("function");
  });

  it("should create connector with default options", () => {
    const connector = zksyncSsoConnector();
    expect(connector).toBeDefined();
    expect(typeof connector).toBe("function");
  });

  it("should create connector with custom options", () => {
    const connector = zksyncSsoConnector({
      authServerUrl: "https://custom.auth.server",
    });
    expect(connector).toBeDefined();
    expect(typeof connector).toBe("function");
  });

  it("should have proper chain explorers", () => {
    expect(eraSepolia.blockExplorers?.default.url).toBe("https://sepolia-era.zksync.network/");
    expect(eraMainnet.blockExplorers?.default.url).toBe("https://era.zksync.network/");
  });
});
