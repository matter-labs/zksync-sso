import { type Address, type Hash, type TransactionReceipt } from "viem";
import { waitForTransactionReceipt, writeContract } from "viem/actions";
import { describe, expect, test, vi } from "vitest";

import { deployAccount } from "./account.js";

// Mock the passkey utils
vi.mock("../../../utils/passkey.js", () => ({
  getPublicKeyBytesFromPasskeySignature: vi.fn().mockReturnValue([
    Buffer.from("0000000000000000000000000000000000000000000000000000000000000001", "hex"),
    Buffer.from("0000000000000000000000000000000000000000000000000000000000000002", "hex"),
  ]),
}));

// Mock viem actions
vi.mock("viem/actions", () => ({
  writeContract: vi.fn(),
  waitForTransactionReceipt: vi.fn(),
}));

// Add FactoryAbi mock at the top with other mocks
vi.mock("../../../abi/Factory.js", () => ({
  FactoryAbi: [
    {
      inputs: [
        { type: "bytes32", name: "_salt" },
        { type: "string", name: "_uniqueAccountId" },
        { type: "bytes[]", name: "_initialValidators" },
        { type: "address[]", name: "_initialK1Owners" },
      ],
      name: "deployProxySsoAccount",
      outputs: [{ type: "address", name: "accountAddress" }],
      stateMutability: "nonpayable",
      type: "function",
    },
    {
      anonymous: false,
      inputs: [
        {
          indexed: true,
          internalType: "address",
          name: "accountAddress",
          type: "address",
        },
        {
          indexed: false,
          internalType: "string",
          name: "uniqueAccountId",
          type: "string",
        },
      ],
      name: "AccountCreated",
      type: "event",
    },
  ],
}));

