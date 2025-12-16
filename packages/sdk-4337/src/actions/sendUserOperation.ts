import type { Address, Hex } from "viem";
import {
  PaymasterParams,
  send_user_operation,
  SendTransactionConfig,
} from "zksync-sso-web-sdk/bundler";

/**
 * Paymaster configuration for sponsoring transactions
 */
export interface PaymasterConfig {
  /** Paymaster contract address */
  address: Address;
  /** Additional paymaster data (optional) */
  data?: Hex;
  /** Verification gas limit (optional) */
  verificationGasLimit?: bigint;
  /** Post-operation gas limit (optional) */
  postOpGasLimit?: bigint;
}

/**
 * Parameters for sending a user operation
 */
export interface SendUserOperationParams {
  /** RPC URL for the provider */
  rpcUrl: string;
  /** Bundler URL */
  bundlerUrl: string;
  /** EntryPoint contract address */
  entryPoint: Address;
  /** Smart account address */
  account: Address;
  /** Encoded call data */
  callData: Hex;
  /** Private key for signing (EOA) */
  privateKey: Hex;
  /** Validator module address */
  validator: Address;
  /** Optional paymaster configuration */
  paymaster?: PaymasterConfig;
}

/**
 * Send a user operation using the Rust SDK
 *
 * This function uses the Rust-based send_user_op implementation which:
 * - Includes paymaster in the UserOp before gas estimation
 * - Computes the UserOp hash with paymaster included
 * - Signs the complete hash (signature covers paymaster)
 * - Submits to the bundler with proper PaymasterAndData
 *
 * @param params - Send user operation parameters
 * @returns Promise that resolves with the user operation receipt
 */
export async function sendUserOperation(
  params: SendUserOperationParams,
): Promise<string> {
  const {
    rpcUrl,
    bundlerUrl,
    entryPoint,
    account,
    callData,
    privateKey,
    validator,
    paymaster,
  } = params;

  // Create SendTransactionConfig
  const config = new SendTransactionConfig(
    rpcUrl,
    bundlerUrl,
    entryPoint,
  );

  // Create PaymasterParams if provided
  const paymasterParams = paymaster
    ? new PaymasterParams(
      paymaster.address,
      paymaster.data || "0x",
      paymaster.verificationGasLimit?.toString(),
      paymaster.postOpGasLimit?.toString(),
    )
    : undefined;

  // Call Rust FFI function
  const result = await send_user_operation(
    config,
    account,
    callData,
    privateKey,
    validator,
    paymasterParams,
  );

  return result;
}
