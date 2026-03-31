// Re-export all client-new implementations
export * from "./actions/index.js";
export * from "./ecdsa/index.js";
export * from "./passkey/index.js";
export * from "./session/index.js";

// Re-export client-auth-server types for auth server compatibility
export type { AppMetadata, SessionPreferences } from "../client-auth-server/index.js";
export type { ExtractParams, ExtractReturnType, Method, RPCRequestMessage, RPCResponseMessage } from "../client-auth-server/rpc.js";
export { formatSessionPreferences } from "../client-auth-server/session/index.js";
