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
