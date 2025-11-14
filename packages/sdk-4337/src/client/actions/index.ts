export type {
  PrepareDeploySmartAccountParams,
  PrepareDeploySmartAccountResult,
} from "./deploy.js";
export { getAccountAddressFromLogs, prepareDeploySmartAccount } from "./deploy.js";
export type {
  AddPasskeyParams,
  AddPasskeyResult,
  FindAddressesByPasskeyParams,
  FindAddressesByPasskeyResult,
} from "./passkey.js";
export { addPasskey, findAddressesByPasskey } from "./passkey.js";
export { generateAccountId } from "./utils.js";
