import { type Account, type Address, type Chain, type Client, createClient, getAddress, type Prettify, type PublicActions, publicActions, type PublicRpcSchema, type RpcSchema, type Transport, type WalletActions, walletActions, type WalletClientConfig, type WalletRpcSchema } from "viem";
import { eip712WalletActions } from "viem/zksync";

import type { CustomPaymasterHandler } from "../../paymaster/index.js";
import { toEcdsaAccount } from "./account.js";
import { type ZksyncSsoEcdsaActions, zksyncSsoEcdsaActions } from "./decorators/ecdsa.js";
import { zksyncSsoEcdsaWalletActions } from "./decorators/wallet.js";
import type { Signer } from "./types.js";

export async function createZksyncEcdsaClient<
  transport extends Transport,
  chain extends Chain,
  rpcSchema extends RpcSchema | undefined = undefined,
>(_parameters: ZksyncSsoEcdsaClientConfig<transport, chain, rpcSchema>): Promise<ZksyncSsoEcdsaClient<transport, chain, rpcSchema>> {
  type WalletClientParameters = typeof _parameters;
  const parameters: WalletClientParameters & {
    key: NonNullable<WalletClientParameters["key"]>;
    name: NonNullable<WalletClientParameters["name"]>;
  } = {
    ..._parameters,
    address: getAddress(_parameters.address),
    key: _parameters.key || "zksync-sso-ecdsa-wallet",
    name: _parameters.name || "ZKsync SSO ECDSA Client",
  };

  const account = await toEcdsaAccount({
    address: parameters.address,
    owner: parameters.owner,
  });

  const client = createClient<transport, chain, Account, rpcSchema>({
    ...parameters,
    account,
    type: "walletClient",
  })
    .extend(() => ({
      contracts: parameters.contracts,
    }))
    .extend(publicActions)
    .extend(walletActions)
    .extend(eip712WalletActions())
    .extend(zksyncSsoEcdsaActions)
    .extend(zksyncSsoEcdsaWalletActions);
  return client;
}

export type EcdsaRequiredContracts = {
  session: Address; // Session, spend limit, etc.
  accountFactory?: Address; // For account creation
};

type ZksyncSsoEcdsaData = {
  contracts: EcdsaRequiredContracts;
  paymasterHandler?: CustomPaymasterHandler;
};

export type ClientWithZksyncSsoEcdsaData<
  transport extends Transport = Transport,
  chain extends Chain = Chain,
> = Client<transport, chain, Account> & ZksyncSsoEcdsaData;

export type ZksyncSsoEcdsaClient<
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
    PublicActions<transport, chain, account> & WalletActions<chain, account> & ZksyncSsoEcdsaActions
  > & ZksyncSsoEcdsaData
>;

export interface ZksyncSsoEcdsaClientConfig<
  transport extends Transport = Transport,
  chain extends Chain = Chain,
  rpcSchema extends RpcSchema | undefined = undefined,
> extends Omit<WalletClientConfig<transport, chain, Account, rpcSchema>, "account"> {
  chain: NonNullable<chain>;
  address: Address;
  owner: Signer;
  contracts: EcdsaRequiredContracts;
  key?: string;
  name?: string;
}
