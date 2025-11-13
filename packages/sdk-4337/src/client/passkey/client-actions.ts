import {
  type Account,
  type Address,
  type Chain,
  type Client,
  encodeFunctionData,
  type Hash,
  type Hex,
  type Transport,
  type WalletActions,
} from "viem";
import type { BundlerClient, ToSmartAccountReturnType } from "viem/account-abstraction";
import {
  getAddresses,
  getCallsStatus,
  getCapabilities,
  getChainId,
  sendRawTransaction as viemSendRawTransaction,
  showCallsStatus,
  waitForCallsStatus,
} from "viem/actions";

import { addPasskey } from "../actions/passkey.js";
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
};

/**
 * Wallet actions type for passkey client
 * Based on WalletActions but omits methods that don't make sense for smart accounts or are overridden
 */
export type PasskeyClientActions<TChain extends Chain = Chain, TAccount extends Account = Account> = Omit<
  WalletActions<TChain, TAccount>,
  "addChain" | "getPermissions" | "requestAddresses" | "requestPermissions" | "switchChain" | "watchAsset" | "prepareTransactionRequest" | "signTransaction"
> & {
  /**
   * Add a passkey to the smart account.
   * Returns the transaction hash.
   */
  addPasskey: (params: {
    credentialId: Hex;
    publicKey: { x: Hex; y: Hex };
    originDomain: string;
  }) => Promise<Hash>;

  /**
   * Wait for a user operation receipt.
   */
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  waitForUserOperationReceipt: (params: { hash: Hash }) => Promise<any>;
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
  // Lazy-load the smart account (cached after first load)
  let smartAccountPromise: Promise<ToSmartAccountReturnType> | null = null;
  const getSmartAccount = async (): Promise<ToSmartAccountReturnType> => {
    if (!smartAccountPromise) {
      smartAccountPromise = toPasskeySmartAccount(config.passkeyAccount);
    }
    return smartAccountPromise;
  };

  return {
    // Read-only actions - pass through
    getAddresses: () => getAddresses(config.client),
    getChainId: () => getChainId(config.client),
    getCallsStatus: (args) => getCallsStatus(config.client, args),
    getCapabilities: (args) => getCapabilities(config.client, args),
    showCallsStatus: (args) => showCallsStatus(config.client, args),
    waitForCallsStatus: (args) => waitForCallsStatus(config.client, args),

    // Deploy contract through bundler
    // Note: Direct contract deployment via userOps is not supported in the standard way
    // Users should use a factory pattern or CREATE2 through sendTransaction instead
    deployContract: () => {
      throw new Error("deployContract is not supported for passkey smart accounts.");
    },

    // Send raw transaction - pass through to transport
    sendRawTransaction: (args) => {
      return viemSendRawTransaction(config.client, args);
    },

    // Send multiple calls in a single userOp
    sendCalls: async (args) => {
      const account = await getSmartAccount();

      // Send all calls through bundler in a single userOp
      const userOpHash = await config.bundler.sendUserOperation({
        account,
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        calls: args.calls.map((call: any) => ({
          to: call.to!,
          value: call.value ?? 0n,
          data: call.data ?? "0x",
        })),
      });

      // Return the userOp hash as the ID in the expected format
      return {
        id: userOpHash,
      };
    },

    // Unsupported signing actions - throw errors
    signTypedData: () => {
      throw new Error("signTypedData is not supported for passkey smart accounts");
    },
    signMessage: () => {
      throw new Error("signMessage is not supported for passkey smart accounts");
    },
    signAuthorization: () => {
      throw new Error("signAuthorization is not supported for passkey smart accounts");
    },
    prepareAuthorization: () => {
      throw new Error("prepareAuthorization is not supported for passkey smart accounts");
    },

    // Override sendTransaction to use bundler
    sendTransaction: async (args) => {
      const account = await getSmartAccount();

      // Send user operation through bundler
      const userOpHash = await config.bundler.sendUserOperation({
        account,
        calls: [
          {
            to: args.to!,
            value: args.value ?? 0n,
            data: args.data ?? "0x",
          },
        ],
      });

      // Wait for the user operation to be included and get the actual transaction hash
      const receipt = await config.bundler.waitForUserOperationReceipt({
        hash: userOpHash,
      });

      // Return the actual transaction hash (not the userOp hash)
      // This makes the client fully compatible with EOA behavior
      return receipt.receipt.transactionHash;
    },

    // Override writeContract to use bundler
    writeContract: async (args) => {
      const account = await getSmartAccount();

      // Encode the contract call data
      const data = encodeFunctionData({
        abi: args.abi,
        functionName: args.functionName,
        args: args.args,
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      } as any);

      // Send through bundler
      const userOpHash = await config.bundler.sendUserOperation({
        account,
        calls: [
          {
            to: args.address,
            value: args.value ?? 0n,
            data,
          },
        ],
      });

      // Wait for confirmation and return actual tx hash
      const receipt = await config.bundler.waitForUserOperationReceipt({
        hash: userOpHash,
      });

      return receipt.receipt.transactionHash;
    },

    // Add passkey to the smart account
    addPasskey: async (params: {
      credentialId: Hex;
      publicKey: { x: Hex; y: Hex };
      originDomain: string;
    }) => {
      const account = await getSmartAccount();

      // Generate the transaction using the existing addPasskey function
      const { transaction } = addPasskey({
        account: config.accountAddress,
        contracts: {
          webauthnValidator: config.validatorAddress,
        },
        passkeySigner: {
          credentialId: params.credentialId,
          publicKey: params.publicKey,
          originDomain: params.originDomain,
        },
      });

      // Send the transaction through bundler
      const userOpHash = await config.bundler.sendUserOperation({
        account,
        calls: [
          {
            to: transaction.to,
            data: transaction.data,
            value: 0n,
          },
        ],
      });

      // Wait for the user operation to be included
      const receipt = await config.bundler.waitForUserOperationReceipt({
        hash: userOpHash,
      });

      // Return the actual transaction hash from the receipt
      return receipt.receipt.transactionHash;
    },

    // Bundler-specific helper
    waitForUserOperationReceipt: (params: { hash: Hash }) => {
      return config.bundler.waitForUserOperationReceipt({
        hash: params.hash,
      });
    },
  };
}
