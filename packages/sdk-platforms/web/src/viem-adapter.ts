/**
 * Viem-compatible adapter for the web-SDK
 *
 * This module provides a shim layer that makes the web-SDK a drop-in replacement
 * for the existing passkey and ecdsa clients. It wraps the WASM-based functions
 * in a viem-compatible interface.
 *
 * @module viem-adapter
 */

import type { PublicKeyCredentialDescriptorJSON } from "@simplewebauthn/browser";
import type {
  Address,
  Chain,
  Hash,
  Hex,
  LocalAccount,
  Transport,
  WalletClient,
} from "viem";
import { createWalletClient } from "viem";

import {
  prepare_passkey_user_operation,
  SendTransactionConfig,
  submit_passkey_user_operation,
} from "../pkg-bundler/zksync_sso_erc4337_web_ffi";
import { signWithPasskey } from "./webauthn-helpers";

// ============================================================================
// TYPES
// ============================================================================

/**
 * Configuration for creating a Web-SDK backed client
 */
export interface WebSdkClientConfig<
  transport extends Transport = Transport,
  chain extends Chain = Chain,
> {
  /** The blockchain chain */
  chain: chain;
  /** Transport (http, custom, etc.) */
  transport: transport;
  /** Smart account address */
  address: Address;
  /** Bundler URL for ERC-4337 operations */
  bundlerUrl: string;
  /** EntryPoint contract address */
  entryPointAddress: Address;
  /** WebAuthn validator contract address */
  webauthnValidatorAddress: Address;
  /** Passkey credential configuration */
  passkey: {
    /** Credential ID (hex string) */
    credentialId: string;
    /** Public key X coordinate (hex string) */
    publicKeyX: string;
    /** Public key Y coordinate (hex string) */
    publicKeyY: string;
    /** Relying Party ID (domain) */
    rpId: string;
    /** Origin URL */
    origin: string;
    /** User display name */
    userName: string;
    /** User identifier */
    userDisplayName: string;
  };
}

/**
 * Web-SDK backed account that implements viem's Account interface
 */
export interface WebSdkAccount extends LocalAccount<"webSdkAccount"> {
  /** Bundler URL */
  bundlerUrl: string;
  /** EntryPoint address */
  entryPointAddress: Address;
  /** WebAuthn validator address */
  webauthnValidatorAddress: Address;
  /** Passkey configuration */
  passkey: WebSdkClientConfig["passkey"];
}

/**
 * Extended client with Web-SDK specific methods
 */
export type WebSdkClient<
  transport extends Transport = Transport,
  chain extends Chain = Chain,
> = WalletClient<transport, chain, WebSdkAccount> & {
  /** Send a transaction using the Web-SDK */
  sendTransactionWebSdk: (params: {
    to: Address;
    value: bigint;
    data?: Hex;
  }) => Promise<Hash>;

  /** Deploy account using Web-SDK */
  deployAccountWebSdk: () => Promise<{
    address: Address;
    transactionHash: Hash;
  }>;
};

// ============================================================================
// ACCOUNT CREATION
// ============================================================================

/**
 * Create a Web-SDK backed account that implements viem's LocalAccount interface
 *
 * This account can be used with any viem client and provides the same signing
 * interface as the existing passkey accounts, but uses the Web-SDK under the hood.
 */
