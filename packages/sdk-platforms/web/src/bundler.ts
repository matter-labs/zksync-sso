// Bundler-specific entry point for web applications
import initWasm, * as wasm from "../pkg-bundler/zksync_sso_erc4337_web_ffi";

export * from "./session";
export * from "./types";
export * from "./webauthn";
export * from "./webauthn-helpers";

// Re-export core WASM functions for passkey flow
// Explicitly export WASM functions and types
export const deploy_account = wasm.deploy_account;
export const add_passkey_to_account = wasm.add_passkey_to_account;
export const add_session_to_account = wasm.add_session_to_account;
export const prepare_passkey_user_operation = wasm.prepare_passkey_user_operation;
export const send_transaction_eoa = wasm.send_transaction_eoa;
export const send_transaction_session = wasm.send_transaction_session;
export const submit_passkey_user_operation = wasm.submit_passkey_user_operation;
export const compute_account_id = wasm.compute_account_id;

export const PasskeyPayload = wasm.PasskeyPayload;
export const DeployAccountConfig = wasm.DeployAccountConfig;
export const SendTransactionConfig = wasm.SendTransactionConfig;

// ===== SESSION TYPES (PHASE 2) =====
export const SessionPayload = wasm.SessionPayload;
export const TransferPayload = wasm.TransferPayload;

// Initialize WASM module (async for web target)
let initPromise: Promise<unknown> | null = null;

export async function init(): Promise<void> {
  if (!initPromise) {
    initPromise = initWasm();
  }
  await initPromise;
}

// Auto-initialize for bundler environments (async)
init();
