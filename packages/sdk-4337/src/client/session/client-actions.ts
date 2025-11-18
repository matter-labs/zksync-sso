import type {
  Account,
  Address,
  Chain,
  Client,
  Hash,
  Transport,
} from "viem";
import type { BundlerClient } from "viem/account-abstraction";
import { encode_create_session_call_data } from "zksync-sso-web-sdk/bundler";

import {
  type SmartAccountClientActions,
  smartAccountClientActions,
} from "../common/smart-account-client-actions.js";
import { toSessionSmartAccount, type ToSessionSmartAccountParams } from "./account.js";
import type { SessionSpec } from "./types.js";
import { sessionSpecToJSON } from "./utils.js";

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
};

/**
 * Session client actions extend generic smart account actions but:
 *  - Remove addPasskey (not applicable for pure session key accounts)
 *  - Add createSession (send UserOp to create a session spec on-chain)
 *  - Add getSessionState (query on-chain session state)
 *  - Add sendSessionTransaction (alias of sendTransaction for clarity)
 */
export type SessionClientActions<
  TChain extends Chain = Chain,
  TAccount extends Account = Account,
> = Omit<SmartAccountClientActions<TChain, TAccount>, "addPasskey"> & {
  /** Create a session on-chain using the provided specification */
  createSession: (params: { sessionSpec: SessionSpec; contracts: { sessionValidator: Address } }) => Promise<Hash>;
};

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
  });

  return {
    ...base,
    // Explicit session creation via validator module
    createSession: async (params: { sessionSpec: SessionSpec; contracts: { sessionValidator: Address } }) => {
      // Build smart account instance (lazy, not cached here; acceptable overhead for now)
      const smartAccount = await toSessionSmartAccount(config.sessionAccount);
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
    // Remove addPasskey (not valid for session-only signing)
    addPasskey: undefined as never,
  } as SessionClientActions<TChain, TAccount extends Account ? TAccount : Account>;
}
