// Bundler-specific entry point for web applications
import * as wasm from "../pkg-bundler/zksync_sso_erc4337_web_ffi";

export * from "./passkey-utils";
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

  // ===== HELPER FUNCTIONS =====
  // Used internally by webauthn-helpers.ts
  encode_passkey_signature, // ABI encode passkey signature components
  create_stub_passkey_signature, // Create stub signature for gas estimation

  // ===== CONFIGURATION TYPES =====
  // TypeScript interfaces for configuration objects
  PasskeyPayload, // Passkey credential data
  DeployAccountConfig, // Account deployment configuration
  SendTransactionConfig, // Transaction sending configuration

  // ===== ADVANCED/OPTIONAL FUNCTIONS =====
  // These may be useful for advanced use cases
  compute_smart_account_address, // Compute account address without deploying
  bytes_to_hex, // Convert bytes to hex string
  hex_to_bytes, // Convert hex string to bytes
} = wasm;

// Initialize WASM module
export const init = wasm.init;

// Auto-initialize for bundler environments
wasm.init();
