import type { Address, Chain, Hash, PublicRpcSchema, RpcSchema as RpcSchemaGeneric, WalletRpcSchema } from "viem";

import type { SessionPreferences } from "../client/session/index.js";
import type { SessionSpecJSON } from "../client/session/utils.js";
import type { Message } from "../communicator/index.js";
import type { AppMetadata } from "../communicator/interface.js";
import type { SerializedEthereumRpcError } from "../errors/index.js";

export interface RequestArguments<M extends Method<TSchema>, TSchema extends RpcSchemaGeneric = RpcSchema> {
  readonly method: M;
  readonly params?: ExtractParams<M, TSchema>;
}

export type SessionRequiredContracts = {
  webauthnValidator: Address;
  sessionValidator: Address;
};

export type AuthServerRpcSchema = [
  {
    Method: "eth_requestAccounts";
    Parameters: {
      metadata: AppMetadata;
      sessionPreferences?: SessionPreferences;
    };
    ReturnType: {
      account: {
        address: Address;
        activeChainId: Chain["id"];
        session?: {
          sessionKey: Hash;
          sessionConfig: SessionSpecJSON;
        };
      };
      chainsInfo: {
        id: Chain["id"];
        capabilities: Record<string, unknown>;
        contracts: SessionRequiredContracts;
      }[];
    };
  },
];

export type RpcSchema = WalletRpcSchema | PublicRpcSchema | AuthServerRpcSchema;

export type Method<TSchema extends RpcSchemaGeneric = RpcSchema> = TSchema[number]["Method"];

export type ExtractParams<
  TMethod extends Method<TSchema>,
  TSchema extends RpcSchemaGeneric = RpcSchema,
> = TSchema extends Array<infer T>
  ? T extends { Method: TMethod; Parameters: infer R }
    ? R
    : never
  : never;

export type ExtractReturnType<
  TMethod extends Method<TSchema>,
  TSchema extends RpcSchemaGeneric = RpcSchema,
> = TSchema extends Array<infer T>
  ? T extends { Method: TMethod; ReturnType: infer R }
    ? R
    : never
  : never;

export interface RPCRequestMessage<
  TMethod extends Method<TSchema>,
  TSchema extends RpcSchemaGeneric = RpcSchema,
> extends Message {
  content: {
    action: RequestArguments<TMethod, TSchema>;
    chainId: number;
  };
}

export interface RPCResponseMessage<T = unknown> extends Message {
  requestId: NonNullable<Message["requestId"]>;
  content: {
    result?: T; // For successful responses
    error?: SerializedEthereumRpcError; // For error responses
  };
}
