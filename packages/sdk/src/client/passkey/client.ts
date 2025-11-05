import { type PublicKeyCredentialDescriptorJSON } from "@simplewebauthn/browser";
import { type Account, type Address, type Chain, type Client, createClient, getAddress, type Prettify, type PublicActions, publicActions, type PublicRpcSchema, type RpcSchema, type Transport, type WalletActions, walletActions, type WalletClientConfig, type WalletRpcSchema } from "viem";
import { erc7739Actions } from "viem/experimental";
import { eip712WalletActions } from "viem/zksync";

import type { CustomPaymasterHandler } from "../../paymaster/index.js";
import { passkeyHashSignatureResponseFormat } from "../../utils/passkey.js";
import { toPasskeyAccount } from "./account.js";
import { requestPasskeyAuthentication } from "./actions/passkey.js";
import { type ZksyncSsoPasskeyActions, zksyncSsoPasskeyActions } from "./decorators/passkey.js";
import { zksyncSsoPasskeyWalletActions } from "./decorators/wallet.js";
import { type WebSdkActions, webSdkActions } from "./decorators/webSdk.js";

export function createZksyncPasskeyClient<
  transport extends Transport,
  chain extends Chain,
  rpcSchema extends RpcSchema | undefined = undefined,
>(_parameters: ZksyncSsoPasskeyClientConfig<transport, chain, rpcSchema>): ZksyncSsoPasskeyClient<transport, chain, rpcSchema> {
  type WalletClientParameters = typeof _parameters;
  const parameters: WalletClientParameters & {
    key: NonNullable<WalletClientParameters["key"]>;
    name: NonNullable<WalletClientParameters["name"]>;
  } = {
    ..._parameters,
    address: getAddress(_parameters.address),
    key: _parameters.key || "zksync-sso-passkey-wallet",
    name: _parameters.name || "ZKsync SSO Passkey Client",
  };

  const account = toPasskeyAccount({
    address: parameters.address,
    chain: parameters.chain,
    contracts: parameters.contracts,
    transport: parameters.transport,
    sign: async ({ hash }) => {
      const passkeySignature = await requestPasskeyAuthentication({
        challenge: hash,
        credentialPublicKey: parameters.credentialPublicKey,
        credential: parameters.credential,
      });

      return passkeyHashSignatureResponseFormat(
        passkeySignature.passkeyAuthenticationResponse.id,
        passkeySignature.passkeyAuthenticationResponse.response,
        parameters.contracts,
      );
    },
  });
  const client = createClient<transport, chain, Account, rpcSchema>({
    ...parameters,
    account,
    type: "walletClient",
  })
    .extend(() => ({
      credentialPublicKey: parameters.credentialPublicKey,
      userName: parameters.userName,
      userDisplayName: parameters.userDisplayName,
      contracts: parameters.contracts,
      credential: parameters.credential,
      // NEW: Add web-SDK configuration
      bundlerUrl: parameters.bundlerUrl,
      entryPointAddress: parameters.entryPointAddress,
    }))
    .extend(publicActions)
    .extend(walletActions)
    .extend(eip712WalletActions())
    .extend(zksyncSsoPasskeyActions)
    .extend(zksyncSsoPasskeyWalletActions)
    .extend(webSdkActions) // NEW: Add Web-SDK transaction actions
    .extend(erc7739Actions({
      verifier: account.address,
    }))
    .extend(webSdkActions); // NEW: Add Web-SDK transaction actions
  return client;
}

export type PasskeyRequiredContracts = {
  oidcKeyRegistry: Address; // Oidc key registry
  session: Address; // Session, spend limit, etc.
  passkey: Address; // Validator for passkey signature
  recovery: Address; // Validator for account recovery
  recoveryOidc: Address; // Validator for account recovery with OIDC
  accountFactory?: Address; // For account creation
};
type ZksyncSsoPasskeyData = {
  credentialPublicKey: Uint8Array; // Public key of the passkey
  userName: string; // Basically unique user id (which is called `userName` in webauthn)
  userDisplayName: string; // Also option required for webauthn
  contracts: PasskeyRequiredContracts;
  paymasterHandler?: CustomPaymasterHandler;
  // NEW: Web-SDK / ERC-4337 configuration
  bundlerUrl?: string; // Bundler URL for ERC-4337 UserOperations
  entryPointAddress?: Address; // EntryPoint contract address
  credential?: PublicKeyCredentialDescriptorJSON; // Credential ID for passkey
};

export type ClientWithZksyncSsoPasskeyData<
  transport extends Transport = Transport,
  chain extends Chain = Chain,
> = Client<transport, chain, Account> & ZksyncSsoPasskeyData;

export type ZksyncSsoPasskeyClient<
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
    PublicActions<transport, chain, account> & WalletActions<chain, account> & ZksyncSsoPasskeyActions & WebSdkActions<chain, account>
  > & ZksyncSsoPasskeyData
>;

export interface ZksyncSsoPasskeyClientConfig<
  transport extends Transport = Transport,
  chain extends Chain = Chain,
  rpcSchema extends RpcSchema | undefined = undefined,
> extends Omit<WalletClientConfig<transport, chain, Account, rpcSchema>, "account"> {
  chain: NonNullable<chain>;
  address: Address;
  credentialPublicKey: Uint8Array;
  userName: string;
  userDisplayName: string;
  contracts: PasskeyRequiredContracts;
  credential?: PublicKeyCredentialDescriptorJSON;
  key?: string;
  name?: string;
  // NEW: Web-SDK / ERC-4337 configuration (optional)
  bundlerUrl?: string;
  entryPointAddress?: Address;
}
