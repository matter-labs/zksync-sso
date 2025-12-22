import {
  ChainNotConfiguredError,
  type Config,
  type Connector,
  createConnector,
  getConnections,
  type GetConnectorClientParameters,
} from "@wagmi/core";
import type { Compute } from "@wagmi/core/internal";
import {
  type Address,
  type Client,
  getAddress,
  SwitchChainError,
  toHex,
  UserRejectedRequestError,
} from "viem";
import type { BundlerClient } from "viem/account-abstraction";

import type { PaymasterConfig } from "../actions/sendUserOperation.js";
import type { SessionClient, SessionPreferences } from "../client/index.js";
import type { ProviderInterface } from "../client-auth-server/index.js";
import { WalletProvider } from "../client-auth-server/WalletProvider.js";
import type { AppMetadata, Communicator } from "../communicator/interface.js";
import { EthereumProviderError } from "../errors/errors.js";

export type ConnectorMetadata = {
  icon: string;
  id: string;
  name: string;
  type: string;
};

export type ZksyncSsoConnectorOptions = {
  metadata?: Partial<AppMetadata>;
  session?: SessionPreferences | (() => SessionPreferences | Promise<SessionPreferences>);
  bundlerClients?: Record<number, BundlerClient>;
  authServerUrl?: string;
  communicator?: Communicator;
  provider?: ProviderInterface;
  connectorMetadata?: ConnectorMetadata;
  paymaster?: Address | PaymasterConfig;
};

export const zksyncSsoConnector = (parameters: ZksyncSsoConnectorOptions) => {
  let walletProvider: ProviderInterface | undefined;

  let accountsChanged: Connector["onAccountsChanged"] | undefined;
  let chainChanged: Connector["onChainChanged"] | undefined;
  let disconnect: Connector["onDisconnect"] | undefined;

  const destroyWallet = () => {
    if (walletProvider) {
      if (accountsChanged) {
        walletProvider.removeListener("accountsChanged", accountsChanged);
        accountsChanged = undefined;
      }
      if (chainChanged) {
        walletProvider.removeListener("chainChanged", chainChanged);
        chainChanged = undefined;
      }
      if (disconnect) {
        walletProvider.removeListener("disconnect", disconnect);
        disconnect = undefined;
      }
    }
    walletProvider = undefined;
  };

  return createConnector<ProviderInterface>((config) => ({
    icon: parameters.connectorMetadata?.icon ?? "https://zksync.io/favicon.ico",
    id: parameters.connectorMetadata?.id ?? "zksync-sso",
    name: parameters.connectorMetadata?.name ?? "ZKsync",
    type: parameters.connectorMetadata?.type ?? "zksync-sso",
    // supportsSimulation: true,
    async connect({ chainId } = {}) {
      try {
        const provider = await this.getProvider();
        const accounts = (
          (await provider.request({
            method: "eth_requestAccounts",
          })) as string[]
        ).map((x) => getAddress(x));

        if (!accountsChanged) {
          accountsChanged = this.onAccountsChanged.bind(this);
          provider.on("accountsChanged", accountsChanged);
        }
        if (!chainChanged) {
          chainChanged = this.onChainChanged.bind(this);
          provider.on("chainChanged", chainChanged);
        }
        if (!disconnect) {
          disconnect = this.onDisconnect.bind(this);
          provider.on("disconnect", disconnect);
        }

        // Switch to chain if provided
        let walletChainId = await this.getChainId();
        if (chainId && walletChainId !== chainId) {
          const chain = await this.switchChain!({ chainId }).catch((error) => {
            if (error.code === UserRejectedRequestError.code) throw error;
            return { id: walletChainId };
          });
          walletChainId = chain?.id ?? walletChainId;
        }

        return { accounts, chainId: walletChainId };
      } catch (error) {
        console.error(`Error connecting to ${this.name}`, error);
        if (
          /(user closed modal|accounts received is empty|user denied account|request rejected)/i.test(
            (error as Error).message,
          )
        )
          throw new UserRejectedRequestError(error as Error);
        throw error;
      }
    },
    async disconnect() {
      const provider = await this.getProvider();
      provider.disconnect();
      destroyWallet();
    },
    async getAccounts() {
      const provider = await this.getProvider();
      return (
        await provider.request({
          method: "eth_accounts",
        })
      ).map((x) => getAddress(x));
    },
    async getChainId() {
      const provider = await this.getProvider();
      const chainId = await provider.request({
        method: "eth_chainId",
      });
      if (!chainId) return config.chains[0].id;
      return Number(chainId);
    },
    async getClient(parameters) {
      if (!walletProvider) throw new Error("Wallet provider not initialized");
      return walletProvider.getClient(parameters);
    },
    async getProvider() {
      if (!walletProvider) {
        // Normalize paymaster to PaymasterConfig format
        let paymasterConfig: PaymasterConfig | undefined;
        if (parameters.paymaster) {
          if (typeof parameters.paymaster === "string") {
            // Legacy format: just an address, use default gas limits for passkey
            paymasterConfig = {
              address: parameters.paymaster as Address,
              verificationGasLimit: 500_000n, // Default for passkey validation
              postOpGasLimit: 1_000_000n, // Default for post-operation
            };
          } else {
            // Full config provided
            paymasterConfig = parameters.paymaster;
          }
        }

        walletProvider = parameters.provider ?? new WalletProvider({
          metadata: {
            name: parameters.metadata?.name,
            icon: parameters.metadata?.icon,
            configData: parameters.metadata?.configData,
          },
          authServerUrl: parameters.authServerUrl,
          session: parameters.session,
          transports: config.transports,
          bundlerClients: parameters.bundlerClients,
          chains: config.chains,
          customCommunicator: parameters.communicator,
          paymaster: paymasterConfig,
        });
      }
      return walletProvider;
    },
    async isAuthorized() {
      try {
        const accounts = await this.getAccounts();
        return !!accounts.length;
      } catch {
        return false;
      }
    },
    async switchChain({ chainId }) {
      const chain = config.chains.find((chain) => chain.id === chainId);
      if (!chain) throw new SwitchChainError(new ChainNotConfiguredError());

      try {
        const provider = await this.getProvider();
        await provider.request({
          method: "wallet_switchEthereumChain",
          params: [{ chainId: toHex(chainId) }],
        });
        return chain;
      } catch (error) {
        throw new SwitchChainError(error as Error);
      }
    },
    onAccountsChanged(accounts) {
      if (!accounts.length) return;
      config.emitter.emit("change", {
        accounts: accounts.map((x) => getAddress(x)),
      });
    },
    onChainChanged(chain) {
      config.emitter.emit("change", { chainId: Number(chain) });
    },
    async onDisconnect(error) {
      config.emitter.emit("disconnect");
      if (error instanceof EthereumProviderError && error.code === 4900) return; // User initiated
      console.error("Account disconnected", error);
    },
    // Add a helper method to get the client directly (accessible via connector._getClient)
    _getClient(parameters?: { chainId?: number }) {
      if (!walletProvider) {
        throw new Error("Wallet provider not initialized. Please connect your wallet first.");
      }
      return walletProvider.getClient(parameters);
    },
  }));
};

