import {
  type Address,
  type Chain,
  type Client,
  createClient,
  createPublicClient,
  type Hash,
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

import type { ToEcdsaSmartAccountParams } from "./account.js";
import {
  type EcdsaClientActions,
  ecdsaClientActions,
} from "./client-actions.js";

/**
 * Parameters for creating an ECDSA bundler client
 */
export type CreateEcdsaClientParams<
  TTransport extends Transport = Transport,
  TChain extends Chain = Chain,
> = {
  /** ECDSA account configuration */
  account: {
    /** Smart account address (required - no counterfactual support). */
    address: Address;
    /** ECDSA signer private key (hex string). */
    signerPrivateKey: Hash;
    /** EOA validator contract address (required for signature formatting). */
    eoaValidatorAddress: Address;
  };

  /** Bundler client instance (created externally by user) */
  bundlerClient: BundlerClient;

  /** Chain configuration */
  chain: TChain;

  /** Transport for public RPC calls */
  transport: TTransport;

  /** Optional client metadata */
  key?: string;
  name?: string;
};

/**
 * ECDSA bundler client type with all actions
 */
export type EcdsaClient<
  TTransport extends Transport = Transport,
  TChain extends Chain = Chain,
  TRpcSchema extends RpcSchema | undefined = undefined,
> = Prettify<
  Client<
    TTransport,
    TChain,
    undefined,
    TRpcSchema extends RpcSchema
      ? [...PublicRpcSchema, ...WalletRpcSchema, ...TRpcSchema]
      : [...PublicRpcSchema, ...WalletRpcSchema]
  > &
  PublicActions<TTransport, TChain> &
  EcdsaClientActions & {
    /** The bundler client instance */
    bundler: BundlerClient;
  }
>;

/**
 * Create a viem wallet client wrapper around an ECDSA smart account.
 *
 * This client provides a standard viem WalletClient interface while using
 * ERC-4337 user operations under the hood. All transactions are sent via
 * the bundler and return actual transaction hashes (not userOp hashes).
 *
 * The smart account is lazy-loaded on first transaction since toEcdsaSmartAccount is async.
 *
 * @param params - Configuration including account details, bundler client, and chain
 * @returns A viem wallet client compatible with wagmi and standard tooling
 *
 * @example
 * ```typescript
 * import { createPublicClient, http } from "viem";
 * import { createBundlerClient } from "viem/account-abstraction";
 * import { createEcdsaClient } from "zksync-sso-4337/client/ecdsa";
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
 * const client = createEcdsaClient({
 *   account: {
 *     address: "0x...",
 *     signerPrivateKey: "0x...",
 *     eoaValidatorAddress: "0x...",
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
 * // Sign messages (supported for ECDSA)
 * const signature = await client.signMessage({ message: "Hello" });
 *
 * // Access bundler client directly
 * const bundler = client.bundler;
 * ```
 */
export function createEcdsaClient<
  TTransport extends Transport = Transport,
  TChain extends Chain = Chain,
  TRpcSchema extends RpcSchema | undefined = undefined,
>(
  params: CreateEcdsaClientParams<TTransport, TChain>,
): EcdsaClient<TTransport, TChain, TRpcSchema> {
  const { account: accountConfig, bundlerClient, chain, transport } = params;

  // Create public client for RPC calls
  const publicClient = createPublicClient({
    chain,
    transport,
  });

  // Prepare ECDSA account params for lazy loading
  const ecdsaAccountParams: ToEcdsaSmartAccountParams = {
    client: publicClient as ToEcdsaSmartAccountParams["client"],
    address: accountConfig.address,
    signerPrivateKey: accountConfig.signerPrivateKey,
    eoaValidatorAddress: accountConfig.eoaValidatorAddress,
  };

  // Create the client with all actions
  // Use standard pattern: extend with walletActions first, then override with custom actions
  const client = createClient({
    chain,
    transport,
    account: undefined,
    type: "walletClient",
    key: params.key || "zksync-sso-ecdsa-client",
    name: params.name || "ZKsync SSO ECDSA Client",
  })
    .extend(publicActions)
    .extend(walletActions)
    .extend((client) =>
      ecdsaClientActions({
        client,
        bundler: bundlerClient,
        ecdsaAccount: ecdsaAccountParams,
        accountAddress: accountConfig.address,
      }),
    )
    .extend(() => ({
      bundler: bundlerClient,
    }));

  return client as EcdsaClient<TTransport, TChain, TRpcSchema>;
}
