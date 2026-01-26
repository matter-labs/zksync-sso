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

import type { PaymasterConfig } from "../../actions/sendUserOperation.js";
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
  paymaster?: PaymasterConfig | Address;
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
  createSession: (params: { sessionSpec: SessionSpec; proof: Hex; contracts: { sessionValidator: Address } }) => Promise<Hash>;
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
    paymaster: config.paymaster,
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
    createSession: async (params: { sessionSpec: SessionSpec; proof: Hex; contracts: { sessionValidator: Address } }) => {
      // Build smart account instance (lazy, not cached here; acceptable overhead for now)
      const smartAccount = await toPasskeySmartAccount(config.passkeyAccount);
      const sessionSpecJSON = sessionSpecToJSON(params.sessionSpec);
      const callData = encode_create_session_call_data(sessionSpecJSON, params.proof) as unknown as `0x${string}`;
      // Normalize paymaster config (same logic as smartAccountClientActions)
      const normalizedPaymaster = config.paymaster
        ? typeof config.paymaster === "string"
          ? {
              address: config.paymaster as Address,
              verificationGasLimit: 500_000n,
              postOpGasLimit: 1_000_000n,
              data: "0x" as `0x${string}`,
            }
          : config.paymaster
        : undefined;

      // For v0.8 EntryPoint, pass separate paymaster fields (not packed paymasterAndData)
      const paymasterParams = normalizedPaymaster
        ? {
            paymaster: normalizedPaymaster.address,
            paymasterData: normalizedPaymaster.data ?? ("0x" as Hex),
            paymasterVerificationGasLimit: normalizedPaymaster.verificationGasLimit ?? 500_000n,
            paymasterPostOpGasLimit: normalizedPaymaster.postOpGasLimit ?? 1_000_000n,
          }
        : {};

      const userOpHash = await config.bundler.sendUserOperation({
        account: smartAccount,
        calls: [
          {
            to: params.contracts.sessionValidator,
            data: callData,
            value: 0n,
          },
        ],
        ...paymasterParams,
      });
      return userOpHash;
    },
  };
}
