// Session actions exports
export type {
  CreateSessionParams,
  CreateSessionReturnType,
  GetSessionStateParams,
  GetSessionStateReturnType,
  SessionState,
  SessionStateEvent,
  SessionStateEventCallback,
} from "./sessions.js";
export { createSession, getSessionState, SessionEventType, SessionStatus } from "./sessions.js";

// Smart account deployment (prepare-only) exports
export type {
  PrepareDeploySmartAccountParams,
  PrepareDeploySmartAccountResult,
} from "./deploy.js";
export { getAccountAddressFromLogs, prepareDeploySmartAccount } from "./deploy.js";

// Passkey management exports
export type {
  AddPasskeyParams,
  AddPasskeyResult,
  FetchAccountParams,
  FetchAccountResult,
  FindAddressesByPasskeyParams,
  FindAddressesByPasskeyResult,
} from "./passkey.js";
export { addPasskey, fetchAccount, findAddressesByPasskey } from "./passkey.js";

// Module management exports
export type {
  IsModuleInstalledParams,
  IsModuleInstalledResult,
} from "./modules.js";
export { isGuardianModuleInstalled, isModuleInstalled, ModuleType } from "./modules.js";

// Utilities
export { generateAccountId } from "./utils.js";
