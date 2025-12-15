import {
  type Address,
  type Chain,
  type Hash,
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
import type { CustomPaymasterHandler } from "zksync-sso/paymaster";
import {
  decode_nonce_result,
  encode_get_nonce_call_data,
  encode_session_execute_call_data,
  generate_session_stub_signature_wasm,
  keyed_nonce_decimal,
  session_signature_no_validation_wasm,
} from "zksync-sso-web-sdk/bundler";

import type { SessionSpec } from "./types.js";
import { sessionSpecToJSON } from "./utils.js";

export type SessionRequiredContracts = {
  /** Session validator contract address (required for signature generation). */
  sessionValidator: Address;
};

export type ToSessionSmartAccountParams<
  TTransport extends Transport = Transport,
  TChain extends Chain = Chain,
> = {
  /** Public client used internally by the smart account implementation (must have chain). */
  client: PublicClient<TTransport, TChain>;
  /** Session key private key (used to sign user operations within policy limits). */
  sessionKeyPrivateKey: Hash;
  /** Smart account address (required - no counterfactual support). */
  address: Address;
  /** Session specification defining policies and limits. */
  sessionSpec: SessionSpec;
  /** Optional timestamp override (for testing or special timestamp handling). */
  currentTimestamp?: bigint;
  /** Session required contracts. */
  contracts: SessionRequiredContracts;
  /** Optional paymaster handler for sponsored transactions. */
  paymasterHandler?: CustomPaymasterHandler;
  /** Optional override for EntryPoint address (defaults to viem's entryPoint08Address). */
  entryPointAddress?: Address;
};

/**
 * Builds a SSO SmartAccount instance which uses a session key for signing within policy limits.
 * This allows delegated transaction signing with enforced limits on gas fees, transfer values, and call parameters.
 */
export async function toSessionSmartAccount<
  TTransport extends Transport = Transport,
  TChain extends Chain = Chain,
>({
  client,
  sessionKeyPrivateKey,
  address,
  contracts,
  sessionSpec,
  currentTimestamp,
  entryPointAddress,
}: ToSessionSmartAccountParams<TTransport, TChain>): Promise<ToSmartAccountReturnType> {
  // Precompute session spec JSON once
  const sessionSpecJSON = sessionSpecToJSON(sessionSpec);
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

      // Get the nonce key for this session signer
      const nonceKeyDecimal = keyed_nonce_decimal(sessionSpec.signer);

      // Encode the getNonce call with the session's nonce key
      const calldata = encode_get_nonce_call_data(sender, nonceKeyDecimal) as Hex;

      // Call EntryPoint
      const result = await client.call({
        to: epAddress,
        data: calldata,
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
      } as any);

      // Decode result
      const nonceStr = decode_nonce_result(result.data!);
      return BigInt(nonceStr as string);
    },

    // --- Calls encoding (session execute only - batch not supported yet) ---
    async encodeCalls(calls) {
      // Only support single calls (no batch operations)
      if (calls.length !== 1) {
        throw new Error(
          "Batch transactions are not supported for session accounts. Only single execute() calls are allowed.",
        );
      }

      const call = calls[0];

      // Use Rust SDK for encoding session execute
      const valueStr = (call.value ?? 0n).toString();

      const encoded = encode_session_execute_call_data(
        call.to,
        valueStr,
        call.data ?? "0x",
      ) as Hex;

      return encoded;
    },

    async decodeCalls() {
      throw new Error("decodeCalls is not supported yet for session accounts.");
    },

    // --- Address & factory args ---
    async getAddress() {
      // Return the provided address (no counterfactual support yet)
      return address;
    },
    async getFactoryArgs() {
      // No counterfactual deployment support yet
      throw new Error(
        "getFactoryArgs is not supported for session accounts. Deploy the account first and provide the address.",
      );
    },

    // --- Stubs & signing ---
    async getStubSignature() {
      // Generate session stub signature for gas estimation
      const timestampStr = currentTimestamp?.toString();
      return generate_session_stub_signature_wasm(
        contracts.sessionValidator,
        sessionSpecJSON,
        timestampStr,
      ) as Hex;
    },
    async signMessage({ message }) {
      throw new Error(
        `signMessage is not supported for session accounts. Message: ${String(message)}`,
      );
    },

    async signTypedData() {
      throw new Error("signTypedData is not supported for session accounts");
    },
    async signUserOperation(params) {
      const sender = await this.getAddress();

      // Default undefined fields to 0n to avoid runtime errors during early signing.
      const nonce = params.nonce ?? 0n;
      const callGasLimit = params.callGasLimit ?? 0n;
      const verificationGasLimit = params.verificationGasLimit ?? 0n;
      const preVerificationGas = params.preVerificationGas ?? 0n;
      const maxFeePerGas = params.maxFeePerGas ?? 0n;
      const maxPriorityFeePerGas = params.maxPriorityFeePerGas ?? 0n;
      const paymasterAndData = (params.paymasterAndData ?? "0x") as Hex;

      // Debug: Log effective gas & nonce passed into signing to diagnose zero-gas AA23 issues
      console.debug(
        "[toSessionSmartAccount] signUserOperation params:",
        {
          nonce: nonce.toString(),
          callGasLimit: callGasLimit.toString(),
          verificationGasLimit: verificationGasLimit.toString(),
          preVerificationGas: preVerificationGas.toString(),
          maxFeePerGas: maxFeePerGas.toString(),
          maxPriorityFeePerGas: maxPriorityFeePerGas.toString(),
        },
      );

      // Compute user operation hash locally using viem helper (includes paymasterAndData)
      const userOpHash = getUserOperationHash({
        chainId: Number(client.chain!.id),
        entryPointAddress: this.entryPoint.address,
        entryPointVersion: "0.8",
        userOperation: {
          sender,
          nonce,
          initCode: (params.initCode ?? "0x") as Hex,
          callData: (params.callData ?? "0x") as Hex,
          callGasLimit,
          verificationGasLimit,
          preVerificationGas,
          maxFeePerGas,
          maxPriorityFeePerGas,
          paymasterAndData,
          signature: "0x",
        },
      } as any) as Hex;

      // Parse target and selector from callData for validation
      // The callData is already encoded by encodeCalls, so we need to extract execute() params
      // For now, we'll trust that the session policies are validated at contract level
      // In a full implementation, you'd decode the execute call to validate against session spec

      const timestampStr = currentTimestamp?.toString();

      // Sign using session key with no validation (contract will validate)
      const signature = session_signature_no_validation_wasm(
        sessionKeyPrivateKey,
        contracts.sessionValidator,
        sessionSpecJSON,
        userOpHash,
        timestampStr,
      ) as Hex;

      return signature;
    },
  });
}
