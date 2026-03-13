import { defineChain } from "viem";

let _chain: ReturnType<typeof defineChain> | null = null;

export const useChain = () => {
  if (!_chain) {
    const runtimeConfig = useRuntimeConfig();
    _chain = defineChain({
      id: runtimeConfig.public.chainId,
      name: runtimeConfig.public.chainName || `Chain ${runtimeConfig.public.chainId}`,
      nativeCurrency: { name: "Ether", symbol: "ETH", decimals: 18 },
      rpcUrls: {
        default: { http: [runtimeConfig.public.chainRpcUrl || "http://localhost:5050"] },
      },
      ...(runtimeConfig.public.blockExplorerUrl
        ? {
            blockExplorers: {
              default: {
                name: "Explorer",
                url: runtimeConfig.public.blockExplorerUrl,
              },
            },
          }
        : {}),
    });
  }
  return _chain;
};
