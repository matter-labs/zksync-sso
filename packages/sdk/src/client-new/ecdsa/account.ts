import {
  type Address,
  type Chain,
  type Hash,
  hashMessage,
  type Hex,
  type PublicClient,
  type Transport,
} from "viem";
import {
  entryPoint08Abi,
  entryPoint08Address,
  toSmartAccount,
  type ToSmartAccountReturnType,
} from "viem/account-abstraction";
import {
  decode_nonce_result,
  encode_execute_call_data,
  encode_get_nonce_call_data,
  encode_get_user_operation_hash_call_data,
  generate_eoa_stub_signature,
  sign_eoa_message,
  sign_eoa_user_operation_hash,
  // @ts-expect-error - TypeScript doesn't understand package.json exports with node module resolution
} from "zksync-sso-web-sdk/bundler";

export type ToEcdsaSmartAccountParams<
  TTransport extends Transport = Transport,
  TChain extends Chain = Chain,
> = {
  /** Public client used internally by the smart account implementation (must have chain). */
  client: PublicClient<TTransport, TChain>;
  /** EOA signer private key. */
  signerPrivateKey: Hash;
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
  signerPrivateKey,
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
    async decodeCalls(data) {
      throw new Error("decodeCalls is not supported yet.");
    },

    // --- Address & factory args ---
    async getAddress() {
      // Simply return the provided address (no counterfactual support yet)
      return address;
    },
    async getFactoryArgs() {
      // No counterfactual deployment support yet
      throw new Error("getFactoryArgs is not supported. Deploy the account first and provide the address.");
    },

    // --- Stubs & signing ---
    async getStubSignature() {
      // Use Rust SDK to generate proper stub signature
      return generate_eoa_stub_signature(validatorAddress) as Hex;
    },
    async signMessage({ message }) {
      // Hash the message if it's not already hashed
      const messageToSign = typeof message === "string" ? message : hashMessage(message);
      return sign_eoa_message(signerPrivateKey, messageToSign) as Hex;
    },
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    async signTypedData({ domain, types, primaryType, message }) {
      throw new Error("signTypedData is not supported for ECDSA smart accounts");
    },
    async signUserOperation(params) {
      const sender = await this.getAddress();

      // Encode call data for EntryPoint.getUserOpHash() using Rust SDK
      const callData = encode_get_user_operation_hash_call_data(
        sender,
        params.nonce.toString(),
        params.callData,
        params.callGasLimit.toString(),
        params.verificationGasLimit.toString(),
        params.preVerificationGas.toString(),
        params.maxFeePerGas.toString(),
        params.maxPriorityFeePerGas.toString(),
      ) as Hex;

      // Result is the bytes32 hash from EntryPoint.getUserOpHash()
      const { data: userOpHash } = await client.call({
        to: this.entryPoint.address,
        data: callData,
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
      } as any);

      const signature = sign_eoa_user_operation_hash(
        userOpHash,
        signerPrivateKey,
        validatorAddress,
      ) as Hex;

      return signature;
    },
  });
}
