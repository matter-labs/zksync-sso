import { useAppKitProvider } from "@reown/appkit/vue";
import { type Address, createPublicClient, createWalletClient, custom, defineChain, type Hex, http, publicActions, walletActions } from "viem";
import { createBundlerClient } from "viem/account-abstraction";
import { /* generatePrivateKey, */ generatePrivateKey, privateKeyToAccount } from "viem/accounts";
import { localhost } from "viem/chains";
import { createPasskeyClient } from "zksync-sso-4337/client";

// TODO: OIDC and guardian recovery are not yet available in sdk-4337
// import { createZkSyncOidcClient, type ZkSyncSsoClient } from "zksync-sso/client/oidc";
// import { createZksyncRecoveryGuardianClient } from "zksync-sso/client/recovery";
import localChainData from "./local-node.json";

const zksyncOsTestnet = defineChain({
  id: 8022833,
  name: "ZKsyncOS Testnet",
  nativeCurrency: {
    name: "Ether",
    symbol: "ETH",
    decimals: 18,
  },
  rpcUrls: {
    default: {
      http: ["https://zksync-os-testnet-alpha.zksync.dev"],
    },
  },
  blockExplorers: {
    default: {
      name: "ZKsyncOS Testnet Explorer",
      url: "https://zksync-os-testnet-alpha.staging-scan-v2.zksync.dev",
    },
  },
});
const zksyncOsLocal = defineChain({
  id: 6565,
  name: "ZKsyncOS Local",
  nativeCurrency: {
    name: "Ether",
    symbol: "ETH",
    decimals: 18,
  },
  rpcUrls: {
    default: {
      http: ["http://localhost:3050"],
    },
  },
});

export const supportedChains = [localhost, zksyncOsTestnet, zksyncOsLocal];
export type SupportedChainId = (typeof supportedChains)[number]["id"];
export const blockExplorerUrlByChain: Record<SupportedChainId, string> = {
  [localhost.id]: "http://localhost:3010",
  [zksyncOsTestnet.id]: "https://zksync-os-testnet-alpha.staging-scan-v2.zksync.dev",
  [zksyncOsLocal.id]: "",
};
export const blockExplorerApiByChain: Record<SupportedChainId, string> = {
  [localhost.id]: "http://localhost:3020",
  [zksyncOsTestnet.id]: "https://block-explorer-api.zksync-os-testnet-alpha.zksync.dev/api",
  [zksyncOsLocal.id]: "",
};

type ChainContracts = {
  eoaValidator: Address;
  webauthnValidator: Address;
  sessionValidator: Address;
  factory: Address;
  bundlerUrl?: string;
  beacon?: Address; // Optional, for deployment
  testPaymaster?: Address; // Optional, for paymaster sponsorship
};

export const contractsByChain: Record<SupportedChainId, ChainContracts> = {
  [localhost.id]: localChainData as ChainContracts,
  [zksyncOsLocal.id]: localChainData as ChainContracts,
  [zksyncOsTestnet.id]: {
    eoaValidator: "0x3497392f9662Da3de1EC2AfE8724CdBF6b884088",
    webauthnValidator: "0xa5C2c5C723239C0cD11a5691954CdAC4369C874b",
    sessionValidator: "0x2bF3B894aA2C13A1545C6982bBbee435B5168b52",
    factory: "0x757b5c9854d327A6B76840c996dfAac0F6b3Dc1f",
    bundlerUrl: "https://bundler-api.stage-sso.zksync.dev",
    beacon: "0x1D779D791B55a093dE60da664C3F301a87f96C62",
  },
};

export const chainParameters: Record<SupportedChainId, { blockTime: number }> = {
  [localhost.id]: {
    blockTime: 1,
  },
  [zksyncOsLocal.id]: {
    blockTime: 1,
  },
  [zksyncOsTestnet.id]: {
    blockTime: 1,
  },
};

