import { type Address, type Chain, createPublicClient, createWalletClient, custom, type Hash, http, type RpcSchema as RpcSchemaGeneric, type SendTransactionParameters, type Transport, type WalletClient } from "viem";
import { type BundlerClient, createBundlerClient } from "viem/account-abstraction";
import type { CustomPaymasterHandler } from "zksync-sso/paymaster";

import { createSessionClient, type SessionClient } from "../client/session/client.js";
import type { Communicator } from "../communicator/index.js";
import type { AppMetadata, RequestArguments } from "./interface.js";
import type { AuthServerRpcSchema, ExtractParams, ExtractReturnType, Method, RPCRequestMessage, RPCResponseMessage, RpcSchema } from "./rpc.js";
import type { SessionPreferences } from "./session/index.js";
import { parseSessionConfigJSON, type SessionConfigJSON } from "./session/utils.js";
import { StorageItem, type StorageLike } from "./utils/index.js";

type Account = {
  address: Address;
  activeChainId: Chain["id"];
  session?: {
    sessionKey: Hash;
    sessionConfig: SessionConfigJSON;
  };
};

interface SignerInterface {
  accounts: Address[];
  chain: Chain;
  getClient(parameters?: { chainId?: number }): SessionClient | WalletClient;
  handshake(): Promise<Address[]>;
  request<TMethod extends Method>(request: RequestArguments<TMethod>): Promise<ExtractReturnType<TMethod>>;
  disconnect: () => Promise<void>;
}

type UpdateListener = {
  onAccountsUpdate: (_: Address[]) => void;
  onChainUpdate: (_: number) => void;
};

type SignerConstructorParams = {
  metadata: () => AppMetadata;
  communicator: Communicator;
  updateListener: UpdateListener;
  chains: readonly Chain[];
  transports?: Record<number, Transport>;
  bundlerClients?: Record<number, BundlerClient>;
  session?: () => SessionPreferences | Promise<SessionPreferences>;
  // onSessionStateChange?: (event: { address: Address; chainId: number; state: SessionStateEvent }) => void;
  skipPreTransactionStateValidation?: boolean; // Useful if you want to send session transactions really fast
  storage?: StorageLike;
  paymasterHandler?: CustomPaymasterHandler;
};

type ChainsInfo = ExtractReturnType<"eth_requestAccounts", AuthServerRpcSchema>["chainsInfo"];

export class Signer implements SignerInterface {
  private readonly getMetadata: () => AppMetadata;
  private readonly communicator: Communicator;
  private readonly updateListener: UpdateListener;
  private readonly chains: readonly Chain[];
  private readonly transports: Record<number, Transport> = {};
  private readonly bundlerClients: Record<number, BundlerClient> = {};
  private readonly sessionParameters?: () => (SessionPreferences | Promise<SessionPreferences>);
  private readonly paymasterHandler?: CustomPaymasterHandler;
  // private readonly onSessionStateChange?: SignerConstructorParams["onSessionStateChange"];
  // private readonly skipPreTransactionStateValidation?: boolean;

  private _account: StorageItem<Account | null>;
  private _chainsInfo: StorageItem<ChainsInfo>;
  private client: { instance: SessionClient; type: "session" } | { instance: WalletClient; type: "auth-server" } | undefined;

  constructor({ metadata, communicator, updateListener, session, chains, transports, bundlerClients, /* onSessionStateChange, skipPreTransactionStateValidation, */ storage, paymasterHandler }: SignerConstructorParams) {
    if (!chains.length) throw new Error("At least one chain must be included in the config");

    this.getMetadata = metadata;
    this.communicator = communicator;
    this.updateListener = updateListener;
    this.sessionParameters = session;
    this.chains = chains;
    this.transports = transports || {};
    this.bundlerClients = bundlerClients || {};
    this.paymasterHandler = paymasterHandler;
    // this.onSessionStateChange = onSessionStateChange;
    // this.skipPreTransactionStateValidation = skipPreTransactionStateValidation;

    this._chainsInfo = new StorageItem<ChainsInfo>(StorageItem.scopedStorageKey("chainsInfo"), [], { storage });
    this._account = new StorageItem<Account | null>(StorageItem.scopedStorageKey("account"), null, {
      onChange: (newValue) => {
        if (newValue) {
          this.updateListener.onAccountsUpdate([newValue.address]);
          this.updateListener.onChainUpdate(newValue.activeChainId);
          this.createWalletClient();
        } else {
          this.updateListener.onAccountsUpdate([]);
        }
      },
      storage,
    });
    try {
      if (this.account) this.createWalletClient();
    } catch (error) {
      console.error("Failed to create wallet client", error);
      console.error("Logging out to prevent crash loop");
      this.clearState();
    }
  }

