import { connect, createConfig, type CreateConnectorFn, disconnect, getAccount, http, reconnect, watchAccount } from "@wagmi/core";
import { localhost } from "@wagmi/core/chains";
import { type Address, parseEther } from "viem";
import { zksyncSsoConnector } from "zksync-sso-4337/connector";

import { ZeekNftQuestAbi } from "@/abi/ZeekNFTQuest";

export const useConnectorStore = defineStore("connector", () => {
  const runtimeConfig = useRuntimeConfig();
  console.log("[ConnectorStore] runtimeConfig.public.contracts:", JSON.stringify(runtimeConfig.public.contracts));
  console.log("[ConnectorStore] paymaster from runtimeConfig:", runtimeConfig.public.contracts.paymaster);
  const supportedChains = [localhost] as const;
  const chain = supportedChains.filter((x) => x.id == runtimeConfig.public.chain.id)[0];
  type SupportedChainId = (typeof supportedChains)[number]["id"];
  if (!chain) throw new Error(`Chain with id ${runtimeConfig.public.chain.id} was not found in supported chains list`);

  // Session state - tracks whether a session has been added
  const hasSession = ref(false);

  // Session configuration for NFT minting
  const sessionConfig = {
    // Set a reasonable fee limit for the session
    feeLimit: parseEther("0.1"),
    // Allow calls to the NFT contract
    contractCalls: [
      {
        address: runtimeConfig.public.contracts.nft as Address,
        abi: ZeekNftQuestAbi,
        // No functionName = allow all functions (mint, safeTransferFrom, etc.)
      },
    ],
  };

  // Connector WITHOUT session - for initial account creation
  const connectorWithoutSession = zksyncSsoConnector({
    metadata: {
      icon: `${runtimeConfig.public.baseUrl}/icon-192.png`,
    },
    authServerUrl: runtimeConfig.public.authServerUrl,
    // No session config - just create account
    paymaster: runtimeConfig.public.contracts.paymaster as Address,
  });

  // Connector WITH session - for adding session to existing account
  const connectorWithSession = zksyncSsoConnector({
    metadata: {
      icon: `${runtimeConfig.public.baseUrl}/icon-192.png`,
    },
    authServerUrl: runtimeConfig.public.authServerUrl,
    session: sessionConfig,
    paymaster: runtimeConfig.public.contracts.paymaster as Address,
  });

  const wagmiConfig = createConfig({
    chains: [chain],
    connectors: [connectorWithoutSession as CreateConnectorFn, connectorWithSession as CreateConnectorFn],
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

  // Connect without session - for initial account creation
  const connectAccount = async () => {
    const result = await connect(wagmiConfig, {
      connector: connectorWithoutSession,
      chainId: chain.id,
    });
    return result;
  };

  // Connect with session - for adding session to existing account
  // Must disconnect first since wagmi won't allow a new connection when already connected
  const connectWithSession = async () => {
    // Disconnect from the current connector (without session)
    await disconnect(wagmiConfig);
    // Connect with the session-enabled connector
    const result = await connect(wagmiConfig, {
      connector: connectorWithSession,
      chainId: chain.id,
    });
    hasSession.value = true;
    return result;
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
    hasSession: computed(() => hasSession.value),
    connectAccount,
    connectWithSession,
    disconnectAccount,
    address$,
    shortAddress,
  };
});