export type GetConnectedSsoClientReturnType<
  config extends Config = Config,
  chainId extends config["chains"][number]["id"] = config["chains"][number]["id"],
> = Compute<
  SessionClient<
    config["_internal"]["transports"][chainId],
    Extract<config["chains"][number], { id: chainId }>
  >
>;

export const isSsoSessionClient = (client: Client): boolean => {
  // Accept both passkey and session clients
  return client.key === "zksync-sso-passkey-client"
    || client.key === "zksync-sso-session-client";
};

export const isSsoSessionClientConnected = async<
  config extends Config,
  chainId extends config["chains"][number]["id"],
>(
  config: config,
  parameters: GetConnectorClientParameters<config, chainId> = {},
): Promise<boolean> => {
  // Get the current connection
  const connections = getConnections(config);
  const connection = connections.find((c) => c.accounts.length > 0);

  if (!connection?.connector) {
    return false;
  }

  try {
    // Check if this is a ZKsync SSO connector with our custom _getClient method
    if (typeof (connection.connector as any)._getClient !== "function") {
      return false;
    }

    // Use the custom _getClient method to get our custom client
    const client = await (connection.connector as any)._getClient(parameters);
    return isSsoSessionClient(client);
  } catch {
    return false;
  }
};

export const getConnectedSsoSessionClient = async<
  config extends Config,
  chainId extends config["chains"][number]["id"],
>(
  config: config,
  parameters: GetConnectorClientParameters<config, chainId> = {},
): Promise<GetConnectedSsoClientReturnType<config, chainId>> => {
  const connections = getConnections(config);
  const connection = connections.find((c) => c.accounts.length > 0);

  if (!connection?.connector) {
    throw new Error("No active wallet connection found");
  }

  // Check if this is a ZKsync SSO connector with our custom _getClient method
  if (typeof (connection.connector as any)._getClient !== "function") {
    throw new Error("Connector does not support getClient method. Make sure you're using the ZKsync SSO connector.");
  }

  const client = await (connection.connector as any)._getClient(parameters);

  if (!isSsoSessionClient(client)) {
    throw new Error("ZKsync SSO Session Client not connected");
  }

  const sessionClient = client as unknown as GetConnectedSsoClientReturnType<config, chainId>;
  return sessionClient;
};
