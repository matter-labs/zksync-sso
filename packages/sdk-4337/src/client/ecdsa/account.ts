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
  getUserOperationHash,
  toSmartAccount,
  type ToSmartAccountReturnType,
} from "viem/account-abstraction";
import {
  decode_nonce_result,
  encode_execute_call_data,
  encode_get_nonce_call_data,
  generate_eoa_stub_signature,
  sign_eoa_message,
  sign_eoa_user_operation_hash,
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
  eoaValidatorAddress: Address;
  /** Optional override for EntryPoint address (defaults to viem's entryPoint08Address). */
  entryPointAddress?: Address;
};

/**
 * Builds a SSO SmartAccount instance which uses an ECDSA EOA for signing.
 */
export async function toEcdsaSmartAccount<
  TTransport extends Transport = Transport,
  TChain extends Chain = Chain,
>({
  client,
  signerPrivateKey,
  address,
  eoaValidatorAddress,
  entryPointAddress,
}: ToEcdsaSmartAccountParams<TTransport, TChain>): Promise<ToSmartAccountReturnType> {
  const epAddress = entryPointAddress ?? entryPoint08Address;
  return toSmartAccount({
    client,
    entryPoint: {
      abi: entryPoint08Abi,
      address: epAddress,
      version: "0.8",
    },
    async getNonce() {
      const sender = await this.getAddress();
      const calldata = encode_get_nonce_call_data(sender, "0") as Hex;
      const result = await client.request({
        method: "eth_call",
        params: [{
          from: sender,
          to: epAddress,
          data: calldata,
        }],
      });
      const nonceStr = decode_nonce_result(result);
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
      return generate_eoa_stub_signature(eoaValidatorAddress) as Hex;
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
      // Compute user operation hash locally with full fields (including paymaster)
      const userOpHash = getUserOperationHash({
        chainId: Number(client.chain!.id),
        entryPointAddress: this.entryPoint.address,
        entryPointVersion: "0.8",
        userOperation: {
          sender,
          nonce: params.nonce,
          initCode: (params.initCode ?? "0x") as Hex,
          callData: (params.callData ?? "0x") as Hex,
          callGasLimit: params.callGasLimit,
          verificationGasLimit: params.verificationGasLimit,
          preVerificationGas: params.preVerificationGas,
          maxFeePerGas: params.maxFeePerGas,
          maxPriorityFeePerGas: params.maxPriorityFeePerGas,
          paymasterAndData: (params.paymasterAndData ?? "0x") as Hex,
          signature: "0x",
        },
      } as any) as Hex;

      const signature = sign_eoa_user_operation_hash(
        userOpHash,
        signerPrivateKey,
        eoaValidatorAddress,
      ) as Hex;

      return signature;
    },
  });
}
