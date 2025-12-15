import {
  type Address,
  type Chain,
  type Client,
  createClient,
  createPublicClient,
  type Hex,
  type JsonRpcAccount,
  type Prettify,
  type PublicActions,
  publicActions,
  type PublicRpcSchema,
  type RpcSchema,
  type Transport,
  walletActions,
  type WalletRpcSchema,
} from "viem";
import type { BundlerClient } from "viem/account-abstraction";

import { createPaymasterBundlerClient } from "../common/paymaster-middleware.js";
import type { ToPasskeySmartAccountParams } from "./account.js";
import {
  type PasskeyClientActions,
  passkeyClientActions,
} from "./client-actions.js";

/**
 * Parameters for creating a passkey bundler client
 */
export type CreatePasskeyClientParams<
  TTransport extends Transport = Transport,
  TChain extends Chain = Chain,
> = {
  /** Passkey account configuration */
  account: {
    /** Smart account address (required - no counterfactual support). */
    address: Address;
    /** Passkey validator contract address (required for signature formatting). */
    validatorAddress: Address;
    /** Passkey credential ID (hex string). */
    credentialId: Hex;
    /** Relying Party ID (domain where passkey was created). */
    rpId: string;
    /** Origin URL (for WebAuthn verification). */
    origin: string;
    /** Optional override for EntryPoint address used by the account implementation. */
    entryPointAddress?: Address;
  };

  /** Bundler client instance (created externally by user) */
  bundlerClient: BundlerClient;

  /** Chain configuration */
  chain: TChain;

  /** Transport for public RPC calls */
  transport: TTransport;

  /** Optional paymaster address for sponsored transactions */
  paymaster?: Address;

  /** Optional client metadata */
  key?: string;
  name?: string;
};

/**
 * Passkey bundler client type with all actions
 */
export type PasskeyClient<
  TTransport extends Transport = Transport,
  TChain extends Chain = Chain,
  TRpcSchema extends RpcSchema | undefined = undefined,
> = Prettify<
  Client<
    TTransport,
    TChain,
    JsonRpcAccount,
    TRpcSchema extends RpcSchema
      ? [...PublicRpcSchema, ...WalletRpcSchema, ...TRpcSchema]
      : [...PublicRpcSchema, ...WalletRpcSchema]
  > &
  PublicActions<TTransport, TChain> &
  PasskeyClientActions & {
    /** The bundler client instance */
    bundler: BundlerClient;
  }
>;

/**
 * Create a viem wallet client wrapper around a passkey smart account.
 *
 * This client provides a standard viem WalletClient interface while using
 * ERC-4337 user operations under the hood. All transactions are sent via
 * the bundler and return actual transaction hashes (not userOp hashes).
 *
 * The smart account is lazy-loaded on first transaction since toPasskeySmartAccount is async.
 *
 * @param params - Configuration including account details, bundler client, and chain
 * @returns A viem wallet client compatible with wagmi and standard tooling
 *
 * @example
 * ```typescript
 * import { createPublicClient, http } from "viem";
 * import { createBundlerClient } from "viem/account-abstraction";
 * import { createPasskeyClient } from "zksync-sso-4337/client/passkey";
 *
 * const publicClient = createPublicClient({
 *   chain,
 *   transport: http(rpcUrl),
 * });
 *
 * const bundlerClient = createBundlerClient({
 *   client: publicClient,
 *   transport: http(bundlerUrl),
 * });
 *
 * const client = createPasskeyClient({
 *   account: {
 *     address: "0x...",
 *     validatorAddress: "0x...",
 *     credentialId: "0x...",
 *     rpId: window.location.hostname,
 *     origin: window.location.origin,
 *   },
 *   bundlerClient,
 *   chain,
 *   transport: http(rpcUrl),
 * });
 *
 * // Use like a normal wallet client
 * const hash = await client.sendTransaction({
 *   to: "0x...",
 *   value: parseEther("0.1"),
 *   data: "0x",
 * });
 *
 * // Wait for confirmation
 * const receipt = await client.waitForTransactionReceipt({ hash });
 *
 * // Add a new passkey
 * await client.addPasskey({
 *   credentialId: "0x...",
 *   publicKey: { x: "0x...", y: "0x..." },
 *   originDomain: "example.com",
 * });
 *
 * // Access bundler client directly
 * const bundler = client.bundler;
 * ```
 */
export function createPasskeyClient<
  TTransport extends Transport = Transport,
  TChain extends Chain = Chain,
  TRpcSchema extends RpcSchema | undefined = undefined,
>(
  params: CreatePasskeyClientParams<TTransport, TChain>,
): PasskeyClient<TTransport, TChain, TRpcSchema> {
  const { account: accountConfig, bundlerClient, chain, transport, paymaster } = params;

  // Wrap bundler client to inject paymaster if provided
  const wrappedBundlerClient = paymaster
    ? createPaymasterBundlerClient(bundlerClient, paymaster)
    : bundlerClient;

  // Create public client for RPC calls
  const publicClient = createPublicClient({
    chain,
    transport,
  });

  // Prepare passkey account params for lazy loading
  const passkeyAccountParams: ToPasskeySmartAccountParams = {
    client: publicClient as ToPasskeySmartAccountParams["client"],
    address: accountConfig.address,
    validatorAddress: accountConfig.validatorAddress,
    credentialId: accountConfig.credentialId,
    rpId: accountConfig.rpId,
    origin: accountConfig.origin,
    entryPointAddress: accountConfig.entryPointAddress,
  };

  // Create the client with all actions
  // Use standard pattern: extend with walletActions first, then override with custom actions
  const client = createClient({
    chain,
    transport,
    account: { address: accountConfig.address, type: "json-rpc" },
    type: "walletClient",
    key: params.key || "zksync-sso-passkey-client",
    name: params.name || "ZKsync SSO Passkey Client",
  })
    .extend(publicActions)
    .extend(walletActions)
    .extend((client) =>
      passkeyClientActions({
        client,
        bundler: wrappedBundlerClient,
        passkeyAccount: passkeyAccountParams,
        accountAddress: accountConfig.address,
        validatorAddress: accountConfig.validatorAddress,
      }),
    )
    .extend(() => ({
      bundler: wrappedBundlerClient,
    }));

  return client as PasskeyClient<TTransport, TChain, TRpcSchema>;
}
