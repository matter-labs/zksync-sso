import { createAppKit } from "@reown/appkit/vue";
import { WagmiAdapter } from "@reown/appkit-adapter-wagmi";

// Singleton initialization state
let initializationPromise: Promise<void> | null = null;
let wagmiAdapterInstance: WagmiAdapter | null = null;

// Cached metadata and chain configuration
let cachedMetadata: ReturnType<typeof createMetadata> | null = null;
let cachedPlainChain: ReturnType<typeof createPlainChain> | null = null;

/**
 * Creates metadata configuration for AppKit.
 * Cached to avoid recreation on every call.
 */
function createMetadata(origin: string) {
  if (!cachedMetadata) {
    cachedMetadata = {
      name: "ZKsync SSO Auth Server",
      description: "ZKsync SSO Auth Server",
      url: origin,
      icons: [`${origin}/icon-512.png`],
    };
  }
  return cachedMetadata;
}

/**
 * Creates plain chain object to avoid Viem Proxy serialization issues.
 * Cached to avoid recreation and unnecessary property access on defaultChain.
 */
function createPlainChain(defaultChain: ReturnType<typeof useClientStore>["defaultChain"]) {
  if (!cachedPlainChain) {
    cachedPlainChain = {
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
  }
  return cachedPlainChain;
}

/**
 * Initializes AppKit with proper singleton pattern to prevent race conditions.
 * Uses Promise-based locking to ensure only one initialization occurs.
 */
async function initializeAppKit(
  projectId: string,
  metadata: ReturnType<typeof createMetadata>,
  plainChain: ReturnType<typeof createPlainChain>,
) {
  if (initializationPromise) {
    // Another initialization is in progress, wait for it
    await initializationPromise;
    return;
  }

  if (wagmiAdapterInstance) {
    // Already initialized
    return;
  }

  // Create new initialization promise
  initializationPromise = (async () => {
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
    } catch (error) {
      console.warn("Failed to initialize AppKit:", error);
      wagmiAdapterInstance = null;
      throw error;
    } finally {
      initializationPromise = null;
    }
  })();

  await initializationPromise;
}

/**
 * Composable for accessing AppKit functionality.
 * Implements lazy initialization on first call to avoid SSR issues.
 *
 * Note: wagmiConfig and wagmiAdapter may be null/undefined during SSR or before
 * the first client-side call. Consuming code should handle these cases.
 */
export const useAppKit = () => {
  const runtimeConfig = useRuntimeConfig();
  const { defaultChain } = useClientStore();

  const projectId = runtimeConfig.public.appKitProjectId;
  const origin = typeof window !== "undefined" ? window.location.origin : "https://auth.zksync.dev";

  const metadata = createMetadata(origin);
  const plainChain = createPlainChain(defaultChain);

  // Lazy initialization - only create AppKit when first used on client
  if (typeof window !== "undefined" && !wagmiAdapterInstance && !initializationPromise) {
    // Fire and forget - initialization happens asynchronously
    initializeAppKit(projectId, metadata, plainChain).catch((error) => {
      console.warn("AppKit initialization failed:", error);
    });
  }

  const wagmiConfig = wagmiAdapterInstance?.wagmiConfig;

  return {
    metadata,
    projectId,
    wagmiAdapter: wagmiAdapterInstance,
    wagmiConfig,
  };
};

// HMR cleanup - reset state on hot module replacement
if (import.meta.hot) {
  import.meta.hot.dispose(() => {
    initializationPromise = null;
    wagmiAdapterInstance = null;
    cachedMetadata = null;
    cachedPlainChain = null;
  });
}
