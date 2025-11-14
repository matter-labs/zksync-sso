import type {
  Account,
  Address,
  Chain,
  Client,
  Transport,
} from "viem";
import type { BundlerClient } from "viem/account-abstraction";

import {
  type SmartAccountClientActions,
  smartAccountClientActions,
} from "../common/smart-account-client-actions.js";
import { toEcdsaSmartAccount, type ToEcdsaSmartAccountParams } from "./account.js";

/**
 * Extended client type that includes bundler-specific configuration for ECDSA
 */
export type EcdsaClientData<
  TTransport extends Transport = Transport,
  TChain extends Chain = Chain,
> = {
  bundler: BundlerClient;
  ecdsaAccount: ToEcdsaSmartAccountParams<TTransport, TChain>;
  accountAddress: Address;
};

/**
 * Wallet actions type for ECDSA client
 * Uses generic smart account actions (no ECDSA-specific actions needed)
 */
export type EcdsaClientActions<TChain extends Chain = Chain, TAccount extends Account = Account> = SmartAccountClientActions<TChain, TAccount>;

/**
 * Decorator that provides wallet actions for ECDSA client.
 * Overrides standard wallet methods to use ERC-4337 bundler under the hood.
 */
export function ecdsaClientActions<
  TTransport extends Transport,
  TChain extends Chain,
  TAccount extends Account | undefined = undefined,
>(
  config: EcdsaClientData<TTransport, TChain> & {
    client: Client<TTransport, TChain, TAccount>;
  },
): EcdsaClientActions<TChain, TAccount extends Account ? TAccount : Account> {
  // Return generic smart account actions
  return smartAccountClientActions<TTransport, TChain, TAccount>({
    bundler: config.bundler,
    accountFactory: () => toEcdsaSmartAccount(config.ecdsaAccount),
    client: config.client,
    accountAddress: config.accountAddress,
  });
}
