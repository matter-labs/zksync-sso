import type {
  Account,
  Address,
  Chain,
  Client,
  Transport,
} from "viem";
import type { BundlerClient } from "viem/account-abstraction";

import type { PaymasterConfig } from "../../actions/sendUserOperation.js";
import {
  type SmartAccountClientActions,
  smartAccountClientActions,
} from "../common/smart-account-client-actions.js";
import { toSessionSmartAccount, type ToSessionSmartAccountParams } from "./account.js";

/**
 * Extended client data needed for session smart account wrapper
 */
export type SessionClientData<
  TTransport extends Transport = Transport,
  TChain extends Chain = Chain,
> = {
  bundler: BundlerClient;
  sessionAccount: ToSessionSmartAccountParams<TTransport, TChain>;
  accountAddress: Address;
  paymaster?: PaymasterConfig;
};

/**
 * Session client actions extend generic smart account actions but:
 *  - Remove addPasskey (not applicable for pure session key accounts)
 */
export type SessionClientActions<
  TChain extends Chain = Chain,
  TAccount extends Account = Account,
> = Omit<SmartAccountClientActions<TChain, TAccount>, "addPasskey">;

/**
 * Decorator providing session-specific client actions
 */
export function sessionClientActions<
  TTransport extends Transport,
  TChain extends Chain,
  TAccount extends Account | undefined = undefined,
>(
  config: SessionClientData<TTransport, TChain> & { client: Client<TTransport, TChain, TAccount> },
): SessionClientActions<TChain, TAccount extends Account ? TAccount : Account> {
  const base = smartAccountClientActions<TTransport, TChain, TAccount>({
    bundler: config.bundler,
    accountFactory: () => toSessionSmartAccount(config.sessionAccount),
    client: config.client,
    accountAddress: config.accountAddress,
    paymaster: config.paymaster,
  });

  return {
    ...base,
    // Remove addPasskey (not valid for session-only signing)
    addPasskey: undefined as never,
  } as SessionClientActions<TChain, TAccount extends Account ? TAccount : Account>;
}