  get walletClient() {
    return this.client?.instance as WalletClient | undefined;
  }

  getClient(parameters?: { chainId?: number }) {
    const chainId = parameters?.chainId || this.chain.id;
    const chain = this.chains.find((e) => e.id === chainId);
    if (!chain) throw new Error(`Chain with id ${chainId} is not supported`);

    if (!this.walletClient) throw new Error("Wallet client is not created");
    return this.walletClient;
  }

  private get account(): Account | null {
    const account = this._account.get();
    if (!account) return null;
    const chain = this.chains.find((e) => e.id === account.activeChainId);
    return {
      ...account,
      activeChainId: chain?.id || this.chains[0]!.id,
    };
  }

  private get session() { return this.account?.session; }
  private get chainsInfo() { return this._chainsInfo.get(); }
  private readonly clearState = () => {
    this._account.remove();
    this._chainsInfo.remove();
  };

  public get accounts() { return this.account ? [this.account.address] : []; }
  public get chain() {
    const chainId = this.account?.activeChainId || this.chains[0]!.id;
    return this.chains.find((e) => e.id === chainId)!;
  }

  private getBundlerClient(chainId: number): BundlerClient | undefined {
    // Return existing bundler client if already created
    if (this.bundlerClients[chainId]) {
      return this.bundlerClients[chainId];
    }

    // Try to create bundler client from chainsInfo if bundlerUrl is available
    const chainInfo = this.chainsInfo.find((c) => c.id === chainId);
    if (!chainInfo?.bundlerUrl) {
      return undefined;
    }

    const chain = this.chains.find((c) => c.id === chainId);
    if (!chain) {
      return undefined;
    }

    const publicClient = createPublicClient({
      chain,
      transport: this.transports[chain.id] || http(),
    });

    this.bundlerClients[chain.id] = createBundlerClient({
      client: publicClient,
      chain,
      transport: http(chainInfo.bundlerUrl),
      userOperation: {
        // Use fixed gas values matching old Rust SDK implementation
        // (old SDK used: 2M callGas, 2M verificationGas, 1M preVerificationGas)
        async estimateFeesPerGas() {
          const feesPerGas = await publicClient.estimateFeesPerGas();
          return {
            callGasLimit: 2_000_000n,
            verificationGasLimit: 2_000_000n,
            preVerificationGas: 1_000_000n,
            ...feesPerGas,
          };
        },
      },
    });

    return this.bundlerClients[chain.id];
  }

  createWalletClient() {
    const session = this.session;
    const chain = this.chain;
    const chainInfo = this.chainsInfo.find((e) => e.id === chain.id);
    if (!this.account) throw new Error("Account is not set");
    if (!chainInfo) throw new Error(`Chain info for ${chain} wasn't set during handshake`);
    if (session) {
      const bundlerClient = this.getBundlerClient(chain.id);
      if (!bundlerClient) {
        throw new Error(`Bundler client is required for session client but not provided for chain ${chain.id}. Make sure bundlerUrl is set in chainsInfo or pass bundlerClients in constructor.`);
      }

      this.client = {
        type: "session",
        instance: createSessionClient({
          address: this.account.address,
          sessionKeyPrivateKey: session.sessionKey,
          sessionSpec: parseSessionConfigJSON(session.sessionConfig),
          contracts: chainInfo.contracts,
          bundlerClient,
          chain,
          transport: this.transports[chain.id] || http(),
          paymasterHandler: this.paymasterHandler,
          /* onSessionStateChange: (event: SessionStateEvent) => {
            if (!this.onSessionStateChange) return;
            this.onSessionStateChange({
              state: event,
              address: this.account!.address,
              chainId: chain.id,
            });
          }, */
          // skipPreTransactionStateValidation: this.skipPreTransactionStateValidation,
        }),
      };
    } else {
      this.client = {
        type: "auth-server",
        instance: createWalletClient({
          key: "zksync-sso-auth-server-wallet",
          account: this.account.address,
          chain,
          transport: custom({
            request: this.request.bind(this),
          }),
        }),
      };
    }
  }

