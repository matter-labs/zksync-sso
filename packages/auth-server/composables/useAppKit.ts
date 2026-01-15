import { createAppKit } from "@reown/appkit/vue";
import { WagmiAdapter } from "@reown/appkit-adapter-wagmi";

// Singleton initialization state
let initializationPromise: Promise<void> | null = null;
let wagmiAdapterInstance: WagmiAdapter | null = null;

/**
 * Creates metadata configuration for AppKit.
 */
function createMetadata(origin: string) {
  return {
    name: "ZKsync SSO Auth Server",
    description: "ZKsync SSO Auth Server",
    url: origin,
    icons: [`${origin}/icon-512.png`],
  };
}

/**
 * Creates plain chain object to avoid Viem Proxy serialization issues.
 */
function createPlainChain(defaultChain: ReturnType<typeof useClientStore>["defaultChain"]) {
  return {
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
 * IMPORTANT: Due to lazy 'fire-and-forget' initialization:
 * - wagmiConfig and wagmiAdapter will be null/undefined during SSR
 * - They will also be null/undefined on the first client-side call
 * - They will only be available after async initialization completes
 * - All consuming components MUST handle null values gracefully
 * - Components should reactively watch these values or check for null before use
 */
export const useAppKit = () => {
  const runtimeConfig = useRuntimeConfig();
  const { defaultChain } = useClientStore();

  const projectId = runtimeConfig.public.appKitProjectId;
  const origin
    = typeof window !== "undefined"
      ? window.location.origin
      : runtimeConfig.public?.authServerOrigin
        ?? process.env.NUXT_PUBLIC_AUTH_SERVER_ORIGIN
        ?? "https://auth.zksync.dev";

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
  });
}