export const useClientStore = defineStore("client", () => {
  const runtimeConfig = useRuntimeConfig();
  const { address, credentialId } = storeToRefs(useAccountStore());
  const prividiumAuthStore = usePrividiumAuthStore();

  const defaultChainId = runtimeConfig.public.chainId as SupportedChainId;
  const defaultChain = supportedChains.find((chain) => chain.id === defaultChainId);
  if (!defaultChain)
    throw new Error(`Default chain is set to ${defaultChainId}, but is missing from the supported chains list`);

  // Create transport with or without authentication based on Prividium mode
  const createTransport = () => {
    if (runtimeConfig.public.prividiumMode) {
      const prividiumTransport = prividiumAuthStore.getTransport();
      if (!prividiumTransport) {
        throw new Error("Prividium transport not available. User may need to authenticate.");
      }
      return prividiumTransport;
    }

    return http();
  };

  const getPublicClient = ({ chainId }: { chainId: SupportedChainId }) => {
    const chain = supportedChains.find((chain) => chain.id === chainId);
    if (!chain) throw new Error(`Chain with id ${chainId} is not supported`);

    const client = createPublicClient({
      chain,
      transport: createTransport(),
    });

    return client;
  };

  const getBundlerClient = ({ chainId }: { chainId: SupportedChainId }) => {
    const chain = supportedChains.find((chain) => chain.id === chainId);
    if (!chain) throw new Error(`Chain with id ${chainId} is not supported`);
    const contracts = contractsByChain[chainId];
    const publicClient = getPublicClient({ chainId });

    return createBundlerClient({
      client: publicClient,
      chain,
      transport: http(contracts.bundlerUrl || "http://localhost:4337"),
      userOperation: {
        async estimateFeesPerGas() {
          const feesPerGas = await publicClient.estimateFeesPerGas();
          return {
            callGasLimit: 2_000_000n,
            verificationGasLimit: 2_000_000n,
            preVerificationGas: 1_000_000n,
            ...feesPerGas,
          } as const;
        },
      },
    });
  };

  const getClient = ({ chainId = defaultChain.id, usePaymaster = false }: { chainId?: SupportedChainId; usePaymaster?: boolean } = {}) => {
    if (!address.value) throw new Error("Address is not set");
    if (!credentialId.value) throw new Error("Credential ID is not set");
    const chain = supportedChains.find((chain) => chain.id === chainId);
    if (!chain) throw new Error(`Chain with id ${chainId} is not supported`);
    const contracts = contractsByChain[chainId];
    const bundlerClient = getBundlerClient({ chainId });

    const client = createPasskeyClient({
      account: {
        address: address.value,
        validatorAddress: contracts.webauthnValidator,
        credentialId: credentialId.value,
        rpId: window.location.hostname,
        origin: window.location.origin,
      },
      bundlerClient,
      chain,
      transport: createTransport(),
      paymaster: usePaymaster ? contracts.testPaymaster : undefined,
    });

    return client;
  };

  // TODO: Guardian recovery not yet available in sdk-4337
  // const getRecoveryClient = ({ chainId, address }: { chainId: SupportedChainId; address: Address }) => {
  //   const chain = supportedChains.find((chain) => chain.id === chainId);
  //   if (!chain) throw new Error(`Chain with id ${chainId} is not supported`);
  //   const contracts = contractsByChain[chainId];
  //
  //   const client = createZksyncRecoveryGuardianClient({
  //     address,
  //     contracts,
  //     chain: chain,
  //     transport: createTransport(),
  //   });
  //
  //   return client;
  // };

  // TODO: OIDC client not yet available in sdk-4337
  // const getOidcClient = ({ chainId, address }: { chainId: SupportedChainId; address: Address }): ZkSyncSsoClient => {
  //   const chain = supportedChains.find((chain) => chain.id === chainId);
  //   if (!chain) throw new Error(`Chain with id ${chainId} is not supported`);
  //   const contracts = contractsByChain[chainId];
  //
  //   return createZkSyncOidcClient({
  //     address,
  //     contracts,
  //     chain: chain,
  //     transport: http(),
  //   });
  // };

  const getConfigurableClient = ({
    chainId,
    address,
    credentialId,
    usePaymaster = false,
  }: {
    chainId: SupportedChainId;
    address: Address;
    credentialId: Hex;
    usePaymaster?: boolean;
  }) => {
    const chain = supportedChains.find((chain) => chain.id === chainId);
    if (!chain) throw new Error(`Chain with id ${chainId} is not supported`);
    const contracts = contractsByChain[chainId];
    const bundlerClient = getBundlerClient({ chainId });

    return createPasskeyClient({
      account: {
        address,
        validatorAddress: contracts.webauthnValidator,
        credentialId,
        rpId: window.location.hostname,
        origin: window.location.origin,
      },
      bundlerClient,
      chain,
      transport: createTransport(),
      paymaster: usePaymaster ? contracts.testPaymaster : undefined,
    });
  };

  const getThrowAwayClient = ({ chainId }: { chainId: SupportedChainId }) => {
    const chain = supportedChains.find((chain) => chain.id === chainId);
    if (!chain) throw new Error(`Chain with id ${chainId} is not supported`);

    const throwAwayClient = createWalletClient({
      account: privateKeyToAccount(generatePrivateKey()),
      chain,
      transport: createTransport(),
    })
      .extend(publicActions)
      .extend(walletActions);
    return throwAwayClient;
  };

  const getWalletClient = async ({ chainId }: { chainId: SupportedChainId }) => {
    const accountProvider = useAppKitProvider("eip155");
    const chain = supportedChains.find((chain) => chain.id === chainId);
    if (!chain) throw new Error(`Chain with id ${chainId} is not supported`);

    if (!accountProvider.walletProvider) throw new Error("No ethereum provider found");

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const provider = accountProvider.walletProvider as any;
    const accounts = await (provider as { request: (args: { method: string }) => Promise<Address[]> }).request({
      method: "eth_requestAccounts",
    });

    // For wallet client, we use the provider's transport, not our authenticated transport
    // as this is for external wallet connections
    return createWalletClient({
      chain,
      account: accounts[0],
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      transport: custom(provider as any),
    })
      .extend(publicActions)
      .extend(walletActions);
  };

  return {
    defaultChain,
    createTransport,
    getPublicClient,
    getBundlerClient,
    getClient,
    getThrowAwayClient,
    getWalletClient,
    // getRecoveryClient, // TODO: Not available in sdk-4337
    getConfigurableClient,
    // getOidcClient, // TODO: Not available in sdk-4337
  };
});
