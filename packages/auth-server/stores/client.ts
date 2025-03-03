import { type Address, createPublicClient, createWalletClient, custom, http, publicActions, walletActions } from "viem";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";
import { zksyncInMemoryNode, zksyncSepoliaTestnet } from "viem/chains";
import { eip712WalletActions } from "viem/zksync";
import { createZksyncPasskeyClient, type PasskeyRequiredContracts } from "zksync-sso/client/passkey";
import { createZksyncRecoveryGuardianClient } from "zksync-sso/client/recovery";

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
  [zksyncSepoliaTestnet.id]: {
    session: "0x64Bf5C3229CafF50e39Ec58C4BFBbE67bEA90B0F",
    passkey: "0x0F65cFE984d494DAa7165863f1Eb61C606e45fFb",
    recovery: "0xDf8F9b39Cd69Cb8Dc29137f83E89fE1AdA26912D",
    recoveryOidc: "0x3ad654fC38bb5Abe789c912cfE900A867d52A164",
    accountFactory: "0x73CFa70318FD25F2166d47Af9d93Cf72eED48724",
    accountPaymaster: "0xA46D949858335308859076FA605E773eB679e534",
  },
  [zksyncInMemoryNode.id]: {
    session: "0x644040Bc7f2b243BB5ba28ccFa67Ec3dD7f9a77F",
    passkey: "0x1Ec1126fab9eE89d0babC8669076e1dd1e36cd09",
    recovery: "0x4E619cA9DDb3A207E4764F3Ee5D36DD478212335",
    recoveryOidc: "0xcC31745F406Dc2f090DAd022D4d62801541a9358",
    accountFactory: "0x01F99512191c036FcA9Fcd416dE73b19e93B7D60",
    accountPaymaster: "0x865D411751Bdc5B59268cBD5BE8d48B6d6bEf1C5",
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
    credentialPublicKey: Uint8Array;
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
    const chain = supportedChains.find((chain) => chain.id === chainId);
    if (!chain) throw new Error(`Chain with id ${chainId} is not supported`);

    if (!window?.ethereum) throw new Error("No ethereum provider found");

    const accounts = await (window.ethereum as { request: (args: { method: string }) => Promise<Address[]> }).request({
      method: "eth_requestAccounts",
    });

    return createWalletClient({
      chain,
      account: accounts[0],
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      transport: custom(window!.ethereum as any),
    })
      .extend(publicActions)
      .extend(walletActions)
      .extend(eip712WalletActions());
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
