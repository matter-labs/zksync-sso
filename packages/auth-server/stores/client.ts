import { useAppKitProvider } from "@reown/appkit/vue";
import { type Address, createPublicClient, createWalletClient, custom, http, publicActions, walletActions } from "viem";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";
import { zksyncInMemoryNode, zksyncSepoliaTestnet } from "viem/chains";
import { eip712WalletActions } from "viem/zksync";
import { createZksyncPasskeyClient, type PasskeyRequiredContracts } from "zksync-sso/client/passkey";
import { createZksyncRecoveryGuardianClient } from "zksync-sso/client/recovery";

import eraSepoliaChainData from "./era-sepolia.json";
import localChainData from "./local-node.json";

export const supportedChains = [zksyncSepoliaTestnet, zksyncInMemoryNode];
export type SupportedChainId = (typeof supportedChains)[number]["id"];
export const blockExplorerUrlByChain: Record<SupportedChainId, string> = {
  [zksyncSepoliaTestnet.id]: zksyncSepoliaTestnet.blockExplorers.native.url,
  [zksyncInMemoryNode.id]: "http://localhost:3010",
};
export const blockExplorerApiByChain: Record<SupportedChainId, string> = {
  [zksyncSepoliaTestnet.id]: zksyncSepoliaTestnet.blockExplorers.native.blockExplorerApi,
  [zksyncInMemoryNode.id]: "http://localhost:3020",
};

type ChainContracts = PasskeyRequiredContracts & {
  accountFactory: NonNullable<PasskeyRequiredContracts["accountFactory"]>;
  accountPaymaster: Address;
};
export const contractsByChain: Record<SupportedChainId, ChainContracts> = {
  [zksyncSepoliaTestnet.id]: eraSepoliaChainData,
  [zksyncInMemoryNode.id]: localChainData,
};

export const chainParameters: Record<SupportedChainId, { blockTime: number }> = {
  [zksyncSepoliaTestnet.id]: {
    blockTime: 15,
  },
  [zksyncInMemoryNode.id]: {
    blockTime: 1,
  },
};

export const useClientStore = defineStore("client", () => {
  const runtimeConfig = useRuntimeConfig();
  const { address, username, passkey } = storeToRefs(useAccountStore());

  const defaultChainId = runtimeConfig.public.chainId as SupportedChainId;
  const defaultChain = supportedChains.find((chain) => chain.id === defaultChainId);
  if (!defaultChain)
    throw new Error(`Default chain is set to ${defaultChainId}, but is missing from the supported chains list`);

  const getPublicClient = ({ chainId }: { chainId: SupportedChainId }) => {
    const chain = supportedChains.find((chain) => chain.id === chainId);
    if (!chain) throw new Error(`Chain with id ${chainId} is not supported`);

    const client = createPublicClient({
      chain,
      transport: http(),
    });

    return client;
  };

  const getClient = ({ chainId }: { chainId: SupportedChainId }) => {
    if (!address.value) throw new Error("Address is not set");
    const chain = supportedChains.find((chain) => chain.id === chainId);
    if (!chain) throw new Error(`Chain with id ${chainId} is not supported`);
    const contracts = contractsByChain[chainId];

    const client = createZksyncPasskeyClient({
      address: address.value,
      credentialPublicKey: passkey.value!,
      userName: username.value!,
      userDisplayName: username.value!,
      contracts,
      chain,
      transport: http(),
    });

    return client;
  };

  const getRecoveryClient = ({ chainId, address }: { chainId: SupportedChainId; address: Address }) => {
    const chain = supportedChains.find((chain) => chain.id === chainId);
    if (!chain) throw new Error(`Chain with id ${chainId} is not supported`);
    const contracts = contractsByChain[chainId];

    const client = createZksyncRecoveryGuardianClient({
      address,
      contracts,
      chain: chain,
      transport: http(),
    });

    return client;
  };

  const getConfigurableClient = ({
    chainId,
    address,
    credentialPublicKey,
    username,
  }: {
    chainId: SupportedChainId;
    address: Address;
    credentialPublicKey: Uint8Array<ArrayBufferLike>;
    username: string;
  }) => {
    const chain = supportedChains.find((chain) => chain.id === chainId);
    if (!chain) throw new Error(`Chain with id ${chainId} is not supported`);
    const contracts = contractsByChain[chainId];
    return createZksyncPasskeyClient({
      address,
      credentialPublicKey,
      userName: username,
      userDisplayName: username,
      contracts,
      chain,
      transport: http(),
    });
  };

  const getThrowAwayClient = ({ chainId }: { chainId: SupportedChainId }) => {
    const chain = supportedChains.find((chain) => chain.id === chainId);
    if (!chain) throw new Error(`Chain with id ${chainId} is not supported`);

    const throwAwayClient = createWalletClient({
      account: privateKeyToAccount(generatePrivateKey()),
      chain,
      transport: http(),
    })
      .extend(publicActions)
      .extend(walletActions)
      .extend(eip712WalletActions());
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

    return createWalletClient({
      chain,
      account: accounts[0],
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      transport: custom(provider as any),
    })
      .extend(publicActions)
      .extend(walletActions)
      .extend(eip712WalletActions());
    ;
  };

  return {
    defaultChain,
    getPublicClient,
    getClient,
    getThrowAwayClient,
    getWalletClient,
    getRecoveryClient,
    getConfigurableClient,
  };
});
