import { WagmiAdapter } from "@reown/appkit-adapter-wagmi";

export const useAppKit = () => {
  const runtimeConfig = useRuntimeConfig();
  const { defaultChain } = useClientStore();

  const projectId = runtimeConfig.public.appKitProjectId;

  // Use fallback for SSR/build time when window is not available
  const origin = typeof window !== "undefined" ? window.location.origin : "https://auth.zksync.dev";

  const metadata = {
    name: "ZKsync SSO Auth Server",
    description: "ZKsync SSO Auth Server",
    url: origin,
    icons: [new URL("/icon-512.png", origin).toString()],
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
