import type { Address, Chain } from "viem";
import { createPublicClient as viemCreatePublicClient, http } from "viem";

/**
 * Contracts configuration loaded from /contracts.json
 */
export interface ContractsConfig {
  rpcUrl: string;
  chainId: number;
  deployer: Address;
  eoaValidator: Address;
  sessionValidator: Address;
  webauthnValidator: Address;
  guardianExecutor: Address;
  accountImplementation: Address;
  beacon: Address;
  factory: Address;
  bundlerUrl?: string;
  testPaymaster?: Address;
  entryPoint?: Address;
}

let cachedContracts: ContractsConfig | null = null;

/**
 * Loads contracts configuration from /contracts.json with caching
 * @returns Promise resolving to contracts configuration
 * @throws Error if contracts.json cannot be loaded or parsed
 */
export async function loadContracts(): Promise<ContractsConfig> {
  if (cachedContracts) {
    return cachedContracts;
  }

  try {
    const response = await fetch("/contracts.json");
    if (!response.ok) {
      throw new Error(`Failed to load contracts.json: ${response.statusText}`);
    }
    const contracts = (await response.json()) as ContractsConfig;
    cachedContracts = contracts;
    return contracts;
  } catch (error) {
    // eslint-disable-next-line no-console
    console.error("Error loading contracts.json:", error);
    throw new Error(
      "Failed to load contracts configuration. Make sure contracts.json exists in the public directory.",
    );
  }
}

/**
 * Clears the cached contracts configuration (useful for testing or hot-reload scenarios)
 */
export function clearContractsCache(): void {
  cachedContracts = null;
}

/**
 * Gets the bundler URL from contracts config, with fallback to default
 * @param contracts Contracts configuration
 * @returns Bundler URL
 */
export function getBundlerUrl(contracts: ContractsConfig): string {
  return contracts.bundlerUrl || "http://localhost:4337";
}

/**
 * Creates a viem Chain configuration from contracts config
 * @param contracts Contracts configuration
 * @returns Chain configuration for viem
 */
export function getChainConfig(contracts: ContractsConfig): Chain {
  return {
    id: contracts.chainId,
    name: "Anvil",
    nativeCurrency: { name: "Ether", symbol: "ETH", decimals: 18 },
    rpcUrls: { default: { http: [contracts.rpcUrl] } },
  } satisfies Chain;
}

/**
 * Creates a viem public client from contracts configuration
 * @param contracts Optional contracts configuration. If not provided, will load from contracts.json
 * @returns Promise resolving to a configured public client
 */
export async function createPublicClient(contracts?: ContractsConfig) {
  const config = contracts || (await loadContracts());
  const chain = getChainConfig(config);

  return viemCreatePublicClient({
    chain,
    transport: http(),
  });
}

/**
 * Convenience function that loads contracts and returns chain configuration
 * @returns Promise resolving to Chain configuration
 */
export async function getChainConfigFromContracts(): Promise<Chain> {
  const contracts = await loadContracts();
  return getChainConfig(contracts);
}
