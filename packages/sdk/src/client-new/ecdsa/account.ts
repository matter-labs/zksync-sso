import {
  type Address,
  type Chain,
  hashMessage,
  type Hex,
  type PublicClient,
  type Transport,
} from "viem";
import {
  entryPoint08Abi,
  entryPoint08Address,
  getUserOperationHash,
  toSmartAccount,
  type ToSmartAccountReturnType,
} from "viem/account-abstraction";
import type { LocalAccount } from "viem/accounts";
import {
  decode_nonce_result,
  encode_execute_call_data,
  encode_get_nonce_call_data,
  generate_eoa_stub_signature,
  sign_eoa_message,
  sign_eoa_typed_data,
  sign_eoa_user_operation_hash,
  // @ts-expect-error - TypeScript doesn't understand package.json exports with node module resolution
} from "zksync-sso-web-sdk/bundler";

export type ToEcdsaSmartAccountParams<
  TTransport extends Transport = Transport,
  TChain extends Chain = Chain,
> = {
  /** Public client used internally by the smart account implementation (must have chain). */
  client: PublicClient<TTransport, TChain>;
  /** EOA owner. */
  owner: LocalAccount;
  /** Smart account address (required - no counterfactual support). */
  address: Address;
  /** EOA validator contract address (required for stub signature generation). */
  validatorAddress: Address;
};

/**
 * Builds a SmartAccount that is owned by an EOA.
 * Uses viem's generic `toSmartAccount` under the hood.
 * All encoding/signing is handled by Rust SDK, viem handles network requests.
 */
export async function toEcdsaSmartAccount<
  TTransport extends Transport = Transport,
  TChain extends Chain = Chain,
>({
  client,
  owner,
  address,
  validatorAddress,
}: ToEcdsaSmartAccountParams<TTransport, TChain>): Promise<ToSmartAccountReturnType> {
  return toSmartAccount({
    client,
    entryPoint: {
      abi: entryPoint08Abi,
      address: entryPoint08Address,
      version: "0.8",
    },
    async getNonce() {
      const sender = await this.getAddress();

      // Encode the getNonce call
      const calldata = encode_get_nonce_call_data(sender, "0") as Hex;

      // Viem makes the network call
      const result = await client.call({
        to: entryPoint08Address,
        data: calldata,
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      } as any);

      // Decode the result
      const nonceStr = decode_nonce_result(result.data!);
      return BigInt(nonceStr as string);
    },

    // --- Calls encoding (execute only - batch not supported yet) ---
    async encodeCalls(calls) {
      // Only support single calls (no batch operations)
      if (calls.length !== 1) {
        throw new Error("Batch transactions are not supported. Only single execute() calls are allowed.");
      }

      const call = calls[0];

      // Use Rust SDK for encoding
      // Convert value to string (Rust expects string representation)
      const valueStr = (call.value ?? 0n).toString();

      // Call Rust SDK to encode
      const encoded = encode_execute_call_data(
        call.to,
        valueStr,
        call.data ?? "0x",
      ) as Hex;

      return encoded;
    },
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    async decodeCalls(_data) {
      // Decoding is not needed for EOA accounts using Rust SDK
      // All encoding is handled by Rust, and we don't need to decode back
      throw new Error("decodeCalls is not supported. All encoding is handled by Rust SDK.");
    },

    // --- Address & factory args ---
    async getAddress() {
      // Simply return the provided address (no counterfactual support)
      return address;
    },
    async getFactoryArgs() {
      // No counterfactual deployment support
      throw new Error("getFactoryArgs is not supported. Deploy the account first and provide the address.");
    },

    // --- Stubs & signing ---
    async getStubSignature() {
      // Use Rust SDK to generate proper stub signature
      return generate_eoa_stub_signature(validatorAddress) as Hex;
    },
    async signMessage({ message }) {
      // Use Rust SDK for message signing
      // @ts-expect-error - accessing private key from LocalAccount
      const privateKey = owner.key as Hex;

      // Hash the message if it's not already hashed
      const messageToSign = typeof message === "string" ? message : hashMessage(message);
      return sign_eoa_message(privateKey, messageToSign) as Hex;
    },
    async signTypedData({ domain, types, primaryType, message }) {
      // Use Rust SDK for typed data signing
      // @ts-expect-error - accessing private key from LocalAccount
      const privateKey = owner.key as Hex;

      return sign_eoa_typed_data(
        privateKey,
        JSON.stringify(domain),
        JSON.stringify(types),
        primaryType,
        JSON.stringify(message),
      ) as Hex;
    },
    async signUserOperation(params) {
      const sender = await this.getAddress();
      const chainId = client.chain.id;
      const userOpHash = getUserOperationHash({
        chainId,
        entryPointAddress: this.entryPoint.address,
        entryPointVersion: this.entryPoint.version,
        userOperation: { ...params, sender },
      });

      // Use Rust SDK for signing (ensures proper s-value normalization)
      console.log("owner", owner);
      // Access the private key from the owner
      // @ts-expect-error - accessing private key from LocalAccount
      const privateKey = owner.key as Hex;

      const signature = sign_eoa_user_operation_hash(
        userOpHash,
        privateKey,
      ) as Hex;

      return signature;
    },
  });
}
