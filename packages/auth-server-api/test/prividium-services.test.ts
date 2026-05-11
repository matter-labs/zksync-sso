import assert from "node:assert/strict";
import { afterEach, describe, it } from "node:test";

import { authenticatedFetch, type PrividiumApiAuth } from "../src/services/prividium/authenticated-fetch.ts";
import { whitelistContract } from "../src/services/prividium/contract-whitelist.ts";

const originalFetch = globalThis.fetch;

afterEach(() => {
  globalThis.fetch = originalFetch;
});

describe("authenticatedFetch", () => {
  it("uses cached auth headers without reauthorizing", async () => {
    const auth: PrividiumApiAuth = {
      getAuthHeaders: () => ({ Authorization: "Bearer cached-token" }),
      authorize: async () => {
        throw new Error("authorize should not be called");
      },
    };

    globalThis.fetch = (async (_url, init) => {
      assert.equal(new Headers(init?.headers).get("Authorization"), "Bearer cached-token");
      return new Response("{}", { status: 200 });
    }) as typeof fetch;

    const response = await authenticatedFetch(auth, "https://api.example.com/api/contracts");

    assert.equal(response.status, 200);
  });

  it("reauthorizes and retries once after a 401 response", async () => {
    let token = "old-token";
    let authorizeCalls = 0;
    const seenTokens: string[] = [];
    const auth: PrividiumApiAuth = {
      getAuthHeaders: () => ({ Authorization: `Bearer ${token}` }),
      authorize: async () => {
        authorizeCalls++;
        token = "new-token";
      },
    };

    globalThis.fetch = (async (_url, init) => {
      seenTokens.push(new Headers(init?.headers).get("Authorization") ?? "");
      if (seenTokens.length === 1) {
        return new Response("{}", { status: 401 });
      }
      return new Response("{}", { status: 200 });
    }) as typeof fetch;

    const response = await authenticatedFetch(auth, "https://api.example.com/api/contracts");

    assert.equal(response.status, 200);
    assert.equal(authorizeCalls, 1);
    assert.deepEqual(seenTokens, ["Bearer old-token", "Bearer new-token"]);
  });
});

describe("whitelistContract", () => {
  it("sends Prividium contract disclosure fields expected by the API", async () => {
    let requestBody: unknown;
    const auth: PrividiumApiAuth = {
      getAuthHeaders: () => ({ Authorization: "Bearer token" }),
      authorize: async () => {},
    };

    globalThis.fetch = (async (_url, init) => {
      requestBody = JSON.parse(init?.body as string);
      return new Response("{}", { status: 201 });
    }) as typeof fetch;

    await whitelistContract(
      "0x1234567890123456789012345678901234567890",
      "sso-account",
      auth,
      "https://api.example.com",
    );

    assert.deepEqual(requestBody, {
      contractAddress: "0x1234567890123456789012345678901234567890",
      templateKey: "sso-account",
      abi: "[]",
      name: null,
      description: null,
      discloseErc20TotalSupply: false,
      discloseBytecode: false,
    });
  });
});
