import {
  type Account,
  type Address,
  type Chain,
  type Client,
  type Hash,
  type Hex,
  type Transport,
} from "viem";
import type { BundlerClient } from "viem/account-abstraction";
import { encode_create_session_call_data } from "zksync-sso-web-sdk/bundler";

import {
  type SmartAccountClientActions,
  smartAccountClientActions,
} from "../common/smart-account-client-actions.js";
import type { SessionSpec } from "../session/types.js";
import { sessionSpecToJSON } from "../session/utils.js";
import { toPasskeySmartAccount, type ToPasskeySmartAccountParams } from "./account.js";

/**
 * Extended client type that includes bundler-specific configuration
 */
export type PasskeyClientData<
  TTransport extends Transport = Transport,
  TChain extends Chain = Chain,
> = {
  bundler: BundlerClient;
  passkeyAccount: ToPasskeySmartAccountParams<TTransport, TChain>;
  accountAddress: Address;
  validatorAddress: Address;
};

/**
 * Wallet actions type for passkey client
 * Extends generic smart account actions with passkey-specific overrides
 */
export type PasskeyClientActions<TChain extends Chain = Chain, TAccount extends Account = Account> = Omit<SmartAccountClientActions<TChain, TAccount>, "addPasskey"> & {
  /**
   * Add a passkey to the smart account.
   * Returns the transaction hash.
   * The webauthnValidatorAddress is automatically injected from the client configuration.
   */
  addPasskey: (params: {
    credentialId: Hex;
    publicKey: { x: Hex; y: Hex };
    originDomain: string;
  }) => Promise<Hash>;

  /**
   * Create a session on-chain using the provided specification
   */
  createSession: (params: { sessionSpec: SessionSpec; contracts: { sessionValidator: Address } }) => Promise<Hash>;
};

/**
 * Decorator that provides wallet actions for passkey client.
 * Overrides standard wallet methods to use ERC-4337 bundler under the hood.
 * Includes passkey-specific actions like addPasskey.
 */
export function passkeyClientActions<
  TTransport extends Transport,
  TChain extends Chain,
  TAccount extends Account | undefined = undefined,
>(
  config: PasskeyClientData<TTransport, TChain> & {
    client: Client<TTransport, TChain, TAccount>;
  },
): PasskeyClientActions<TChain, TAccount extends Account ? TAccount : Account> {
  // Get generic smart account actions
  const baseActions = smartAccountClientActions<TTransport, TChain, TAccount>({
    bundler: config.bundler,
    accountFactory: () => toPasskeySmartAccount(config.passkeyAccount),
    client: config.client,
    accountAddress: config.accountAddress,
  });

  // Return base actions with passkey-specific overrides
  return {
    ...baseActions,

    // Override addPasskey to automatically inject validator address
    addPasskey: async (params: {
      credentialId: Hex;
      publicKey: { x: Hex; y: Hex };
      originDomain: string;
    }) => {
      // Call base implementation with validator address injected
      return baseActions.addPasskey({
        ...params,
        webauthnValidatorAddress: config.validatorAddress,
      });
    },

    // Create a session on-chain using the provided specification
    createSession: async (params: { sessionSpec: SessionSpec; contracts: { sessionValidator: Address } }) => {
      // Build smart account instance (lazy, not cached here; acceptable overhead for now)
      const smartAccount = await toPasskeySmartAccount(config.passkeyAccount);
      const sessionSpecJSON = sessionSpecToJSON(params.sessionSpec);
      const callData = encode_create_session_call_data(sessionSpecJSON) as unknown as `0x${string}`;
      const userOpHash = await config.bundler.sendUserOperation({
        account: smartAccount,
        calls: [
          {
            to: params.contracts.sessionValidator,
            data: callData,
            value: 0n,
          },
        ],
      });
      return userOpHash;
    },
  };
}