  async handshake(): Promise<Address[]> {
    let sessionPreferences: SessionPreferences | undefined;
    let metadata: AppMetadata = {
      name: "Unknown DApp",
      icon: null,
      configData: {},
    };
    try {
      metadata = this.getMetadata();
    } catch (error) {
      console.error("Failed to get website metadata. Proceeding with default one.", error);
    }
    if (this.sessionParameters) {
      try {
        sessionPreferences = await this.sessionParameters();
      } catch (error) {
        console.error("Failed to get session data. Proceeding connection with no session.", error);
      }
    }
    const responseMessage = await this.sendRpcRequest<"eth_requestAccounts", AuthServerRpcSchema>({
      method: "eth_requestAccounts",
      params: {
        metadata,
        sessionPreferences,
      },
    });
    const handshakeData = responseMessage.content.result!;

    this._chainsInfo.set(handshakeData.chainsInfo);
    this._account.set({
      address: handshakeData.account.address,
      activeChainId: handshakeData.account.activeChainId || this.chain.id,
      session: handshakeData.account.session,
    });
    return this.accounts;
  }

  switchChain(chainId: number): boolean {
    const chain = this.chains.find((chain) => chain.id === chainId);
    const chainInfo = this.chainsInfo.find((e) => e.id === chainId);
    if (!chainInfo) {
      console.error(`Chain ${chainId} is not supported or chain info was not set during handshake`);
      return false;
    };
    if (!chain) {
      console.error(`Chain ${chainId} is missing in the configuration`);
      return false;
    };
    if (chain.id === this.chain.id) return true;

    this._account.set({
      ...this.account!,
      activeChainId: chain.id,
    });
    return true;
  }

  async request<TMethod extends Method>(request: RequestArguments<TMethod>): Promise<ExtractReturnType<TMethod>> {
    const localResult = await this.tryLocalHandling(request);
    if (localResult !== undefined) return localResult;

    const response = await this.sendRpcRequest(request);
    return response.content.result as ExtractReturnType<TMethod>;
  }

  async disconnect() {
    this.clearState();
  }

  private async tryLocalHandling<TMethod extends Method>(request: RequestArguments<TMethod>): Promise<ExtractReturnType<TMethod> | undefined> {
    const client = this.walletClient;
    const originalClient = this.client;

    switch (request.method) {
      case "eth_estimateGas": {
        if (!client) return undefined;
        const params = request.params as ExtractParams<"eth_estimateGas">;
        const res = await client.request({ method: request.method, params: params });
        return res as ExtractReturnType<TMethod>;
      }
      case "eth_sendTransaction": {
        if (originalClient?.type !== "session") return undefined;
        const params = request.params as ExtractParams<"eth_sendTransaction">;
        const transactionRequest = params[0];
        const res = await originalClient.instance.sendTransaction(transactionRequest as unknown as SendTransactionParameters);
        return res as ExtractReturnType<TMethod>;
      }
      case "wallet_switchEthereumChain": {
        throw new Error("Chain switching is not supported yet");
        // const params = request.params as ExtractParams<"wallet_switchEthereumChain">;
        // const chainId = params[0].chainId;
        // const switched = this.switchChain(typeof chainId === "string" ? hexToNumber(chainId as Hash) : chainId);
        // return switched ? (null as ExtractReturnType<TMethod>) : undefined;
      }
      case "wallet_getCapabilities": {
        const chainInfo = this.chainsInfo.find((e) => e.id === this.chain.id);
        if (!chainInfo) throw new Error("Chain info is not set");
        return { [this.chain.id]: chainInfo.capabilities } as ExtractReturnType<TMethod>;
      }
      case "eth_accounts": {
        return this.accounts as ExtractReturnType<TMethod>;
      }
      default:
        return undefined;
    }
  }

  private async sendRpcRequest<
    TMethod extends Method<TSchema>,
    TSchema extends RpcSchemaGeneric = RpcSchema,
  >(request: RequestArguments<TMethod, TSchema>): Promise<RPCResponseMessage<ExtractReturnType<TMethod, TSchema>>> {
    // Open popup immediately to make sure popup won't be blocked by Safari
    await this.communicator.ready();

    const message = this.createRequestMessage<TMethod, TSchema>({
      action: request,
      chainId: this.chain.id,
    });
    const response: RPCResponseMessage<ExtractReturnType<TMethod, TSchema>>
      = await this.communicator.postRequestAndWaitForResponse(message);

    const content = response.content;
    if ("error" in content) throw content.error;

    return response;
  }

  private createRequestMessage<
    TMethod extends Method<TSchema>,
    TSchema extends RpcSchemaGeneric = RpcSchema,
  >(content: RPCRequestMessage<TMethod, TSchema>["content"]): RPCRequestMessage<TMethod, TSchema> {
    return {
      id: crypto.randomUUID(),
      content,
    };
  }
}