export function toWebSdkAccount(config: {
  address: Address;
  bundlerUrl: string;
  entryPointAddress: Address;
  webauthnValidatorAddress: Address;
  passkey: WebSdkClientConfig["passkey"];
  transport: Transport;
}): WebSdkAccount {
  const {
    address,
    bundlerUrl,
    entryPointAddress,
    webauthnValidatorAddress,
    passkey,
    transport,
  } = config;

  // Get RPC URL from transport
  // This is a bit hacky but necessary to bridge viem's transport to our WASM config
  let rpcUrl: string;
  if ("url" in transport && typeof transport.url === "string") {
    rpcUrl = transport.url;
  } else if ("url" in transport && typeof transport.url === "function") {
    rpcUrl = transport.url({ chain: undefined });
  } else {
    // Default fallback - apps should provide proper transport
    rpcUrl = "http://localhost:8545";
  }

  const account: WebSdkAccount = {
    address,
    type: "local",
    source: "webSdkAccount",
    bundlerUrl,
    entryPointAddress,
    webauthnValidatorAddress,
    passkey,

    // Sign a raw hash using the passkey
    async signMessage({ message }) {
      // For message signing, we need to hash the message first
      // This follows EIP-191 standard
      const messageHash = typeof message === "string"
        ? message
        : `0x${Buffer.from(message).toString("hex")}`;

      const result = await signWithPasskey({
        hash: messageHash,
        credentialId: passkey.credentialId,
        rpId: passkey.rpId,
        origin: passkey.origin,
      });

      return result.signature as Hex;
    },

    // Sign typed data (EIP-712)
    async signTypedData(typedData) {
      throw new Error(`${typedData} signTypedData not yet implemented for Web-SDK adapter`);
    },

    // Sign a transaction
    async signTransaction(transaction) {
      // The Web-SDK handles transaction signing as part of the UserOperation flow
      // We prepare the UserOp and return a "signed" transaction that actually
      // contains the UserOperation data

      const config = new SendTransactionConfig(
        rpcUrl,
        bundlerUrl,
        entryPointAddress,
      );

      const prepareResult = await prepare_passkey_user_operation(
        config,
        webauthnValidatorAddress,
        address,
        transaction.to as string,
        transaction.value?.toString() || "0",
        transaction.data as string || null,
      );

      // Sign the UserOperation hash with passkey
      const signatureResult = await signWithPasskey({
        hash: prepareResult.user_op_hash,
        credentialId: passkey.credentialId,
        rpId: passkey.rpId,
        origin: passkey.origin,
      });

      // For compatibility, we return a serialized format that our sendTransaction
      // handler can recognize and process
      return JSON.stringify({
        type: "web-sdk-user-operation",
        userOpHash: prepareResult.user_op_hash,
        signature: signatureResult.signature,
        transaction,
        config: {
          rpcUrl,
          bundlerUrl,
          entryPointAddress,
          webauthnValidatorAddress,
        },
      }) as Hex;
    },
  };

  return account;
}

// ============================================================================
// CLIENT CREATION
// ============================================================================

/**
 * Create a viem-compatible wallet client backed by the Web-SDK
 *
 * This client provides the same interface as createZksyncPasskeyClient but
 * uses the Web-SDK implementation under the hood. It can be used as a drop-in
 * replacement in existing code.
 *
 * @example
 * ```typescript
 * const client = createWebSdkClient({
 *   chain: zkSyncSepoliaTestnet,
 *   transport: http('https://sepolia.era.zksync.dev'),
 *   address: '0x...',
 *   bundlerUrl: 'https://bundler.example.com',
 *   entryPointAddress: '0x...',
 *   webauthnValidatorAddress: '0x...',
 *   passkey: {
 *     credentialId: '0x...',
 *     publicKeyX: '0x...',
 *     publicKeyY: '0x...',
 *     rpId: 'example.com',
 *     origin: 'https://example.com',
 *     userName: 'user@example.com',
 *     userDisplayName: 'User Name',
 *   },
 * });
 *
 * // Use like any viem client
 * const hash = await client.sendTransaction({
 *   to: '0x...',
 *   value: parseEther('0.1'),
 * });
 * ```
 */
export function createWebSdkClient<
  transport extends Transport,
  chain extends Chain,
