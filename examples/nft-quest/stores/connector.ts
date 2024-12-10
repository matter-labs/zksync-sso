import { connect, createConfig, type CreateConnectorFn, disconnect, getAccount, http, reconnect, watchAccount } from "@wagmi/core";
import { zksyncInMemoryNode, zksyncSepoliaTestnet } from "@wagmi/core/chains";
import { type Address, parseEther } from "viem";
import { callPolicy, zksyncSsoConnector } from "zksync-sso/connector";

import { Nft } from "~/abi";

export type SupportedChainId = (typeof supportedChains)[number]["id"];

const supportedChains = [
  zksyncSepoliaTestnet,
  zksyncInMemoryNode,
] as const;

export const useConnectorStore = defineStore("connector", () => {
  const runtimeConfig = useRuntimeConfig();

  const chainId = runtimeConfig.public.defaultChainId;
  const chain = supportedChains.filter((x) => x.id == chainId)[0];
  type SupportedChainId = (typeof supportedChains)[number]["id"];
  if (!chain) throw new Error(`Chain with id ${chainId} was not found in supported chains list`);

  const connector = zksyncSsoConnector({
    metadata: {
      icon: `${runtimeConfig.public.baseUrl}/icon-192.png`,
    },
    authServerUrl: runtimeConfig.public.authServerUrl,
    session: {
      feeLimit: parseEther("0.001"),
      contractCalls: [
        callPolicy({
          address: Nft.addressByChain[chainId],
          abi: Nft.Abi,
          functionName: "mint",
        }),
      ],
    },
  });
  const wagmiConfig = createConfig({
    chains: [chain],
    connectors: [connector as CreateConnectorFn],
    transports: (Object.fromEntries(supportedChains.map((chain) => [chain.id, http()]))) as Record<SupportedChainId, ReturnType<typeof http>>,
  });

  const account = ref(getAccount(wagmiConfig));
  const isConnected = computed(() => account.value.isConnected);
  const address = computed(() => account.value.address);
  reconnect(wagmiConfig);

  watchAccount(wagmiConfig, {
    onChange: async (updatedAccount) => {
      account.value = updatedAccount;
    },
  });

  const connectAccount = async () => {
    return await connect(wagmiConfig, {
      connector,
      chainId: chain.id,
    });
  };

  const disconnectAccount = () => {
    disconnect(wagmiConfig);
  };

  const shortAddress = computed(() => {
    if (!address.value) return null;
    return useTruncateAddress(address.value);
  });

  const { subscribe: address$, notify: notifyOnAccountChange } = useObservable<Address | undefined>();
  watch(address, (newAddress) => {
    notifyOnAccountChange(newAddress);
  });

  return {
    wagmiConfig: computed(() => wagmiConfig),
    account: computed(() => account.value),
    isConnected,
    connectAccount,
    disconnectAccount,
    address$,
    shortAddress,
  };
});
