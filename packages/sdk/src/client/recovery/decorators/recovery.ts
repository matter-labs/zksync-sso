import { type Chain, type Transport } from "viem";

import { addAccountOwnerPasskey, type AddAccountOwnerPasskeyArgs, type AddAccountOwnerPasskeyReturnType } from "../../passkey/index.js";
import type { ClientWithZksyncSsoRecoveryData } from "../client.js";

export type ZksyncSsoRecoveryActions = {
  addAccountOwnerPasskey: (args: Omit<AddAccountOwnerPasskeyArgs, "contracts">) => Promise<AddAccountOwnerPasskeyReturnType>;
};

export function zksyncSsoRecoveryActions<
  transport extends Transport,
  chain extends Chain,
>(client: ClientWithZksyncSsoRecoveryData<transport, chain>): ZksyncSsoRecoveryActions {
  return {
    addAccountOwnerPasskey: async (args: Omit<AddAccountOwnerPasskeyArgs, "contracts">) => {
      return await addAccountOwnerPasskey(client, {
        ...args,
        contracts: client.contracts,
      });
    },
  };
}
