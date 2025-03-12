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

import { type ZksyncSsoWalletActions, zksyncSsoWalletActions } from "../recovery/decorators/wallet.js";
import { type OidcAccount, toOidcAccount, type ZkProof } from "./account.js";
import { zksyncSsoRecoveryActions } from "./actions/index.js";
import type { ZksyncSsoOidcActions } from "./decorators/actions.js";

type BigintTuple<N extends number, T extends bigint[] = []> = T["length"] extends N ? T : BigintTuple<N, [...T, bigint]>;

export const signOidcTransaction = (
  recoveryValidatorAddress: Address,
  _hash: Hex,
  proof: ZkProof,
) => {
  if (proof.public.length !== 151) {
    throw new Error("public inputs should be 151 elements long");
  }
  if (proof.oidcKey.n.length !== 17) {
    throw new Error("key modulus should be 17 elements long");
  }

  const encodedProof = encodeAbiParameters(
    [
      {
        type: "tuple",
        components: [
          {
            name: "zkProof",
            type: "tuple",
            components: [
              { name: "pA", type: "uint256[2]" },
              { name: "pB", type: "uint256[2][2]" },
              { name: "pC", type: "uint256[2]" },
            ],
          },
          {
            name: "key",
            type: "tuple",
            components: [
              { name: "issHash", type: "bytes32" },
              { name: "kid", type: "bytes32" },
              { name: "n", type: "uint256[17]" },
              { name: "e", type: "bytes" },
            ],
          },
          {
            name: "merkleProof",
            type: "bytes32[]",
          },
        ],
      },
    ],
    [
      {
        zkProof: {
          pA: [BigInt(proof.groth16Proof.pi_a[0]), BigInt(proof.groth16Proof.pi_a[1])],
          pB: [
            // Order here is inverted because that's what the verifier expects TODO better explanation
            [BigInt(proof.groth16Proof.pi_b[0][1]), BigInt(proof.groth16Proof.pi_b[0][0])],
            [BigInt(proof.groth16Proof.pi_b[1][1]), BigInt(proof.groth16Proof.pi_b[1][0])],
          ],
          pC: [BigInt(proof.groth16Proof.pi_c[0]), BigInt(proof.groth16Proof.pi_c[1])],
        },
        key: {
          issHash: proof.oidcKey.issHash,
          kid: proof.oidcKey.kid,
          n: proof.oidcKey.n as BigintTuple<17>,
          e: proof.oidcKey.e,
        },
        merkleProof: proof.merkleProof,
      },
    ],
  );

  return encodeAbiParameters(
    [
      { type: "bytes", name: "signature" },
      { type: "address", name: "recoveryContract" },
      { type: "bytes[]", name: "validatorData" },
    ],
    [
      encodedProof,
      recoveryValidatorAddress,
      ["0x"],
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
  contracts: OidcRequiredContracts;
  key?: string;
  name?: string;
}

export type OidcRequiredContracts = {
  passkey: Address; // Passkey
  recovery: Address;
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
    ZksyncSsoWalletActions<chain, OidcAccount> & ZksyncSsoOidcActions
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
