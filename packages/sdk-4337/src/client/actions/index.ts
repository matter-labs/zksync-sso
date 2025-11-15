// Session creation exports
export type { CreateSessionParams, CreateSessionReturnType } from "./createSession.js";
export { createSession } from "./createSession.js";

// Session state query exports
export type {
  GetSessionStateParams,
  GetSessionStateReturnType,
  SessionState,
  SessionStateEvent,
  SessionStateEventCallback,
} from "./getSessionState.js";
export { getSessionState, SessionEventType, SessionStatus } from "./getSessionState.js";

// Smart account deployment (prepare-only) exports
export type {
  PrepareDeploySmartAccountParams,
  PrepareDeploySmartAccountResult,
} from "./deploy.js";
export { getAccountAddressFromLogs, prepareDeploySmartAccount } from "./deploy.js";

// Passkey management exports
export type { AddPasskeyParams, AddPasskeyResult } from "./passkey.js";
export { addPasskey } from "./passkey.js";

// Utilities
export { generateAccountId } from "./utils.js";
