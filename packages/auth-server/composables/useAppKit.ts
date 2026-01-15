import { createAppKit } from "@reown/appkit/vue";
import { WagmiAdapter } from "@reown/appkit-adapter-wagmi";

// Track if AppKit has been initialized
let appKitInitialized = false;
let wagmiAdapterInstance: WagmiAdapter | null = null;

export const useAppKit = () => {
  const runtimeConfig = useRuntimeConfig();
  const { defaultChain } = useClientStore();

  const projectId = runtimeConfig.public.appKitProjectId;
  const origin = typeof window !== "undefined" ? window.location.origin : "https://auth.zksync.dev";

  const metadata = {
    name: "ZKsync SSO Auth Server",
    description: "ZKsync SSO Auth Server",
    url: origin,
    icons: [`${origin}/icon-512.png`],
  };

  // Create plain chain object to avoid Viem Proxy issues
  const plainChain = {
    id: defaultChain.id,
    name: defaultChain.name,
    nativeCurrency: {
      name: defaultChain.nativeCurrency.name,
      symbol: defaultChain.nativeCurrency.symbol,
      decimals: defaultChain.nativeCurrency.decimals,
    },
    rpcUrls: {
      default: {
        http: [...defaultChain.rpcUrls.default.http],
      },
    },
    blockExplorers: defaultChain.blockExplorers
      ? {
          default: {
            name: defaultChain.blockExplorers.default.name,
            url: defaultChain.blockExplorers.default.url,
          },
        }
      : undefined,
  };

  // Lazy initialization - only create AppKit when first used
  if (!appKitInitialized && typeof window !== "undefined") {
    try {
      wagmiAdapterInstance = new WagmiAdapter({
        networks: [plainChain],
        projectId,
      });

      createAppKit({
        adapters: [wagmiAdapterInstance],
        networks: [plainChain],
        projectId,
        metadata,
      });

      appKitInitialized = true;
    } catch (error) {
      console.warn("Failed to initialize AppKit:", error);
    }
  }

  const wagmiConfig = wagmiAdapterInstance?.wagmiConfig;

  return {
    metadata,
    projectId,
    wagmiAdapter: wagmiAdapterInstance,
    wagmiConfig,
  };
};
