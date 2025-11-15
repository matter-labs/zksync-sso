// Node.js-specific entry point
import * as wasm from "../pkg-node/zksync_sso_erc4337_web_ffi";

export * from "./client";
export * from "./types";

import { setWasmBindings } from "./client";

// Re-export WASM functions
export const {
  greet,
  Client,
  Config,
  Contracts,
  Call,
  SendCallsRequest,
  ZkSyncSsoError,
  console_log_from_rust,
  // Session-related exports
  encode_session_execute_call_data,
  generate_session_stub_signature_wasm,
  session_signature_no_validation_wasm,
  keyed_nonce_decimal,
} = wasm;

// Initialize WASM module
export const init = wasm.init;

// Set up WASM bindings for the client wrapper
setWasmBindings({
  Client,
  Config,
  Contracts,
  Call,
  SendCallsRequest,
});

// Auto-initialize for Node.js environments
wasm.init();