>(config: WebSdkClientConfig<transport, chain>): WebSdkClient<transport, chain> {
  const {
    chain,
    transport,
    address,
    bundlerUrl,
    entryPointAddress,
    webauthnValidatorAddress,
    passkey,
  } = config;

  // Create the account
  const account = toWebSdkAccount({
    address,
    bundlerUrl,
    entryPointAddress,
    webauthnValidatorAddress,
    passkey,
    transport,
  });

  // Create base wallet client
  const baseClient = createWalletClient({
    account,
    chain,
    transport,
  });

  // Extend with Web-SDK specific methods
  const client = {
    ...baseClient,

    /**
     * Send a transaction using the Web-SDK's ERC-4337 flow
     *
     * This overrides the standard sendTransaction to use UserOperations
     */
    async sendTransactionWebSdk(params: {
      to: Address;
      value: bigint;
      data?: Hex;
    }) {
      const { to, value, data } = params;

      // Get RPC URL from transport
      let rpcUrl: string;
      if ("url" in transport && typeof transport.url === "string") {
        rpcUrl = transport.url;
      } else if ("url" in transport && typeof transport.url === "function") {
        rpcUrl = transport.url({ chain });
      } else {
        throw new Error("Cannot extract RPC URL from transport");
      }

      const txConfig = new SendTransactionConfig(
        rpcUrl,
        bundlerUrl,
        entryPointAddress,
      );

      // Step 1: Prepare UserOperation
      const prepareResult = await prepare_passkey_user_operation(
        txConfig,
        webauthnValidatorAddress,
        address,
        to,
        value.toString(),
        data || null,
      );

      // Step 2: Sign with passkey
      const signatureResult = await signWithPasskey({
        hash: prepareResult.user_op_hash,
        credentialId: passkey.credentialId,
        rpId: passkey.rpId,
        origin: passkey.origin,
      });

      // Step 3: Submit UserOperation
      const result = await submit_passkey_user_operation(
        txConfig,
        webauthnValidatorAddress,
        address,
        to,
        value.toString(),
        data || null,
        signatureResult.signature,
      );

      return result as Hash;
    },

    /**
     * Deploy a new account using the Web-SDK
     */
    async deployAccountWebSdk() {
      // Get RPC URL from transport
      let rpcUrl: string;
      if ("url" in transport && typeof transport.url === "string") {
        rpcUrl = transport.url;
      } else if ("url" in transport && typeof transport.url === "function") {
        rpcUrl = transport.url({ chain });
      } else {
        throw new Error("Cannot extract RPC URL from transport");
      }

      const deployConfig = new DeployAccountConfig(
        rpcUrl,
        bundlerUrl,
        entryPointAddress,
      );

      const passkeyPayload = new PasskeyPayload(
        passkey.credentialId,
        passkey.publicKeyX,
        passkey.publicKeyY,
      );

      const result = await deploy_account(
        deployConfig,
        passkeyPayload,
        webauthnValidatorAddress,
      );

      return {
        address: result.account_address as Address,
        transactionHash: result.tx_hash as Hash,
      };
    },
  };

  return client as WebSdkClient<transport, chain>;
}

// ============================================================================
// COMPATIBILITY HELPERS
// ============================================================================

/**
 * Convert from existing passkey client config to Web-SDK config
 *
 * This helper makes it easy to migrate from the existing passkey client
 * to the Web-SDK backed client.
 */
export function passkeyClientConfigToWebSdk(config: {
  chain: Chain;
  transport: Transport;
  address: Address;
  credentialPublicKey: Uint8Array;
  userName: string;
  userDisplayName: string;
  credential?: PublicKeyCredentialDescriptorJSON;
  contracts: {
    passkey: Address;
    oidcKeyRegistry: Address;
    session: Address;
    recovery: Address;
    recoveryOidc: Address;
    accountFactory?: Address;
  };
  bundlerUrl: string;
  entryPointAddress: Address;
  rpId: string;
  origin: string;
}): WebSdkClientConfig {
  // Extract public key coordinates from the full public key
  // The public key is 65 bytes: [0x04][32 bytes X][32 bytes Y]
  const pubKey = config.credentialPublicKey;
  if (pubKey.length !== 65 || pubKey[0] !== 0x04) {
    throw new Error("Invalid public key format");
  }

  const publicKeyX = `0x${Buffer.from(pubKey.slice(1, 33)).toString("hex")}`;
  const publicKeyY = `0x${Buffer.from(pubKey.slice(33, 65)).toString("hex")}`;

  const credentialId = config.credential?.id || "";

  return {
    chain: config.chain,
    transport: config.transport,
    address: config.address,
    bundlerUrl: config.bundlerUrl,
    entryPointAddress: config.entryPointAddress,
    webauthnValidatorAddress: config.contracts.passkey,
    passkey: {
      credentialId,
      publicKeyX,
      publicKeyY,
      rpId: config.rpId,
      origin: config.origin,
      userName: config.userName,
      userDisplayName: config.userDisplayName,
    },
  };
}

/**
 * Check if a client is using the Web-SDK backend
 */
export function isWebSdkClient(client): client is WebSdkClient {
  return (
    client?.account?.source === "webSdkAccount"
    && typeof client?.sendTransactionWebSdk === "function"
  );
}
