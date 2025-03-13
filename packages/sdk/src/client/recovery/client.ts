import { type Account, type Address, type Chain, type Client, createClient, encodeAbiParameters, getAddress, type Prettify, publicActions, type PublicRpcSchema, type RpcSchema, type Transport, type WalletClientConfig, type WalletRpcSchema } from "viem";

import { toRecoveryAccount } from "./account.js";
import { publicActionsRewrite } from "./decorators/publicActionsRewrite.js";
import { type ZksyncSsoRecoveryActions, zksyncSsoRecoveryActions } from "./decorators/recovery.js";
import { type ZksyncSsoWalletActions, zksyncSsoWalletActions } from "./decorators/wallet.js";

export const signRecoveryTransaction = (recoveryValidatorAddress: `0x${string}`) => {
  return encodeAbiParameters(
    [
      { type: "bytes", name: "unusedSignedHash" },
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

export function createZksyncRecoveryGuardianClient<
  transport extends Transport,
  chain extends Chain,
  rpcSchema extends RpcSchema | undefined = undefined,
>(_parameters: ZksyncSsoRecoveryClientConfig<transport, chain, rpcSchema>): ZksyncSsoRecoveryClient<transport, chain, rpcSchema> {
  type WalletClientParameters = typeof _parameters;
  const parameters: WalletClientParameters & {
    key: NonNullable<WalletClientParameters["key"]>;
    name: NonNullable<WalletClientParameters["name"]>;
  } = {
    ..._parameters,
    address: getAddress(_parameters.address),
    key: _parameters.key || "zksync-sso-recovery-wallet",
    name: _parameters.name || "ZKsync SSO Recovery Client",
  };

  const account = toRecoveryAccount({
    address: parameters.address,
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    signTransaction: async ({ hash: _ }) => {
      return signRecoveryTransaction(parameters.contracts.recovery);
    },
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
    .extend(publicActionsRewrite)
    .extend(zksyncSsoWalletActions)
    .extend(zksyncSsoRecoveryActions);
  return client;
}

export type RecoveryRequiredContracts = {
  recovery: Address; // Recovery
  passkey: Address; // Passkey
};
type ZksyncSsoRecoveryData = {
  contracts: RecoveryRequiredContracts;
};

export type ClientWithZksyncSsoRecoveryData<
  transport extends Transport = Transport,
  chain extends Chain = Chain,
  account extends Account = Account,
> = Client<transport, chain, account> & ZksyncSsoRecoveryData;

export type ZksyncSsoRecoveryClient<
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
    ZksyncSsoWalletActions<chain, account> & ZksyncSsoRecoveryActions
  > & ZksyncSsoRecoveryData
>;

export interface ZksyncSsoRecoveryClientConfig<
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
