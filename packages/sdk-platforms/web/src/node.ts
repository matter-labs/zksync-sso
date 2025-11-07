// Node.js-specific entry point
import * as wasm from "../pkg-node/zksync_sso_erc4337_web_ffi.js";

export * from "./types";
// Note: webauthn and webauthn-helpers are browser-only (use @simplewebauthn/browser)
// They are only available in the bundler build

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

  // ===== EOA ENCODING AND SIGNING FUNCTIONS =====
  // Low-level functions for EOA-based smart accounts
  encode_execute_call_data, // Encode a single execute call for ERC-7579 accounts
  sign_eoa_user_operation_hash, // Sign UserOperation hash with EOA private key
  encode_get_nonce_call_data, // Encode getNonce call data (viem makes the network call)
  decode_nonce_result, // Decode nonce result from eth_call
  sign_eoa_message, // Sign message with EOA (EIP-191 personal sign)
  sign_eoa_typed_data, // Sign typed data with EOA (EIP-712)
  generate_eoa_stub_signature, // Generate stub signature for gas estimation
  encode_factory_create_account, // Encode factory createAccount calldata

  // ===== ENCODING FUNCTIONS =====
  // Low-level encoding functions for custom SDK integrations
  encode_create_session_call_data, // Encode session creation call data
  encode_revoke_session_call_data, // Encode session revocation call data
  compute_session_hash, // Compute hash of session spec
  encode_add_passkey_call_data, // Encode add passkey call data
  encode_add_validation_key_call_data, // Encode add validation key call data
  encode_passkey_signature, // Encode passkey signature for verification
  encode_deploy_account_calldata, // Encode complete deployment calldata for factory

  // ===== CONFIGURATION TYPES =====
  // TypeScript interfaces for configuration objects
  PasskeyPayload, // Passkey credential data
  DeployAccountConfig, // Account deployment configuration
  SendTransactionConfig, // Transaction sending configuration
} = wasm;

// Initialize WASM module
export const init = wasm.init;

// Auto-initialize for Node.js environments
wasm.init();
