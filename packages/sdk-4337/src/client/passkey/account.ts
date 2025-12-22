import {
  type Address,
  type Chain,
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
  generate_passkey_stub_signature,
} from "zksync-sso-web-sdk/bundler";

import { signWithPasskey } from "./webauthn.js";

export type ToPasskeySmartAccountParams<
  TTransport extends Transport = Transport,
  TChain extends Chain = Chain,
> = {
  /** Public client used internally by the smart account implementation (must have chain). */
  client: PublicClient<TTransport, TChain>;
  /** Smart account address (required - no counterfactual support). */
  address: Address;
  /** Passkey validator contract address (required for signature formatting). */
  validatorAddress: Address;
  /** Passkey credential ID (hex string). */
  credentialId: Hex;
  /** Relying Party ID (domain where passkey was created). */
  rpId: string;
  /** Origin URL (for WebAuthn verification). */
  origin: string;
  /** Optional override for EntryPoint address (defaults to viem's entryPoint08Address). */
  entryPointAddress?: Address;
};

/**
 * Builds a SmartAccount that is owned by a passkey.
 * Uses viem's generic `toSmartAccount` under the hood.
 * All encoding is handled by Rust SDK, viem handles network requests, WebAuthn handles signing.
 */
export async function toPasskeySmartAccount<
  TTransport extends Transport = Transport,
  TChain extends Chain = Chain,
>({
  client,
  address,
  validatorAddress,
  credentialId,
  rpId,
  origin,
  entryPointAddress,
}: ToPasskeySmartAccountParams<TTransport, TChain>): Promise<ToSmartAccountReturnType> {
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

      // Encode the getNonce call
      const calldata = encode_get_nonce_call_data(sender, "0") as Hex;

      // Viem makes the network call
      const result = await client.call({
        to: epAddress,
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
      // Decoding is not needed for passkey accounts using Rust SDK
      // All encoding is handled by Rust, and we don't need to decode back
      throw new Error("decodeCalls is not supported. All encoding is handled by Rust SDK.");
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
      return generate_passkey_stub_signature(validatorAddress) as Hex;
    },
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    async signMessage({ message }) {
      throw new Error("signMessage is not supported for passkey accounts yet");
    },
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    async signTypedData({ domain, types, primaryType, message }) {
      throw new Error("signTypedData is not supported for passkey accounts");
    },
    async signUserOperation(params) {
      const sender = await this.getAddress();

      // For EntryPoint v0.8, extract paymaster fields if provided as separate params
      const hasPaymaster = !!(params as any).paymaster;
      const paymaster = (params as any).paymaster as Address | undefined;
      const paymasterData = ((params as any).paymasterData as Hex) ?? ("0x" as Hex);
      const paymasterVerificationGasLimit = (params as any).paymasterVerificationGasLimit as bigint | undefined;
      const paymasterPostOpGasLimit = (params as any).paymasterPostOpGasLimit as bigint | undefined;

      console.log("[passkey/account.ts] signUserOperation called with:", {
        hasPaymaster,
        paymaster,
        paymasterData,
        paymasterVerificationGasLimit: paymasterVerificationGasLimit?.toString(),
        paymasterPostOpGasLimit: paymasterPostOpGasLimit?.toString(),
        nonce: params.nonce?.toString(),
      });

      // Compute user operation hash for v0.8
      // IMPORTANT: For v0.8, DO NOT use paymasterAndData - use the separate fields directly
      const userOp: any = {
        sender,
        nonce: params.nonce,
        initCode: (params.initCode ?? "0x") as Hex,
        callData: (params.callData ?? "0x") as Hex,
        callGasLimit: params.callGasLimit,
        verificationGasLimit: params.verificationGasLimit,
        preVerificationGas: params.preVerificationGas,
        maxFeePerGas: params.maxFeePerGas,
        maxPriorityFeePerGas: params.maxPriorityFeePerGas,
        signature: "0x",
      };

      // Add paymaster fields if present (v0.8 uses separate fields, not paymasterAndData)
      if (hasPaymaster && paymaster) {
        userOp.paymaster = paymaster;
        userOp.paymasterVerificationGasLimit = paymasterVerificationGasLimit;
        userOp.paymasterPostOpGasLimit = paymasterPostOpGasLimit;
        userOp.paymasterData = paymasterData;
      }

      const userOpHash = getUserOperationHash({
        chainId: Number(client.chain!.id),
        entryPointAddress: this.entryPoint.address,
        entryPointVersion: "0.8",
        userOperation: userOp,
      } as any) as Hex;

      console.log("[passkey/account.ts] Computed UserOp hash:", userOpHash);
      console.log("[passkey/account.ts] Hash includes paymaster fields:", hasPaymaster);

      // Sign with WebAuthn (browser API) and get complete signature
      // signWithPasskey handles:
      // 1. Converting hash to WebAuthn challenge
      // 2. Calling browser WebAuthn API
      // 3. Parsing DER signature
      // 4. ABI-encoding via Rust SDK
      // 5. Prepending validator address
      const signature = await signWithPasskey({
        hash: userOpHash as Hex,
        credentialId,
        validatorAddress,
        rpId,
        origin,
      });

      console.log("[passkey/account.ts] Generated signature:", signature);
      console.log("[passkey/account.ts] Signature includes paymaster:", hasPaymaster);

      return signature;
    },
  });
}
