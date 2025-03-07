import type { Chain, Transport } from "viem";

import type { ClientWithOidcData } from "../client.js";
import type { ZksyncSsoOidcActions } from "../decorators/actions.js";
import { addNewPasskeyViaOidc, type AddNewPasskeyViaOidcArgs } from "./addNewPasskeyViaOidc.js";
import { calculateTxHash, type CalculateTxHashArgs } from "./calculateTxHash.js";

export function zksyncSsoRecoveryActions<
  transport extends Transport,
  chain extends Chain,
>(client: ClientWithOidcData<transport, chain>): ZksyncSsoOidcActions {
  return {
    addNewPasskeyViaOidc: async (args: Omit<AddNewPasskeyViaOidcArgs, "contracts">) => {
      return addNewPasskeyViaOidc(client, { ...args, contracts: client.contracts });
    },
    calculateTxHash: async (args: Omit<CalculateTxHashArgs, "contracts">) => {
      return calculateTxHash(client, { ...args, contracts: client.contracts });
    },
  };
}
