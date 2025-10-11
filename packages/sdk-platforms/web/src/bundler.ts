// Bundler-specific entry point for web applications
import * as wasm from "../pkg-bundler/zksync_sso_erc4337_web_ffi";

export * from "./client";
export * from "./types";

import { setWasmBindings } from "./client";

// Re-export WASM functions
export const {
  greet,
  get_chain_info,
  get_ethereum_sepolia_info,
  parse_contract_addresses,
  compute_account_id,
  compute_smart_account_address,
  test_http_transport,
  deploy_account,
  Client,
  Config,
  Contracts,
  Call,
  SendCallsRequest,
  ZkSyncSsoError,
  bytes_to_hex,
  hex_to_bytes,
  console_log_from_rust,
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

// Auto-initialize for bundler environments
wasm.init();
