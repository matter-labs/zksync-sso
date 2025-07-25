// Re-export everything from the connector folder in the SDK
export * from "zksync-sso/connector";

// Export the enhanced connector with ZKsync Era chains
import { zksync, zksyncInMemoryNode, zksyncSepoliaTestnet } from "viem/chains";
import { zksyncSsoConnector as originalConnector, type ZksyncSsoConnectorOptions } from "zksync-sso/connector";

// Re-export ZKsync chains from wagmi with our naming convention
export const eraTestNode = zksyncInMemoryNode; // ID 260 - ZKsync InMemory Node (era-test-node)
export const eraSepolia = zksyncSepoliaTestnet; // ID 300 - ZKsync Sepolia Testnet
export const eraMainnet = zksync; // ID 324 - ZKsync Era

// Type alias for the connector options (same as base for now, but allows future extension)
export type ZksyncSsoConnectorWithEraOptions = ZksyncSsoConnectorOptions;

/**
 * Enhanced ZKsync SSO connector that provides easy access to ZKsync Era chains.
 * Import the chains you need (eraTestNode, eraSepolia, eraMainnet) and use them in your wagmi config.
 * eraSepolia is recommended as the default for development.
 *
 * Example usage:
 * ```typescript
 * import { createConfig } from '@wagmi/core'
 * import { zksyncSsoConnector, eraSepolia, eraMainnet } from '@zksync-sso/connector-export'
 *
 * const config = createConfig({
 *   chains: [eraSepolia, eraMainnet],
 *   connectors: [zksyncSsoConnector()],
 * })
 * ```
 *
 * @param parameters - Connector configuration options
 * @returns Wagmi connector for ZKsync Era
 */
export function zksyncSsoConnector(parameters: ZksyncSsoConnectorWithEraOptions = {}) {
  // Return the original connector - chains are configured at the wagmi config level
  // Available chains: eraTestNode (260), eraSepolia (300), eraMainnet (324)
  return originalConnector(parameters);
}
