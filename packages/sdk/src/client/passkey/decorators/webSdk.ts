/**
 * Web-SDK Transaction Decorator
 *
 * Adds sendTransactionWebSdk action that uses ERC-4337 UserOperations via the web-SDK
 * This is a CLEAN BREAK from the old transaction flow.
 */

import type { Account, Chain, Client, Hash, SendTransactionParameters, Transport } from "viem";

import type { ClientWithZksyncSsoPasskeyData } from "../client.js";

/**
 * Web-SDK transaction actions
 */
export type WebSdkActions<
  chain extends Chain | undefined = Chain | undefined,
  account extends Account | undefined = Account | undefined,
> = {
  /**
   * Send a transaction using Web-SDK ERC-4337 UserOperations
   *
   * This is a CLEAN BREAK from the old sendTransaction - it uses the web-SDK
   * for all transaction handling via ERC-4337 UserOperations.
   *
   * @example
   * ```typescript
   * const client = createZksyncPasskeyClient({
   *   ...config,
   *   bundlerUrl: 'https://bundler.example.com',
   *   entryPointAddress: '0x...',
   * });
   *
   * const hash = await client.sendTransactionWebSdk({
   *   to: '0x...',
   *   value: parseEther('0.1'),
   *   data: '0x...',
   * });
   * ```
   */
  sendTransactionWebSdk: (args: SendTransactionParameters<chain, account>) => Promise<Hash>;
};

/**
 * Add Web-SDK transaction actions to a passkey client
 */
export function webSdkActions<
  transport extends Transport,
  chain extends Chain | undefined = Chain | undefined,
  account extends Account | undefined = Account | undefined,
>(
  client: Client<transport, chain, account> & ClientWithZksyncSsoPasskeyData,
): WebSdkActions<chain, account> {
  return {
    async sendTransactionWebSdk(args) {
      return await sendTransactionViaWebSdk(client, args);
    },
  };
}

/**
 * Internal: Send transaction using Web-SDK
 */
async function sendTransactionViaWebSdk(
  client: Client & ClientWithZksyncSsoPasskeyData & { account: Account },
  args: SendTransactionParameters,
): Promise<Hash> {
  // Validate configuration
  if (!client.bundlerUrl) {
    throw new Error("bundlerUrl is required for Web-SDK transactions");
  }
  if (!client.entryPointAddress) {
    throw new Error("entryPointAddress is required for Web-SDK transactions");
  }
  if (!client.contracts.passkey) {
    throw new Error("contracts.passkey (webauthn validator) is required");
  }

  // Extract RPC URL from transport
  const rpcUrl = extractRpcUrl(client);

  // Extract transaction parameters
  const to = args.to;
  const value = args.value || 0n;
  const data = args.data || "0x";

  if (!to) {
    throw new Error("Transaction 'to' address is required");
  }

  // Dynamic import to avoid bundling web-SDK unnecessarily
  const {
    prepare_passkey_user_operation,
    submit_passkey_user_operation,
    SendTransactionConfig,
  } = await import("zksync-sso-web-sdk/bundler");

  const { signWithPasskey } = await import("zksync-sso-web-sdk/webauthn-helpers");

  // Step 1: Prepare UserOperation
  const config = new SendTransactionConfig(
    rpcUrl,
    client.bundlerUrl,
    client.entryPointAddress,
  );

  const prepared = await prepare_passkey_user_operation(
    config,
    client.contracts.passkey, // webauthn validator
    client.account.address,
    to,
    value.toString(),
    data === "0x" ? null : (data as string),
  );

  // Step 2: Sign with passkey
  // Get origin and credential ID
  const origin = getOrigin();
  const credentialId = getCredentialId(client);

  const signature = await signWithPasskey({
    hash: prepared.user_op_hash,
    credentialId,
    rpId: getRpId(origin),
    origin,
  });

  // Step 3: Submit UserOperation
  const txHash = await submit_passkey_user_operation(
    config,
    JSON.stringify(prepared),
    signature.signature,
  );

  return txHash as Hash;
}

/**
 * Helper: Extract RPC URL from viem transport
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
 * Helper: Get credential ID as hex string
 */
function getCredentialId(client: ClientWithZksyncSsoPasskeyData): string {
  if (client.credential?.id) {
    // Convert base64url to hex
    const bytes = base64UrlToBytes(client.credential.id);
    return `0x${Array.from(bytes).map((b) => b.toString(16).padStart(2, "0")).join("")}`;
  }

  throw new Error(
    "Credential ID not found. Please provide client.credential.id",
  );
}

/**
 * Helper: Get origin for passkey validation
 */
function getOrigin(): string {
  if (typeof window !== "undefined" && window.location) {
    return window.location.origin;
  }
  throw new Error(
    "Cannot determine origin automatically. Running in non-browser environment?",
  );
}

/**
 * Helper: Extract RP ID from origin
 */
function getRpId(origin: string): string {
  try {
    const url = new URL(origin);
    return url.hostname;
  } catch {
    throw new Error(`Invalid origin URL: ${origin}`);
  }
}

/**
 * Helper: Convert base64url to Uint8Array
 */
function base64UrlToBytes(base64url: string): Uint8Array {
  // Replace URL-safe characters
  const base64 = base64url.replace(/-/g, "+").replace(/_/g, "/");

  // Pad with '=' if necessary
  const padded = base64.padEnd(
    base64.length + ((4 - (base64.length % 4)) % 4),
    "=",
  );

  // Decode base64 to binary string
  const binary = atob(padded);

  // Convert binary string to Uint8Array
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }

  return bytes;
}
