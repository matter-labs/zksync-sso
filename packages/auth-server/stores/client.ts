import { useAppKitProvider } from "@reown/appkit/vue";
import { type Address, createPublicClient, createWalletClient, custom, defineChain, type Hex, http, publicActions, walletActions } from "viem";
import { createBundlerClient } from "viem/account-abstraction";
import { /* generatePrivateKey, */ generatePrivateKey, privateKeyToAccount } from "viem/accounts";
import { localhost } from "viem/chains";
import { createPasskeyClient } from "zksync-sso-4337/client";

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

const dawnMainnet = defineChain({
  id: 30715,
  name: "Dawn Mainnet",
  nativeCurrency: {
    name: "Ether",
    symbol: "ETH",
    decimals: 18,
  },
  rpcUrls: {
    default: {
      http: ["https://zksync-os-mainnet-dawn.zksync.io"],
    },
  },
  blockExplorers: {
    default: {
      name: "Dawn Mainnet Explorer",
      url: "https://zksync-os-mainnet-dawn.staging-scan-v2.zksync.dev",
    },
  },
});

export const supportedChains = [localhost, zksyncOsTestnet, zksyncOsLocal, dawnMainnet];
export type SupportedChainId = (typeof supportedChains)[number]["id"];
export const blockExplorerUrlByChain: Record<SupportedChainId, string> = {
  [localhost.id]: "http://localhost:3010",
  [zksyncOsTestnet.id]: "https://zksync-os-testnet-alpha.staging-scan-v2.zksync.dev",
  [zksyncOsLocal.id]: "",
  [dawnMainnet.id]: "https://zksync-os-mainnet-dawn.staging-scan-v2.zksync.dev",
};
export const blockExplorerApiByChain: Record<SupportedChainId, string> = {
  [localhost.id]: "http://localhost:3020",
  [zksyncOsTestnet.id]: "https://block-explorer-api.zksync-os-testnet-alpha.zksync.dev/api",
  [zksyncOsLocal.id]: "",
  [dawnMainnet.id]: "https://block-explorer-api.zksync-os-mainnet-dawn.zksync.io",
};

type ChainContracts = {
  eoaValidator: Address;
  webauthnValidator: Address;
  sessionValidator: Address;
  factory: Address;
  bundlerUrl?: string;
  beacon?: Address; // Optional, for deployment
  testPaymaster?: Address; // Optional, for paymaster sponsorship
  recovery?: Address; // Recovery module (legacy SDK)
  guardianExecutor?: Address; // Guardian executor module (ERC-4337)
  accountPaymaster?: Address; // Paymaster for account operations
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
  [dawnMainnet.id]: {
    eoaValidator: "0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512",
    webauthnValidator: "0x5FC8d32690cc91D4c39d9d3abcBD16989F875707",
    sessionValidator: "0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9",
    factory: "0xB7f8BC63BbcaD18155201308C8f3540b07f84F5e",
    bundlerUrl: "https://bundler-api.dawn-mainnet.zksync.io", // Update with actual bundler URL when available
    beacon: "0x8A791620dd6260079BF849Dc5567aDC3F2FdC318",
    guardianExecutor: "0xa513E6E4b8f2a923D98304ec87F64353C4D5C853",
    accountPaymaster: "0x959922bE3CAee4b8Cd9a407cc3ac1C251C2007B1", // MockPaymaster
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
  [dawnMainnet.id]: {
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

    // In prividium mode, use prividium transport for bundler as well
    // Get transport from existing prividium instance - it auto-routes bundler methods
    const bundlerTransport = runtimeConfig.public.prividiumMode
      ? (() => {
          const prividiumTransport = prividiumAuthStore.getTransport();
          if (!prividiumTransport) {
            throw new Error("Prividium transport not available. User may need to authenticate.");
          }
          return prividiumTransport;
        })()
      : http(contracts.bundlerUrl || "http://localhost:4337");

    return createBundlerClient({
      client: publicClient,
      chain,
      transport: bundlerTransport,
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

  const getClient = ({ chainId = defaultChain.id, usePaymaster = false, paymasterAddress }: { chainId?: SupportedChainId; usePaymaster?: boolean; paymasterAddress?: string } = {}) => {
    if (!address.value) throw new Error("Address is not set");
    if (!credentialId.value) throw new Error("Credential ID is not set");
    const chain = supportedChains.find((chain) => chain.id === chainId);
    if (!chain) throw new Error(`Chain with id ${chainId} is not supported`);
    const contracts = contractsByChain[chainId];
    const bundlerClient = getBundlerClient({ chainId });

    const finalPaymasterAddress = paymasterAddress as Address | undefined ?? (usePaymaster ? contracts.testPaymaster : undefined);

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
      paymaster: finalPaymasterAddress,
    });

    return client;
  };

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
    getConfigurableClient,
    contractsByChain,
    // getOidcClient, // TODO: Not available in sdk-4337
  };
});
