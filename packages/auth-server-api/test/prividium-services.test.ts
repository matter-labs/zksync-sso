import assert from "node:assert/strict";
import { describe, it } from "node:test";

import type { PrividiumSiweChain } from "prividium/siwe";
import { getAddress } from "viem";

import { addAddressToUser } from "../src/services/prividium/address-association.ts";
import { whitelistContract } from "../src/services/prividium/contract-whitelist.ts";

type AdminContractsCreate = PrividiumSiweChain["admin"]["contracts"]["create"];
type AdminUsersGetById = PrividiumSiweChain["admin"]["users"]["getById"];
type AdminUsersUpdate = PrividiumSiweChain["admin"]["users"]["update"];

function makeAdminSdk(overrides: {
  contractsCreate?: AdminContractsCreate;
  usersGetById?: AdminUsersGetById;
  usersUpdate?: AdminUsersUpdate;
}): PrividiumSiweChain {
  return {
    admin: {
      contracts: {
        create:
          overrides.contractsCreate
          ?? (async () => {
            throw new Error("contracts.create not stubbed");
          }),
      },
      users: {
        getById:
          overrides.usersGetById
          ?? (async () => {
            throw new Error("users.getById not stubbed");
          }),
        update:
          overrides.usersUpdate
          ?? (async () => {
            throw new Error("users.update not stubbed");
          }),
      },
    },
  } as unknown as PrividiumSiweChain;
}

describe("whitelistContract", () => {
  it("sends Prividium contract disclosure fields expected by the API", async () => {
    let createParams: Parameters<AdminContractsCreate>[0] | undefined;
    const sdk = makeAdminSdk({
      contractsCreate: (async (params) => {
        createParams = params;
        return {} as Awaited<ReturnType<AdminContractsCreate>>;
      }) as AdminContractsCreate,
    });

    await whitelistContract(
      "0x1234567890123456789012345678901234567890",
      "sso-account",
      sdk,
    );

    assert.deepEqual(createParams, {
      contractAddress: "0x1234567890123456789012345678901234567890",
      templateKey: "sso-account",
      abi: "[]",
      name: null,
      description: null,
      discloseErc20TotalSupply: false,
      discloseBytecode: false,
      disclosureStartBlock: "0x0",
    });
  });

  it("propagates errors from sdk.admin.contracts.create", async () => {
    const sdk = makeAdminSdk({
      contractsCreate: (async () => {
        throw new Error("Error calling /api/contracts: 400 VALIDATION_ERROR");
      }) as AdminContractsCreate,
    });

    await assert.rejects(
      () =>
        whitelistContract(
          "0x1234567890123456789012345678901234567890",
          "sso-account",
          sdk,
        ),
      /Error calling \/api\/contracts: 400 VALIDATION_ERROR/,
    );
  });
});

describe("addAddressToUser", () => {
  it("normalises existing + incoming addresses to checksum and de-duplicates across casings", async () => {
    // Realistic case: the SDK returns a checksummed wallet (server stores via
    // EIP-55 normalisation), and the deploy handler passes the same address
    // lowercase (it comes from a viem log topic). The Set-based merge must
    // not see these as two distinct entries.
    const lowercaseA = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" as const;
    const lowercaseB = "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb" as const;
    const checksumA = getAddress(lowercaseA);
    const checksumB = getAddress(lowercaseB);

    let updateParams: Parameters<AdminUsersUpdate> | undefined;
    const sdk = makeAdminSdk({
      usersGetById: (async () =>
        ({
          id: "user-1",
          wallets: [
            // Existing wallet returned by the server in checksum form
            { id: 1, walletAddress: checksumA, userId: "user-1", createdAt: "", updatedAt: "" },
          ],
        }) as Awaited<ReturnType<AdminUsersGetById>>) as AdminUsersGetById,
      usersUpdate: (async (...args) => {
        updateParams = args;
        return {} as Awaited<ReturnType<AdminUsersUpdate>>;
      }) as AdminUsersUpdate,
    });

    await addAddressToUser(
      "user-1",
      [
        lowercaseA, // same as existing but different case — must dedup
        lowercaseB, // new
      ],
      sdk,
    );

    assert.equal(updateParams?.[0], "user-1");
    assert.deepEqual(updateParams?.[1], {
      wallets: [checksumA, checksumB],
    });
  });

  it("propagates errors from sdk.admin.users.getById and does not call update", async () => {
    let updateCalled = false;
    const sdk = makeAdminSdk({
      usersGetById: (async () => {
        throw new Error("Error calling /api/users/user-1: 404 not found");
      }) as AdminUsersGetById,
      usersUpdate: (async () => {
        updateCalled = true;
        return {} as Awaited<ReturnType<AdminUsersUpdate>>;
      }) as AdminUsersUpdate,
    });

    await assert.rejects(
      () => addAddressToUser("user-1", ["0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"], sdk),
      /Error calling \/api\/users\/user-1: 404 not found/,
    );
    assert.equal(updateCalled, false);
  });
});
