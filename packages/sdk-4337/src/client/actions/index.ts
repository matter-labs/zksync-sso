export type {
  CreateSessionParams,
  CreateSessionReturnType,
} from "./createSession.js";
export { createSession } from "./createSession.js";
export type {
  PrepareDeploySmartAccountParams,
  PrepareDeploySmartAccountResult,
} from "./deploy.js";
export type {
  DeploySmartAccountParams,
  DeploySmartAccountResult,
} from "./deploy.js";
export { getAccountAddressFromLogs, prepareDeploySmartAccount } from "./deploy.js";
export { deploySmartAccount, getDeployedAccountAddress } from "./deploy.js";
export type { AddPasskeyParams, AddPasskeyResult } from "./passkey.js";
export { addPasskey } from "./passkey.js";
export { generateAccountId } from "./utils.js";
