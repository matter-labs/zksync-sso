import { type Chain, type Transport } from "viem";

import {
  confirmGuardian, type ConfirmGuardianArgs, type ConfirmGuardianReturnType,
  proposeGuardian, type ProposeGuardianArgs, type ProposeGuardianReturnType,
  removeGuardian, type RemoveGuardianArgs, type RemoveGuardianReturnType,
} from "../../recovery/actions/recovery.js";
import {
  createSession, type CreateSessionArgs, type CreateSessionReturnType,
  revokeSession, type RevokeSessionArgs, type RevokeSessionReturnType,
} from "../../session/actions/session.js";
import {
  removePasskey, type RemovePasskeyArgs, type RemovePasskeyReturnType,
} from "../actions/passkey.js";
import type { ClientWithZksyncSsoPasskeyData } from "../client.js";

export type ZksyncSsoPasskeyActions = {
  createSession: (args: Omit<CreateSessionArgs, "contracts">) => Promise<CreateSessionReturnType>;
  revokeSession: (args: Omit<RevokeSessionArgs, "contracts">) => Promise<RevokeSessionReturnType>;
  proposeGuardian: (args: Omit<ProposeGuardianArgs, "contracts">) => Promise<ProposeGuardianReturnType>;
  confirmGuardian: (args: Omit<ConfirmGuardianArgs, "contracts">) => Promise<ConfirmGuardianReturnType>;
  removeGuardian: (args: Omit<RemoveGuardianArgs, "contracts">) => Promise<RemoveGuardianReturnType>;
  removePasskey: (args: Omit<RemovePasskeyArgs, "contracts">) => Promise<RemovePasskeyReturnType>;
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
    proposeGuardian: async (args: Omit<ProposeGuardianArgs, "contracts">) => {
      return await proposeGuardian(client, {
        ...args,
        contracts: client.contracts,
      });
    },
    confirmGuardian: async (args: Omit<ConfirmGuardianArgs, "contracts">) => {
      return await confirmGuardian(client, {
        ...args,
        contracts: client.contracts,
      });
    },
    removeGuardian: async (args: Omit<RemoveGuardianArgs, "contracts">) => {
      return await removeGuardian(client, {
        ...args,
        contracts: client.contracts,
      });
    },
    removePasskey: async (args: Omit<RemovePasskeyArgs, "contracts">) => {
      return await removePasskey(client, {
        ...args,
        contracts: client.contracts,
      });
    },
  };
}
