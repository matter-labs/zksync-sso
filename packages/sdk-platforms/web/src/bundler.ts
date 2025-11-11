// Bundler-specific entry point for web applications
import * as wasm from "../pkg-bundler/zksync_sso_erc4337_web_ffi";

export * from "./types";
export * from "./webauthn";
export * from "./webauthn-helpers";

// Re-export core WASM functions for passkey flow
export const {
  // ===== CORE PASSKEY FUNCTIONS (PRIMARY API) =====
  // These are the essential functions for implementing passkey-based smart accounts
  deploy_account, // Deploy a new smart account with passkey
  add_passkey_to_account, // Add additional passkey to existing account
  prepare_passkey_user_operation, // Prepare transaction for passkey signing
  send_transaction_eoa,
  submit_passkey_user_operation, // Submit signed transaction to bundler
  compute_account_id, // Generate unique account ID from passkey

  // ===== CONFIGURATION TYPES =====
  // TypeScript interfaces for configuration objects
  PasskeyPayload, // Passkey credential data
  DeployAccountConfig, // Account deployment configuration
  SendTransactionConfig, // Transaction sending configuration
  EncodeGetUserOperationHashParams, // Params for encoding getUserOpHash call

  // ===== NEW SDK HELPER FUNCTIONS =====
  // These functions are used by the new viem-based SDK (packages/sdk/src/client-new)
  encode_get_nonce_call_data, // Encode EntryPoint.getNonce() call
  decode_nonce_result, // Decode nonce result from EntryPoint
  encode_execute_call_data, // Encode execute() call for smart account
  generate_eoa_stub_signature, // Generate stub signature for gas estimation (EOA)
  generate_passkey_stub_signature, // Generate stub signature for gas estimation (Passkey)
  sign_eoa_message, // Sign message hash with EOA private key
  sign_eoa_user_operation_hash, // Sign UserOperation hash with EOA private key
  encode_get_user_operation_hash_call_data, // Encode EntryPoint.getUserOpHash() call (no network request)
  encode_passkey_signature, // Encode passkey signature for on-chain verification

  // ===== DEPLOYMENT HELPER FUNCTIONS =====
  // Functions for encoding deployment transactions (client-new/actions/deploy.ts)
  generate_account_id, // Generate deterministic or random account ID
  encode_deploy_account_call_data, // Encode MSAFactory.deployAccount() call data
} = wasm;

// Initialize WASM module
export const init = wasm.init;

// Auto-initialize for bundler environments
wasm.init();
