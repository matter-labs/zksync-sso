import { type Account, type Address, type Chain, type Client, createClient, createPublicClient, encodeAbiParameters, getAddress, type Hash, type Prettify, publicActions, type PublicRpcSchema, type RpcSchema, type Transport, type WalletClientConfig, type WalletRpcSchema } from "viem";
import { privateKeyToAccount } from "viem/accounts";
import { zksyncInMemoryNode } from "viem/chains";

import { encodeSessionTx } from "../../utils/encoding.js";
import type { SessionConfig } from "../../utils/session.js";
import { publicActionsRewrite } from "../decorators/publicActionsRewrite.js";
import { type ZksyncSsoSessionActions, zksyncSsoSessionActions } from "../decorators/session.js";
import { type ZksyncSsoWalletActions, zksyncSsoWalletActions } from "../decorators/session_wallet.js";
import { toSmartAccount } from "../session-smart-account.js";

export const signSessionTransaction = (args: {
  sessionKeySignedHash: Hash;
  sessionContract: Address;
  sessionConfig: SessionConfig;
  to: Address;
  callData?: Hash;
  timestamp?: bigint;
}) => {
  return encodeAbiParameters(
    [
      { type: "bytes", name: "sessionKeySignedHash" },
      { type: "address", name: "sessionContract" },
      { type: "bytes[]", name: "hooks" },
    ],
    [
      args.sessionKeySignedHash,
      args.sessionContract,
      [
        encodeSessionTx({
          sessionConfig: args.sessionConfig,
          to: args.to,
          callData: args.callData,
          timestamp: args.timestamp,
        }),
        // TODO: this is assuming there are no other hooks
      ],
    ],
  );
};

export function createZksyncSessionClient<
  transport extends Transport,
  chain extends Chain,
  rpcSchema extends RpcSchema | undefined = undefined,
>(_parameters: ZksyncSsoSessionClientConfig<transport, chain, rpcSchema>): ZksyncSsoSessionClient<transport, chain, rpcSchema> {
  type WalletClientParameters = typeof _parameters;
  const parameters: WalletClientParameters & {
    key: NonNullable<WalletClientParameters["key"]>;
    name: NonNullable<WalletClientParameters["name"]>;
  } = {
    ..._parameters,
    address: getAddress(_parameters.address),
    key: _parameters.key || "zksync-sso-session-wallet",
    name: _parameters.name || "ZKsync SSO Session Client",
  };

  const getInMemoryNodeTimestamp = async () => {
    const publicClient = createPublicClient({ chain: parameters.chain, transport: parameters.transport });
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const timestamp: number = await publicClient.request({ method: "config_getCurrentTimestamp" as any });
    return BigInt(timestamp);
  };

  const account = toSmartAccount({
    address: parameters.address,
    signTransaction: async ({ hash, to, callData }) => {
      // In Memory Node uses a different timestamp mechanism which isn't equal to actual timestamp
      const timestamp = parameters.chain.id === zksyncInMemoryNode.id
        ? await getInMemoryNodeTimestamp()
        : undefined;

      const sessionKeySigner = privateKeyToAccount(parameters.sessionKey);
      const hashSignature = await sessionKeySigner.sign({ hash });

      return signSessionTransaction({
        sessionKeySignedHash: hashSignature,
        sessionContract: parameters.contracts.session,
        sessionConfig: parameters.sessionConfig,
        to,
        callData,
        timestamp,
      });
    },
  });
  const client = createClient<transport, chain, Account, rpcSchema>({
    ...parameters,
    account,
    type: "walletClient",
  })
    .extend(() => ({
      sessionKey: parameters.sessionKey,
      sessionConfig: parameters.sessionConfig,
      contracts: parameters.contracts,
    }))
    .extend(publicActions)
    .extend(publicActionsRewrite)
    .extend(zksyncSsoWalletActions)
    .extend(zksyncSsoSessionActions);
  return client;
}

export type SessionRequiredContracts = {
  session: Address; // Session, spend limit, etc.
};
type ZksyncSsoSessionData = {
  sessionKey: Hash;
  sessionConfig: SessionConfig;
  contracts: SessionRequiredContracts;
};

export type ClientWithZksyncSsoSessionData<
  transport extends Transport = Transport,
  chain extends Chain = Chain,
  account extends Account = Account,
> = Client<transport, chain, account> & ZksyncSsoSessionData;

export type ZksyncSsoSessionClient<
  transport extends Transport = Transport,
  chain extends Chain = Chain,
  rpcSchema extends RpcSchema | undefined = undefined,
  account extends Account = Account,
> = Prettify<
  Client<
    transport,
    chain,
    account,
    rpcSchema extends RpcSchema
      ? [...PublicRpcSchema, ...WalletRpcSchema, ...rpcSchema]
      : [...PublicRpcSchema, ...WalletRpcSchema],
    ZksyncSsoWalletActions<chain, account> & ZksyncSsoSessionActions
  > & ZksyncSsoSessionData
>;

export interface ZksyncSsoSessionClientConfig<
  transport extends Transport = Transport,
  chain extends Chain = Chain,
  rpcSchema extends RpcSchema | undefined = undefined,
> extends Omit<WalletClientConfig<transport, chain, Account, rpcSchema>, "account"> {
  chain: NonNullable<chain>;
  address: Address;
  sessionKey: Hash;
  sessionConfig: SessionConfig;
  contracts: SessionRequiredContracts;
  key?: string;
  name?: string;
}