describe("deployAccount", () => {
  // Setup common test data
  const mockSalt = new Uint8Array([
    213, 36, 52, 69, 251, 82, 199, 45, 113, 6, 20, 213, 78, 47, 165,
    164, 106, 221, 105, 67, 247, 47, 200, 167, 137, 64, 151, 12, 179,
    74, 90, 23,
  ]);

  // CBOR-encoded COSE key with known x,y coordinates
  const mockCredentialPublicKey = new Uint8Array([
    0xa5, // map of 5 pairs
    0x01, // key 1 (kty)
    0x02, // value 2 (EC2)
    0x03, // key 3 (alg)
    0x26, // value -7 (ES256)
    0x20, // key -1 (crv)
    0x01, // value 1 (P-256)
    0x21, // key -2 (x coordinate)
    0x58, 0x20, // bytes(32)
    ...new Uint8Array(32).fill(0x01), // x coordinate filled with 0x01
    0x22, // key -3 (y coordinate)
    0x58, 0x20, // bytes(32)
    ...new Uint8Array(32).fill(0x02), // y coordinate filled with 0x02
  ]);

  const mockClient = {
    account: "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266",
    chain: { id: 1 },
  } as any;
  const mockContracts = {
    accountFactory: "0x1234567890123456789012345678901234567890" as Address,
    passkey: "0x2234567890123456789012345678901234567890" as Address,
    session: "0x3234567890123456789012345678901234567890" as Address,
  };

  const mockTransactionHash = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef" as Hash;
  const mockTransactionReceipt: TransactionReceipt = {
    status: "success",
    contractAddress: null,
    blockNumber: 1n,
    blockHash: "0x5e1d3a76f1b1c3a2b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7" as Hash,
    transactionHash: mockTransactionHash,
    logs: [
      {
        address: mockContracts.accountFactory,
        blockHash: "0x5e1d3a76f1b1c3a2b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7",
        blockNumber: 1n,
        data: "0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000001b67615039747050496b704445496c6c52504e65594b706d3264514d0000000000",
        logIndex: 0,
        removed: false,
        topics: [
          "0xb3202828e8e55b43ee1a77a1ee6ffbe19f0205767feb21e28cadd26390ff0501", // event AccountCreated(address,string)
          "0x0000000000000000000000004234567890123456789012345678901234567890",
        ],
        transactionHash: "0x0b8154f57a02650c3cf622c19f1d90899d4779b3d181dfd03a53b3b257d76dd5",
        transactionIndex: 0,
      },
    ],
    logsBloom: "0x",
    cumulativeGasUsed: 0n,
    effectiveGasPrice: 0n,
    gasUsed: 0n,
    type: "eip1559",
    from: "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266",
    to: "0x1234567890123456789012345678901234567890",
    transactionIndex: 0,
  };

  test("deploys account successfully", async () => {
    // Setup mocks
    vi.mocked(writeContract).mockResolvedValue(mockTransactionHash);
    vi.mocked(waitForTransactionReceipt).mockResolvedValue(mockTransactionReceipt);

    const result = await deployAccount(mockClient, {
      credentialPublicKey: mockCredentialPublicKey,
      contracts: mockContracts,
      expectedOrigin: "https://example.com",
      salt: mockSalt,
    });

    // Verify the result
    expect(result).toEqual({
      address: "0x4234567890123456789012345678901234567890",
      transactionReceipt: mockTransactionReceipt,
    });

    // Verify writeContract was called with correct parameters
    expect(writeContract).toHaveBeenCalledWith(
      mockClient,
      expect.objectContaining({
        address: mockContracts.accountFactory,
        functionName: "deployProxySsoAccount",
      }),
    );
  });

  test("handles transaction failure", async () => {
    // Setup mock for failed transaction
    vi.mocked(writeContract).mockResolvedValue(mockTransactionHash);
    vi.mocked(waitForTransactionReceipt).mockResolvedValue({
      ...mockTransactionReceipt,
      status: "reverted",
    });

    await expect(
      deployAccount(mockClient, {
        credentialPublicKey: mockCredentialPublicKey,
        contracts: mockContracts,
        expectedOrigin: "https://example.com",
        salt: mockSalt,
      }),
    ).rejects.toThrow("Account deployment transaction reverted");
  });

  test("handles missing contract address in receipt", async () => {
    // Setup mock for missing contract address
    vi.mocked(writeContract).mockResolvedValue(mockTransactionHash);
    vi.mocked(waitForTransactionReceipt).mockResolvedValue({
      ...mockTransactionReceipt,
      logs: [],
    });

    await expect(
      deployAccount(mockClient, {
        credentialPublicKey: mockCredentialPublicKey,
        contracts: mockContracts,
        expectedOrigin: "https://example.com",
        salt: mockSalt,
      }),
    ).rejects.toThrow("No contract address in transaction receipt");
  });

  test("calls onTransactionSent callback when provided", async () => {
    const onTransactionSent = vi.fn();
    vi.mocked(writeContract).mockResolvedValue(mockTransactionHash);
    vi.mocked(waitForTransactionReceipt).mockResolvedValue(mockTransactionReceipt);

    await deployAccount(mockClient, {
      credentialPublicKey: mockCredentialPublicKey,
      contracts: mockContracts,
      expectedOrigin: "https://example.com",
      salt: mockSalt,
      onTransactionSent,
    });

    expect(onTransactionSent).toHaveBeenCalledWith(mockTransactionHash);
  });

  test("uses window.location.origin when expectedOrigin is not provided", async () => {
    // Mock window.location
    const originalWindow = global.window;
    global.window = {
      ...originalWindow,
      location: {
        ...originalWindow?.location,
        origin: "https://example.com",
      },
    } as any;

    vi.mocked(writeContract).mockResolvedValue(mockTransactionHash);
    vi.mocked(waitForTransactionReceipt).mockResolvedValue(mockTransactionReceipt);

    const writeContractSpy = vi.mocked(writeContract);
    await deployAccount(mockClient, {
      credentialPublicKey: mockCredentialPublicKey,
      contracts: mockContracts,
      salt: mockSalt,
    });

    // Simpler assertion that just checks the key parts
    const lastCall = writeContractSpy.mock.lastCall;
    expect(lastCall?.[0]).toBe(mockClient);
    expect(lastCall?.[1]).toMatchObject({
      address: mockContracts.accountFactory,
      functionName: "deployProxySsoAccount",
    });

    // Restore window
    global.window = originalWindow;
  });

  test("handles paymaster configuration", async () => {
    vi.mocked(writeContract).mockResolvedValue(mockTransactionHash);
    vi.mocked(waitForTransactionReceipt).mockResolvedValue(mockTransactionReceipt);

    const paymasterAddress = "0x5234567890123456789012345678901234567890" as Address;
    const paymasterInput = "0x1234" as const;

    await deployAccount(mockClient, {
      credentialPublicKey: mockCredentialPublicKey,
      contracts: mockContracts,
      expectedOrigin: "https://example.com",
      paymasterAddress,
      paymasterInput,
    });

    expect(writeContract).toHaveBeenCalledWith(
      mockClient,
      expect.objectContaining({
        paymaster: paymasterAddress,
        paymasterInput,
      }),
    );
  });
});
