// Main entry point - detects environment and uses appropriate WASM target
export * from "./client";
export * from "./session";
export * from "./types";

// Platform-specific exports for advanced usage
export type { InitOptions } from "./types";

// Re-export the client as default
export { ZkSyncSsoClient as default } from "./client";
