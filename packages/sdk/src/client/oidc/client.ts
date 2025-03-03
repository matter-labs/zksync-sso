import {
  type Account,
  type Address,
  type Chain, type Client, createClient,
  encodeAbiParameters, getAddress,
  type Prettify, publicActions, type PublicRpcSchema,
  type RpcSchema,
  type Transport,
  type WalletClientConfig, type WalletRpcSchema,
} from "viem";

import { publicActionsRewrite } from "../recovery/decorators/publicActionsRewrite.js";
import { type ZksyncSsoWalletActions, zksyncSsoWalletActions } from "../recovery/decorators/wallet.js";
import type { RecoveryRequiredContracts } from "../recovery/index.js";
import { toOidcAccount } from "./account.js";
import { zksyncSsoRecoveryActions } from "./actions/index.js";
import type { ZksyncSsoOidcActions } from "./decorators/actions.js";

export const signOidcTransaction = (
  recoveryValidatorAddress: Address,
) => {
  return encodeAbiParameters(
    [
      { type: "bytes", name: "signature" },
      { type: "address", name: "recoveryContract" },
      { type: "bytes", name: "validatorData" },
    ],
    [
      "0x",
      recoveryValidatorAddress,
      "0x",
    ],
  );
};

export interface SsoClientConfig<
  transport extends Transport = Transport,
  chain extends Chain = Chain,
  rpcSchema extends RpcSchema | undefined = undefined,
> extends Omit<WalletClientConfig<transport, chain, Account, rpcSchema>, "account"> {
  chain: NonNullable<chain>;
  address: Address;
  contracts: RecoveryRequiredContracts;
  key?: string;
  name?: string;
}

export type OidcRequiredContracts = {
  passkey: Address; // Passkey
  recoveryOidc: Address; // Oidc
};

export type ZksynSsoOidcData = {
  contracts: OidcRequiredContracts;
};

export type ZkSyncSsoClient<
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
    ZksyncSsoWalletActions<chain, account> & ZksyncSsoOidcActions
  > & ZksynSsoOidcData
>;

export type ClientWithOidcData<
  transport extends Transport = Transport,
  chain extends Chain = Chain,
  account extends Account = Account,
> = Client<transport, chain, account> & ZksynSsoOidcData;

export function createZkSyncOidcClient<
  transport extends Transport,
  chain extends Chain,
  rpcSchema extends RpcSchema | undefined = undefined,
>(givenParams: SsoClientConfig<transport, chain, rpcSchema>) {
  type WalletClientParameters = typeof givenParams;
  const parameters: WalletClientParameters & {
    key: NonNullable<WalletClientParameters["key"]>;
    name: NonNullable<WalletClientParameters["name"]>;
  } = {
    ...givenParams,
    address: getAddress(givenParams.address),
    key: givenParams.key || "zksync-sso-oidc-wallet",
    name: givenParams.name || "ZKsync SSO OIDC Client",
  };

  const account = toOidcAccount({
    address: parameters.address,
    signTransaction: async () => {
      console.log("Signing transaction from right place");
      return signOidcTransaction(parameters.contracts.recoveryOidc);
    },
  });

  return createClient<transport, chain, Account, rpcSchema>({
    ...parameters,
    account,
    type: "walletClient",
  })
    .extend(() => ({
      contracts: parameters.contracts,
    }))
    .extend(publicActions)
    .extend(publicActionsRewrite)
    .extend(zksyncSsoWalletActions)
    .extend(zksyncSsoRecoveryActions);
}
