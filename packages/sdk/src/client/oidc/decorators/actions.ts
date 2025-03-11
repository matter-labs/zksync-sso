import { addNewPasskeyViaOidc, type AddNewPasskeyViaOidcArgs } from "../actions/addNewPasskeyViaOidc.js";
import { calculateTxHash, type CalculateTxHashArgs } from "../actions/calculateTxHash.js";

export type ZksyncSsoOidcActions = {
  addNewPasskeyViaOidc: (args: Omit<AddNewPasskeyViaOidcArgs, "contracts">) => ReturnType<typeof addNewPasskeyViaOidc>;
  calculateTxHash: (args: Omit<CalculateTxHashArgs, "contracts">) => ReturnType<typeof calculateTxHash>;
};
