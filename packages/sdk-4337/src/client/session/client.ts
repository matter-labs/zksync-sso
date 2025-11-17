import {
  type Address,
  type Chain,
  type Client,
  createClient,
  createPublicClient,
  type Hash,
  type Prettify,
  type PublicActions,
  publicActions,
  type PublicRpcSchema,
  type RpcSchema,
  type Transport,
  walletActions,
  type WalletRpcSchema,
} from "viem";
import type { BundlerClient } from "viem/account-abstraction";

import type { ToSessionSmartAccountParams } from "./account.js";
import { type SessionClientActions, sessionClientActions } from "./client-actions.js";
import type { SessionSpec } from "./types.js";

/** Parameters for creating a Session smart account client */
export type CreateSessionClientParams<
  TTransport extends Transport = Transport,
  TChain extends Chain = Chain,
> = {
  /** Smart account address (already deployed) */
  address: Address;
  /** Session validator contract address */
  sessionValidatorAddress: Address;
  /** Session key private key */
  sessionKeyPrivateKey: Hash;
  /** Session specification governing allowed actions */
  sessionSpec: SessionSpec;
  /** Bundler client instance */
  bundlerClient: BundlerClient;
  /** Chain config */
  chain: TChain;
  /** Transport for public RPC */
  transport: TTransport;
  /** Optional timestamp override for signature generation */
  currentTimestamp?: bigint;
  /** Optional client metadata */
  key?: string;
  name?: string;
};

/** Session client type with actions */
export type SessionClient<
  TTransport extends Transport = Transport,
  TChain extends Chain = Chain,
  TRpcSchema extends RpcSchema | undefined = undefined,
> = Prettify<
  Client<
    TTransport,
    TChain,
    undefined,
    TRpcSchema extends RpcSchema
      ? [...PublicRpcSchema, ...WalletRpcSchema, ...TRpcSchema]
      : [...PublicRpcSchema, ...WalletRpcSchema]
  > &
  PublicActions<TTransport, TChain> &
  SessionClientActions & {
    bundler: BundlerClient;
  }
>;

/**
 * Create a client wrapping a session smart account. Provides wallet-like API
 * plus helpers for session creation & state queries. Transactions sent via
 * sendTransaction or sendSessionTransaction are encoded as session execute() calls.
 */
export function createSessionClient<
  TTransport extends Transport = Transport,
  TChain extends Chain = Chain,
  TRpcSchema extends RpcSchema | undefined = undefined,
>(
  params: CreateSessionClientParams<TTransport, TChain>,
): SessionClient<TTransport, TChain, TRpcSchema> {
  const {
    address,
    sessionValidatorAddress,
    sessionKeyPrivateKey,
    sessionSpec,
    bundlerClient,
    chain,
    transport,
    currentTimestamp,
  } = params;

  const publicClient = createPublicClient({ chain, transport });

  const sessionAccountParams: ToSessionSmartAccountParams = {
    client: publicClient as ToSessionSmartAccountParams["client"],
    address,
    sessionValidatorAddress,
    sessionKeyPrivateKey,
    sessionSpec,
    currentTimestamp,
  };

  const client = createClient({
    chain,
    transport,
    account: undefined,
    type: "walletClient",
    key: params.key || "zksync-sso-session-client",
    name: params.name || "ZKsync SSO Session Client",
  })
    .extend(publicActions)
    .extend(walletActions)
    .extend((client) =>
      sessionClientActions({
        client,
        bundler: bundlerClient,
        sessionAccount: sessionAccountParams,
        accountAddress: address,
      }),
    )
    .extend(() => ({
      bundler: bundlerClient,
    }));

  return client as SessionClient<TTransport, TChain, TRpcSchema>;
}
