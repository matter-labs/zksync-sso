/**
 * Deploy Account using Web-SDK (ERC-4337)
 *
 * This module provides account deployment using the WASM-based web-SDK,
 * replacing the old TypeScript-based deployment with a clean break to ERC-4337.
 */

import type { Address, Chain, Client, Hash, Hex, Transport } from "viem";

import type { ClientWithZksyncSsoPasskeyData } from "../client.js";

// Import web-SDK types and functions
type DeployAccountResult = {
  account_address: string;
  tx_hash: string;
};

/**
 * Deploy a smart account using passkey with Web-SDK
 *
 * This is a CLEAN BREAK from the old deployment method - it uses ERC-4337
 * UserOperations via the web-SDK instead of the old TypeScript implementation.
 *
 * @param client - Viem client with passkey configuration
 * @param args - Deployment configuration
 * @returns Deployed account address and transaction hash
 *
 * @example
 * ```typescript
 * const client = createZksyncPasskeyClient({
 *   chain: zkSyncSepoliaTestnet,
 *   transport: http(),
 *   address: '0x...', // Will be overwritten by deployed address
 *   credentialPublicKey: new Uint8Array([...]),
 *   userName: 'user@example.com',
 *   userDisplayName: 'User Name',
 *   contracts: { ... },
 *   bundlerUrl: 'https://bundler.example.com',
 *   entryPointAddress: '0x...',
 * });
 *
 * const result = await deployAccountWebSdk(client, {
 *   userId: 'user@example.com',
 *   deployerPrivateKey: '0x...',
 * });
 *
 * console.log('Account deployed:', result.address);
 * console.log('Transaction:', result.transactionHash);
 * ```
 */
export async function deployAccountWebSdk<
  transport extends Transport,
  chain extends Chain,
>(
  client: Client<transport, chain> & ClientWithZksyncSsoPasskeyData,
  args: DeployAccountWebSdkArgs,
): Promise<DeployAccountWebSdkReturnType> {
  // Dynamic import to avoid bundling web-SDK unnecessarily
  const {
    deploy_account,
    DeployAccountConfig,
    PasskeyPayload,
  } = await import("zksync-sso-web-sdk/bundler");

  // Validate required configuration
  if (!client.bundlerUrl) {
    throw new Error("bundlerUrl is required for Web-SDK deployment");
  }
  if (!client.entryPointAddress) {
    throw new Error("entryPointAddress is required for Web-SDK deployment");
  }
  if (!client.contracts.accountFactory) {
    throw new Error("contracts.accountFactory is required for deployment");
  }
  if (!client.contracts.passkey) {
    throw new Error("contracts.passkey (webauthn validator) is required");
  }

  // Extract RPC URL from transport
  const rpcUrl = extractRpcUrl(client);

  // Extract passkey public key coordinates
  // The credentialPublicKey is 65 bytes: [0x04][32 bytes X][32 bytes Y]
  const pubKey = client.credentialPublicKey;
  if (pubKey.length !== 65 || pubKey[0] !== 0x04) {
    throw new Error("Invalid public key format. Expected uncompressed P-256 key (65 bytes)");
  }

  const publicKeyX = pubKey.slice(1, 33);
  const publicKeyY = pubKey.slice(33, 65);

  // Get origin (where the credential was created)
  const origin = args.origin || getOrigin();

  // Create deployment config
  const deployConfig = new DeployAccountConfig(
    rpcUrl,
    client.contracts.accountFactory,
    args.deployerPrivateKey,
    undefined, // eoa_validator_address (not used for passkey-only deployment)
    client.contracts.passkey, // webauthn_validator_address
  );

  // Create passkey payload
  const passkeyPayload = new PasskeyPayload(
    getCredentialIdBytes(client),
    publicKeyX,
    publicKeyY,
    origin,
  );

  // Deploy the account using web-SDK
  const result = (await deploy_account(
    args.userId,
    null, // eoa_signers_addresses (not used)
    passkeyPayload,
    deployConfig,
  )) as DeployAccountResult;

  return {
    address: result.account_address as Address,
    transactionHash: result.tx_hash as Hash,
  };
}

/**
 * Arguments for deploying an account with Web-SDK
 */
export type DeployAccountWebSdkArgs = {
  /**
   * Unique user identifier (will be hashed to create account_id)
   * This ensures deterministic account addresses
   */
  userId: string;

  /**
   * Private key of the account that will pay for deployment
   * Should be a hex string with 0x prefix
   */
  deployerPrivateKey: Hex;

  /**
   * Expected origin where the passkey was created
   * Defaults to window.location.origin if in browser
   */
  origin?: string;
};

/**
 * Return type for account deployment
 */
export type DeployAccountWebSdkReturnType = {
  /**
   * Address of the deployed smart account
   */
  address: Address;

  /**
   * Transaction hash of the deployment
   */
  transactionHash: Hash;
};

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
 * Helper: Get credential ID as Uint8Array
 */
function getCredentialIdBytes(client: ClientWithZksyncSsoPasskeyData): Uint8Array {
  // The credential ID might be stored in different formats
  if (client.credential?.id) {
    // Base64URL format - convert to bytes
    return base64UrlToBytes(client.credential.id);
  }

  // Try to derive from credentialPublicKey or other sources
  // For now, throw an error if not available
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
    "Cannot determine origin automatically. Please provide args.origin",
  );
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
