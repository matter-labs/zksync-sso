import { type Chain, type Hex, type Transport } from "viem";

import { addAccountOwnerPasskey, type AddAccountOwnerPasskeyArgs, type AddAccountOwnerPasskeyReturnType } from "../../passkey/index.js";
import { type CalculateAddKeyHashArgs, calculateAddKeyTxHash } from "../actions/oidc.js";
import type { ClientWithZksyncSsoRecoveryData } from "../client.js";

export type ZksyncSsoRecoveryActions = {
  addAccountOwnerPasskey: (args: Omit<AddAccountOwnerPasskeyArgs, "contracts">) => Promise<AddAccountOwnerPasskeyReturnType>;
  calculateAddKeyTxHash: (args: Omit<CalculateAddKeyHashArgs, "contracts">) => Promise<Hex>;
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
    calculateAddKeyTxHash: async (args: Omit<CalculateAddKeyHashArgs, "contracts">) => {
      return await calculateAddKeyTxHash(client, {
        ...args,
        contracts: client.contracts,
      });
    },
  };
}
