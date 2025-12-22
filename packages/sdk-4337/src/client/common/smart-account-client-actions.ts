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

import type { PaymasterConfig } from "../../actions/sendUserOperation.js";
import { addPasskey as addPasskeyAction } from "../actions/passkey.js";

/**
 * Generic smart account client configuration
 */
export type SmartAccountClientData<
  TTransport extends Transport = Transport,
  TChain extends Chain = Chain,
> = {
  /** Bundler client for sending user operations */
  bundler: BundlerClient;
  /** Factory function to create smart account (lazy-loaded) */
  accountFactory: () => Promise<ToSmartAccountReturnType>;
  /** Client for making RPC calls */
  client: Client<TTransport, TChain, Account | undefined>;
  /** Smart account address */
  accountAddress: Address;
  /** Optional paymaster configuration for sponsoring transactions - can be either a full config object or just an address string */
  paymaster?: PaymasterConfig | Address;
};

/**
 * Wallet actions type for smart account client
 * Based on WalletActions but omits methods that don't make sense for smart accounts or are overridden
 */
export type SmartAccountClientActions<TChain extends Chain = Chain, TAccount extends Account = Account> = Omit<
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
    webauthnValidatorAddress: Address;
  }) => Promise<Hash>;

  /**
   * Wait for a user operation receipt.
   */
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  waitForUserOperationReceipt: (params: { hash: Hash }) => Promise<any>;
};

/**
 * Generic decorator that provides wallet actions for smart account clients.
 * Overrides standard wallet methods to use ERC-4337 bundler under the hood.
 * Works with any smart account type (passkey, ECDSA, etc.) that returns ToSmartAccountReturnType.
 */
export function smartAccountClientActions<
  TTransport extends Transport,
  TChain extends Chain,
  TAccount extends Account | undefined = undefined,
>(
  config: SmartAccountClientData<TTransport, TChain>,
): SmartAccountClientActions<TChain, TAccount extends Account ? TAccount : Account> {
  // Normalize paymaster to PaymasterConfig format at the start
  // config.paymaster can be either PaymasterConfig object OR string address
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

  // Lazy-load the smart account (cached after first load)
  let smartAccountPromise: Promise<ToSmartAccountReturnType> | null = null;
  const getSmartAccount = async (): Promise<ToSmartAccountReturnType> => {
    if (!smartAccountPromise) {
      smartAccountPromise = config.accountFactory();
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

    // Send raw transaction - pass through to transport
    sendRawTransaction: (args) => viemSendRawTransaction(config.client, args),

    deployContract: () => {
      throw new Error("deployContract is not supported for smart accounts.");
    },

    // Send multiple calls in a single userOp
    sendCalls: async (args) => {
      const account = await getSmartAccount();

      // For v0.8 EntryPoint, pass separate paymaster fields (not packed paymasterAndData)
      // These fields get included in the UserOp BEFORE signing
      const paymasterParams = normalizedPaymaster
        ? {
            paymaster: normalizedPaymaster.address,
            paymasterData: normalizedPaymaster.data ?? ("0x" as Hex),
            paymasterVerificationGasLimit: normalizedPaymaster.verificationGasLimit ?? 500_000n,
            paymasterPostOpGasLimit: normalizedPaymaster.postOpGasLimit ?? 1_000_000n,
          }
        : {};

      // Send all calls through bundler in a single userOp
      const userOpHash = await config.bundler.sendUserOperation({
        account,
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        calls: args.calls.map((call: any) => ({
          to: call.to!,
          value: call.value ?? 0n,
          data: call.data ?? "0x",
        })),
        ...paymasterParams,
      });

      // Return the userOp hash as the ID in the expected format
      return {
        id: userOpHash,
      };
    },

    signTypedData: async () => {
      throw new Error("signTypedData is not supported for this smart account type");
    },
    signMessage: async () => {
      throw new Error("signMessage is not supported for this smart account type");
    },

    // Unsupported signing actions - throw errors
    signAuthorization: () => {
      throw new Error("signAuthorization is not supported for smart accounts");
    },
    prepareAuthorization: () => {
      throw new Error("prepareAuthorization is not supported for smart accounts");
    },

    // Override sendTransaction to use bundler
    // Note: We accept an extended parameter set with optional paymaster
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    sendTransaction: async (args: any) => {
      const account = await getSmartAccount();

      // Use per-transaction paymaster if provided, otherwise fall back to config
      const paymasterAddress = args.paymaster ?? normalizedPaymaster?.address;

      // For v0.8 EntryPoint, pass separate paymaster fields (not packed paymasterAndData)
      // These fields get included in the UserOp BEFORE signing
      const paymasterParams
        = normalizedPaymaster || paymasterAddress
          ? {
              paymaster: paymasterAddress!,
              paymasterData: normalizedPaymaster?.data ?? ("0x" as Hex),
              paymasterVerificationGasLimit: normalizedPaymaster?.verificationGasLimit ?? 500_000n,
              paymasterPostOpGasLimit: normalizedPaymaster?.postOpGasLimit ?? 1_000_000n,
            }
          : {};

      console.log("[smart-account-client-actions.ts sendTransaction] About to call bundler.sendUserOperation with paymaster params:", paymasterParams);

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
        ...paymasterParams,
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

      // For paymaster to be included in signature, use paymasterContext not callback
      const paymasterParams = normalizedPaymaster
        ? {
            paymaster: normalizedPaymaster.address,
            paymasterData: normalizedPaymaster.data ?? ("0x" as Hex),
            paymasterVerificationGasLimit: normalizedPaymaster.verificationGasLimit ?? 500_000n,
            paymasterPostOpGasLimit: normalizedPaymaster.postOpGasLimit ?? 1_000_000n,
          }
        : {};

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
        ...paymasterParams,
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
      webauthnValidatorAddress: Address;
    }) => {
      const account = await getSmartAccount();

      // Generate the transaction using the existing addPasskey function
      const { transaction } = addPasskeyAction({
        account: config.accountAddress,
        contracts: {
          webauthnValidator: params.webauthnValidatorAddress,
        },
        passkeySigner: {
          credentialId: params.credentialId,
          publicKey: params.publicKey,
          originDomain: params.originDomain,
        },
      });

      const paymasterParams = normalizedPaymaster
        ? {
            paymaster: normalizedPaymaster.address,
            paymasterData: normalizedPaymaster.data ?? ("0x" as Hex),
            paymasterVerificationGasLimit: normalizedPaymaster.verificationGasLimit ?? 500_000n,
            paymasterPostOpGasLimit: normalizedPaymaster.postOpGasLimit ?? 1_000_000n,
          }
        : {};

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
        ...paymasterParams,
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
