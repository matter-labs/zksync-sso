import { addNewPasskeyViaOidc, type AddNewPasskeyViaOidcArgs } from "../actions/addNewPasskeyViaOidc.js";

export type ZksyncSsoOidcActions = {
  addNewPasskeyViaOidc: (args: Omit<AddNewPasskeyViaOidcArgs, "contracts">) => ReturnType<typeof addNewPasskeyViaOidc>;
};
