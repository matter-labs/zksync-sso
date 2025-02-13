import { WagmiAdapter } from "@reown/appkit-adapter-wagmi";

export const useAppKit = () => {
  const runtimeConfig = useRuntimeConfig();
  const { defaultChain } = useClientStore();

  const projectId = runtimeConfig.public.appKitProjectId;
  const metadata = {
    name: "ZKsync SSO Auth Server",
    description: "ZKsync SSO Auth Server",
    url: runtimeConfig.public.appUrl,
    icons: [`${runtimeConfig.public.appUrl}/icon-512.png`],
  };

  const wagmiAdapter = new WagmiAdapter({
    networks: [defaultChain],
    projectId,
  });

  const wagmiConfig = wagmiAdapter.wagmiConfig;

  return {
    metadata,
    projectId,
    wagmiAdapter,
    wagmiConfig,
  };
};
