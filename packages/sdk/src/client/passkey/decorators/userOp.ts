/**
 * UserOp Actions - Transparent conversion of viem transactions to ERC-4337 UserOperations
 *
 * This decorator intercepts sendTransaction calls and converts them to UserOperations
 * when a bundler is configured, maintaining full viem API compatibility.
 *
 * @module userOpActions
 */

import {
  prepare_passkey_user_operation,
  SendTransactionConfig,
  submit_passkey_user_operation,
} from "@zksync-sso/sdk-platforms-web/bundler";
import type { Account, Chain, Client, Hash, Hex, SendTransactionParameters, Transport } from "viem";
import { sendTransaction as viemSendTransaction } from "viem/actions";

/**
 * Client with UserOp capabilities
 */
export type ClientWithUserOp = {
  bundlerUrl?: string;
  entryPointAddress?: `0x${string}`;
  useWebSdk?: boolean;
  contracts?: {
    passkey?: `0x${string}`;
    [key: string]: any;
  };
};

/**
 * UserOp actions that can be added to any viem client
 */
export type UserOpActions<
  chain extends Chain | undefined = Chain | undefined,
  account extends Account | undefined = Account | undefined,
> = {
  /**
   * Send a transaction - automatically converts to UserOperation if bundler is configured
   *
   * @example
   * // Standard transaction (no bundler)
   * const hash = await client.sendTransaction({ to, value, data });
   *
   * @example
   * // Automatic UserOp conversion (bundler configured)
   * const client = createZksyncPasskeyClient({
   *   ...config,
   *   bundlerUrl: 'https://bundler.example.com',
   *   entryPointAddress: '0x...',
   * });
   * const hash = await client.sendTransaction({ to, value, data });
   */
  sendTransaction: <request extends SendTransactionParameters<chain, account>>(
    args: request,
  ) => Promise<Hash>;
};

/**
 * Add UserOp capabilities to a viem client
 *
 * This decorator transparently converts transactions to UserOperations when:
 * 1. bundlerUrl is configured
 * 2. entryPointAddress is configured
 * 3. useWebSdk is true (or undefined)
 *
 * Otherwise, it falls back to standard viem transactions.
 *
 * @example
 * ```typescript
 * const client = createClient(...)
 *   .extend(publicActions)
 *   .extend(walletActions)
 *   .extend(userOpActions); // Add UserOp support
 *
 * // Use like any viem client
 * await client.sendTransaction({ to, value });
 * ```
 */
export function userOpActions<
  transport extends Transport,
  chain extends Chain | undefined = Chain | undefined,
  account extends Account | undefined = Account | undefined,
>(
  client: Client<transport, chain, account> & ClientWithUserOp,
): UserOpActions<chain, account> {
  return {
    async sendTransaction(args) {
      // Check if we should use UserOps
      const shouldUseUserOps
        = client.bundlerUrl
        && client.entryPointAddress
        && (client.useWebSdk ?? true) // Default to true if not specified
        && client.account; // Must have an account

      if (shouldUseUserOps) {
        try {
          return await sendTransactionAsUserOp(client, args);
        } catch (error) {
          console.error("Failed to send transaction as UserOp, falling back to standard tx:", error);
          // Fall through to standard transaction
        }
      }

      // Fall back to standard viem transaction
      return await viemSendTransaction(client, args as any);
    },
  };
}

/**
 * Internal: Convert a viem transaction to a UserOperation and submit it
 */
async function sendTransactionAsUserOp(
  client: Client & ClientWithUserOp & { account: Account },
  args: SendTransactionParameters,
): Promise<Hash> {
  // Extract RPC URL from transport
  const rpcUrl = extractRpcUrl(client);

  if (!client.bundlerUrl || !client.entryPointAddress) {
    throw new Error("bundlerUrl and entryPointAddress are required for UserOperations");
  }

  // Extract transaction parameters
  const to = args.to;
  const value = args.value || 0n;
  const data = args.data || "0x";

  if (!to) {
    throw new Error("Transaction 'to' address is required");
  }

  // Get webauthn validator address (required for passkey accounts)
  const webauthnValidatorAddress = client.contracts?.passkey;
  if (!webauthnValidatorAddress) {
    throw new Error("contracts.passkey (webauthn validator) is required for UserOperations");
  }

  // Step 1: Prepare UserOperation
  const config = new SendTransactionConfig(
    rpcUrl,
    client.bundlerUrl,
    client.entryPointAddress,
  );

  const prepared = await prepare_passkey_user_operation(
    config,
    webauthnValidatorAddress,
    client.account.address,
    to,
    value.toString(),
    data === "0x" ? null : (data as string),
  );

  // Step 2: Sign the UserOperation hash
  // Use the account's existing sign method (works with passkey, session, etc.)
  const signature = await client.account.signMessage({
    message: { raw: prepared.user_op_hash as Hex },
  });

  // Step 3: Submit the UserOperation
  const txHash = await submit_passkey_user_operation(
    config,
    JSON.stringify(prepared),
    signature,
  );

  return txHash as Hash;
}

/**
 * Extract RPC URL from various transport types
 */
function extractRpcUrl(client: Client): string {
  const transport = client.transport;

  // HTTP transport
  if ("url" in transport) {
    if (typeof transport.url === "string") {
      return transport.url;
    }
    if (typeof transport.url === "function") {
      return transport.url({ chain: client.chain });
    }
  }

  // Custom transport with config
  if ("config" in transport && transport.config && "url" in transport.config) {
    return transport.config.url as string;
  }

  // Fallback: check chain's RPC URLs
  if (client.chain?.rpcUrls?.default?.http?.[0]) {
    return client.chain.rpcUrls.default.http[0];
  }

  throw new Error(
    "Cannot extract RPC URL from transport. Please use http() transport with explicit URL.",
  );
}

/**
 * Type guard to check if a client has UserOp support
 */
export function hasUserOpSupport(client: any): client is Client & ClientWithUserOp {
  return (
    client
    && typeof client.bundlerUrl === "string"
    && typeof client.entryPointAddress === "string"
  );
}

/**
 * Helper to check if UserOps are enabled for a client
 */
export function isUserOpEnabled(client: Client & ClientWithUserOp): boolean {
  return !!(
    client.bundlerUrl
    && client.entryPointAddress
    && (client.useWebSdk ?? true)
  );
}
