import type { Chain, Transport } from "viem";

import type { ClientWithOidcData } from "../client.js";
import type { ZksyncSsoOidcActions } from "../decorators/actions.js";
import { addNewPasskeyViaOidc, type AddNewPasskeyViaOidcArgs } from "./addNewPasskeyViaOidc.js";

export function zksyncSsoRecoveryActions<
  transport extends Transport,
  chain extends Chain,
>(client: ClientWithOidcData<transport, chain>): ZksyncSsoOidcActions {
  return {
    addNewPasskeyViaOidc: async (args: Omit<AddNewPasskeyViaOidcArgs, "contracts">) => {
      return addNewPasskeyViaOidc(client, { ...args, contracts: client.contracts });
    },
  };
}
