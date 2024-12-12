import { type Chain, type Transport } from "viem";

import {
  createSession, type CreateSessionArgs, type CreateSessionReturnType,
  revokeSession, type RevokeSessionArgs, type RevokeSessionReturnType,
} from "../../session/actions/session.js";
import type { ClientWithZksyncSsoPasskeyData } from "../client.js";

export type ZksyncSsoPasskeyActions = {
  createSession: (args: Omit<CreateSessionArgs, "contracts">) => Promise<CreateSessionReturnType>;
  revokeSession: (args: Omit<RevokeSessionArgs, "contracts">) => Promise<RevokeSessionReturnType>;
};

export function zksyncSsoPasskeyActions<
  transport extends Transport,
  chain extends Chain,
>(client: ClientWithZksyncSsoPasskeyData<transport, chain>): ZksyncSsoPasskeyActions {
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
}
