import { type Account, type Address, type Chain, type Client, type Hash, type TransactionReceipt, type Transport } from "viem";
import { waitForTransactionReceipt } from "viem/actions";
import { beforeEach, describe, expect, test, vi } from "vitest";

import { addOidcAccount } from "./oidc.js";

// Mocks
vi.mock("viem/actions", () => ({
  waitForTransactionReceipt: vi.fn(),
}));
vi.mock("viem/zksync", () => ({
  sendTransaction: vi.fn(),
}));
vi.mock("../../../abi/index.js", async () => {
  const actual = await import("../../../abi/index.js");
  return { ...actual };
});

// Helpers
// Minimal viem client mock fulfilling required shape for our function
const mockAccount: Account = {
  address: "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266" as Address,
  type: "json-rpc",
};
const mockClient = {
  account: mockAccount,
  chain: { id: 1 } as Chain,
} as unknown as Client<Transport, Chain, Account>; // Casting acceptable in test context

const recoveryOidc = "0x1234567890123456789012345678901234567890" as Address;
const oidcDigest = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" as Hash;
const iss = "https://accounts.example.com";

const successReceipt: TransactionReceipt = {
  status: "success",
  blockNumber: 1n,
  blockHash: "0x1" as Hash,
  contractAddress: null,
  cumulativeGasUsed: 0n,
  effectiveGasPrice: 0n,
  from: mockAccount.address,
  gasUsed: 0n,
  logs: [],
  logsBloom: "0x",
  to: recoveryOidc,
  transactionHash: "0xdeadbeef" as Hash,
  transactionIndex: 0,
  type: "eip712",
};

const { sendTransaction } = await import("viem/zksync");

describe("addOidcAccount", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(waitForTransactionReceipt).mockResolvedValue(successReceipt);
  });

  test("sends once without paymaster", async () => {
    vi.mocked(sendTransaction).mockResolvedValue("0xdeadbeef" as Hash);
    await addOidcAccount(mockClient, {
      contracts: { recoveryOidc },
      oidcDigest,
      iss,
    });
    expect(sendTransaction).toHaveBeenCalledTimes(1);
    expect(sendTransaction).toHaveBeenCalledWith(
      mockClient,
      expect.objectContaining({ to: recoveryOidc, paymaster: undefined }),
    );
  });

  test("attempts paymaster then falls back when first send throws", async () => {
    const error = new Error("Paymaster failure");
    vi.mocked(sendTransaction)
      .mockRejectedValueOnce(error) // first attempt with paymaster
      .mockResolvedValueOnce("0xdeadbeef" as Hash); // fallback without paymaster

    await addOidcAccount(mockClient, {
      contracts: { recoveryOidc },
      oidcDigest,
      iss,
      paymaster: { address: "0x9999999999999999999999999999999999999999" as Address },
    });

    // First call has paymaster
    const firstCallArgs = vi.mocked(sendTransaction).mock.calls[0];
    expect(firstCallArgs[1]).toMatchObject({ paymaster: "0x9999999999999999999999999999999999999999" });
    // Second call (fallback) has no paymaster
    const secondCallArgs = vi.mocked(sendTransaction).mock.calls[1];
    expect(secondCallArgs[1]).not.toHaveProperty("paymaster");
    expect(sendTransaction).toHaveBeenCalledTimes(2);
  });

  test("succeeds on first attempt with paymaster", async () => {
    vi.mocked(sendTransaction).mockResolvedValue("0xdeadbeef" as Hash);

    await addOidcAccount(mockClient, {
      contracts: { recoveryOidc },
      oidcDigest,
      iss,
      paymaster: { address: "0x9999999999999999999999999999999999999999" as Address },
    });

    expect(sendTransaction).toHaveBeenCalledTimes(1);
    const callArgs = vi.mocked(sendTransaction).mock.calls[0];
    expect(callArgs[1]).toMatchObject({ paymaster: "0x9999999999999999999999999999999999999999" });
  });
});
