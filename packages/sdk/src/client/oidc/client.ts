import {
  type Account,
  type Address,
  type Chain,
  type Client,
  createClient,
  encodeAbiParameters,
  getAddress,
  type Hex,
  type Prettify,
  publicActions,
  type PublicRpcSchema,
  type RpcSchema,
  type Transport,
  type WalletClientConfig,
  type WalletRpcSchema,
} from "viem";

import { type ZKsyncSsoWalletActions, zksyncSsoWalletActions } from "../recovery/decorators/wallet.js";
import type { RecoveryRequiredContracts } from "../recovery/index.js";
import { type OidcAccount, toOidcAccount, type ZkProof } from "./account.js";
import { zksyncSsoRecoveryActions } from "./actions/index.js";
import type { ZksyncSsoOidcActions } from "./decorators/actions.js";

export const signOidcTransaction = (
  recoveryValidatorAddress: Address,
  _hash: Hex,
  proof: ZkProof,
) => {
  if (proof.public.length !== 151) {
    throw new Error("Should be 151 elements long");
  }

  const encodedProof = encodeAbiParameters(
    [
      {
        type: "tuple",
        components: [
          { name: "piA", type: "uint256[2]" },
          { name: "piB", type: "uint256[2][2]" },
          { name: "piC", type: "uint256[2]" },
        ],
      },
      {
        type: "uint256[151]",
        name: "public",
      },
    ],
    [
      {
        piA: [BigInt(proof.groth16Proof.pi_a[0]), BigInt(proof.groth16Proof.pi_a[1])],
        piB: [
          // Order here is inverted because that's what the verifier expects TODO better explanation
          [BigInt(proof.groth16Proof.pi_b[0][1]), BigInt(proof.groth16Proof.pi_b[0][0])],
          [BigInt(proof.groth16Proof.pi_b[1][1]), BigInt(proof.groth16Proof.pi_b[1][0])],
        ],
        piC: [BigInt(proof.groth16Proof.pi_c[0]), BigInt(proof.groth16Proof.pi_c[1])],
      },
      proof.public,
    ],
  );

  return encodeAbiParameters(
    [
      { type: "bytes", name: "signature" },
      { type: "address", name: "recoveryContract" },
      { type: "bytes", name: "validatorData" },
    ],
    [
      encodedProof,
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

export type ZKsyncSsoOidcData = {
  contracts: OidcRequiredContracts;
};

export type ZkSyncSsoClient<
  transport extends Transport = Transport,
  chain extends Chain = Chain,
  rpcSchema extends RpcSchema | undefined = undefined,
> = Prettify<
  Client<
    transport,
    chain,
    OidcAccount,
    rpcSchema extends RpcSchema
      ? [...PublicRpcSchema, ...WalletRpcSchema, ...rpcSchema]
      : [...PublicRpcSchema, ...WalletRpcSchema],
    ZKsyncSsoWalletActions<chain, OidcAccount> & ZksyncSsoOidcActions
  > & ZKsyncSsoOidcData
>;

export type ClientWithOidcData<
  transport extends Transport = Transport,
  chain extends Chain = Chain,
  account extends Account = Account,
> = Client<transport, chain, account> & ZKsyncSsoOidcData;

export function createZkSyncOidcClient<
  transport extends Transport,
  chain extends Chain,
  rpcSchema extends RpcSchema | undefined = undefined,
>(givenParams: SsoClientConfig<transport, chain, rpcSchema>): ZkSyncSsoClient<
  transport,
  chain,
  rpcSchema
> {
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
    signTransaction: async ({ hash, proof }) => {
      return signOidcTransaction(parameters.contracts.recoveryOidc, hash, proof);
    },
  });

  return createClient<transport, chain, OidcAccount, rpcSchema>({
    ...parameters,
    account,
    type: "walletClient",
  })
    .extend(() => ({
      contracts: parameters.contracts,
    }))
    .extend(publicActions)
    .extend(zksyncSsoWalletActions)
    .extend(zksyncSsoRecoveryActions);
}
