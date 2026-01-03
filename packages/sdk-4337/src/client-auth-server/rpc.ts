import type { Address, Chain, Hash, PublicRpcSchema, RpcSchema as RpcSchemaGeneric, WalletRpcSchema } from "viem";

import type { SessionRequiredContracts } from "../client/index.js";
import type { Message } from "../communicator/index.js";
import type { SerializedEthereumRpcError } from "../errors/index.js";
import type { AppMetadata, RequestArguments } from "./interface.js";
import type { SessionPreferences } from "./session/index.js";
import type { SessionConfigJSON } from "./session/utils.js";

export type AuthServerRpcSchema = [
  {
    Method: "eth_requestAccounts";
    Parameters: {
      metadata: AppMetadata;
      sessionPreferences?: SessionPreferences;
      paymaster?: Address;
    };
    ReturnType: {
      account: {
        address: Address;
        activeChainId: Chain["id"];
        session?: {
          sessionKey: Hash;
          sessionConfig: SessionConfigJSON;
        };
      };
      chainsInfo: {
        id: Chain["id"];
        capabilities: Record<string, unknown>;
        contracts: SessionRequiredContracts;
        bundlerUrl?: string;
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
    paymaster?: Address; // ERC-4337 paymaster metadata (not part of transaction params)
  };
}

export interface RPCResponseMessage<T = unknown> extends Message {
  requestId: NonNullable<Message["requestId"]>;
  content: {
    result?: T; // For successful responses
    error?: SerializedEthereumRpcError; // For error responses
  };
}
