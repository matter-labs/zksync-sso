import assert from "node:assert/strict";
import { describe, it } from "node:test";

import type { PrividiumSiweChain } from "prividium/siwe";

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
});

describe("addAddressToUser", () => {
  it("merges new addresses with existing wallets (deduplicated)", async () => {
    let updateParams: Parameters<AdminUsersUpdate> | undefined;
    const sdk = makeAdminSdk({
      usersGetById: (async () =>
        ({
          id: "user-1",
          wallets: [
            { id: 1, walletAddress: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", userId: "user-1", createdAt: "", updatedAt: "" },
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
        "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", // duplicate of existing
        "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", // new
      ],
      sdk,
    );

    assert.equal(updateParams?.[0], "user-1");
    assert.deepEqual(updateParams?.[1], {
      wallets: [
        "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
      ],
    });
  });
});
