import { createAppKit } from "@reown/appkit/vue";

export default defineNuxtPlugin(async () => {
  const { defaultChain } = useClientStore();

  // Dynamic import to ensure browser globals are available
  const { WagmiAdapter } = await import("@reown/appkit-adapter-wagmi");
  const runtimeConfig = useRuntimeConfig();

  const projectId = runtimeConfig.public.appKitProjectId;
  const origin = window.location.origin;

  const metadata = {
    name: "ZKsync SSO Auth Server",
    description: "ZKsync SSO Auth Server",
    url: origin,
    icons: [new URL("/icon-512.png", origin).toString()],
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

  const wagmiAdapter = new WagmiAdapter({
    networks: [plainChain],
    projectId,
  });

  createAppKit({
    adapters: [wagmiAdapter],
    networks: [plainChain],
    projectId,
    metadata,
  });
});
