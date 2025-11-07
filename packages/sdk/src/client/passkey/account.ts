import type { Address } from "abitype";
import { type Chain, concat, encodeAbiParameters, type Hash, hashMessage, type Hex, pad, type Transport } from "viem";
import { type SmartAccount, toSmartAccount } from "viem/account-abstraction";

import type { PasskeyRequiredContracts } from "./client.js";

// ABI for encoding multiple calls
const callAbi = [
  {
    type: "tuple[]",
    components: [
      { name: "to", type: "address" },
      { name: "value", type: "uint256" },
      { name: "data", type: "bytes" },
    ],
  },
] as const;

export type ToPasskeyAccountParameters<
  transport extends Transport = Transport,
  chain extends Chain = Chain,
> = {
  /** Address of the deployed Account's Contract implementation. */
  address: Address;
  sign: (parameters: { hash: Hash }) => Promise<Hex>;
  chain: NonNullable<chain>;
  transport: transport;
  contracts: PasskeyRequiredContracts;
  credentialId: Hex;
  rpId?: string;
  origin?: string;
  entryPoint: {
    address: Address;
    abi: any;
    version: "0.7" | "0.8";
  };
};

export type PasskeyAccount = SmartAccount;

export async function toPasskeyAccount<
  transport extends Transport = Transport,
  chain extends Chain = Chain,
>(
  parameters: ToPasskeyAccountParameters<transport, chain>,
): Promise<PasskeyAccount> {
  const { address, sign, chain, transport: clientTransport, entryPoint } = parameters;

  // Create viem client for reading contract state
  const { createPublicClient } = await import("viem");
  const client = createPublicClient({
    chain,
    transport: clientTransport,
  });

  const account = await toSmartAccount({
    client,
    entryPoint: {
      address: entryPoint.address,
      abi: entryPoint.abi,
      version: entryPoint.version,
    },

    // Encode multiple calls into batch execution format
    async encodeCalls(calls) {
      // Mode code for batch execution
      const modeCode = pad("0x01", { dir: "right" });

      // Encode calls array
      const executionData = encodeAbiParameters(callAbi, [
        calls.map((call) => ({
          to: call.to,
          value: call.value ?? 0n,
          data: call.data ?? "0x",
        })),
      ]);

      // execute(bytes32 mode, bytes executionData) selector
      const selector = "0xe9ae5c53";

      return concat([
        selector,
        encodeAbiParameters(
          [{ type: "bytes32" }, { type: "bytes" }],
          [modeCode, executionData],
        ),
      ]);
    },

    async getAddress() {
      return address;
    },

    async getNonce(parameters) {
      // Get nonce from EntryPoint
      const key = parameters?.key ?? 0n;
      const nonce = await client.readContract({
        abi: entryPoint.abi,
        address: entryPoint.address,
        functionName: "getNonce",
        args: [address, key],
      });
      return nonce as bigint;
    },

    async getStubSignature() {
      // Return stub signature for gas estimation
      // Must match the structure of a real passkey signature
      return await sign({
        hash: pad("0x", { size: 32 }),
      });
    },

    async signUserOperation(userOperation) {
      // Compute UserOperation hash using viem utility
      const { getUserOperationHash } = await import("viem/account-abstraction");

      const userOpHash = getUserOperationHash({
        userOperation: { ...userOperation, sender: address },
        entryPointAddress: entryPoint.address,
        entryPointVersion: entryPoint.version,
        chainId: chain.id,
      });

      // Sign with passkey (via WebAuthn through WASM)
      return await sign({ hash: userOpHash });
    },

    async decodeCalls() {
      // Not needed for basic usage
      return [];
    },

    async getFactoryArgs() {
      // Return empty for already-deployed accounts
      // TODO: Add deployment logic if needed
      return {};
    },

    async signMessage({ message }) {
      // ERC-1271 message signing
      return await sign({
        hash: hashMessage(message),
      });
    },

    async signTypedData() {
      // signTypedData is not commonly used for smart accounts
      // For now, throw an error or return placeholder
      throw new Error("signTypedData not supported for passkey accounts");
    },
  });

  return account;
}
