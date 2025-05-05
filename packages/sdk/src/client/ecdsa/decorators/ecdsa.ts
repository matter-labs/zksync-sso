import { type Chain, type Transport } from "viem";

import {
  createSession, type CreateSessionArgs, type CreateSessionReturnType,
  revokeSession, type RevokeSessionArgs, type RevokeSessionReturnType,
} from "../../session/actions/session.js";
import type { ClientWithZksyncSsoEcdsaData } from "../client.js";

export type ZksyncSsoEcdsaActions = {
  createSession: (args: Omit<CreateSessionArgs, "contracts">) => Promise<CreateSessionReturnType>;
  revokeSession: (args: Omit<RevokeSessionArgs, "contracts">) => Promise<RevokeSessionReturnType>;
};

export const zksyncSsoEcdsaActions = <
  transport extends Transport = Transport,
  chain extends Chain = Chain,
>(client: ClientWithZksyncSsoEcdsaData<transport, chain>) => {
  return {
    createSession: async (args: Omit<CreateSessionArgs, "contracts">) => {
      return await createSession(client, {
        ...args,
        contracts: client.contracts,
      });
    },
    revokeSession: async (args: Omit<RevokeSessionArgs, "contracts">) => {
      return await revokeSession(client, {
        ...args,
        contracts: client.contracts,
      });
    },
  };
};
