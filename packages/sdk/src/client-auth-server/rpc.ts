import type { Address, Hash, PublicRpcSchema, RpcSchema as RpcSchemaGeneric, WalletRpcSchema } from "viem";

import type { SessionRequiredContracts } from "../client/index.js";
import type { Message } from "../communicator/index.js";
import type { SerializedEthereumRpcError } from "../errors/index.js";
import type { AppMetadata, RequestArguments, SessionPreferences } from "./interface.js";

export type AuthServerRpcSchema = [
  {
    Method: "eth_requestAccounts";
    Parameters: {
      metadata: AppMetadata;
      session?: SessionPreferences;
    };
    ReturnType: {
      account: {
        address: Address;
        activeChainId: number;
        session?: SessionPreferences & { sessionKey: Hash };
      };
      chainsInfo: {
        id: number;
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