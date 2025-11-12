/* eslint-disable @typescript-eslint/no-explicit-any */
/**
 * Global type definitions for browser wallet providers
 */

/**
 * EIP-1193 Ethereum Provider
 * @see https://eips.ethereum.org/EIPS/eip-1193
 */
interface EthereumProvider {
  /**
   * Make an RPC request to the provider
   */
  request(args: { method: string; params?: Array<any> | Record<string, any> }): Promise<any>;

  /**
   * Subscribe to provider events
   */
  on(event: string, handler: (...args: any[]) => void): void;

  /**
   * Unsubscribe from provider events
   */
  removeListener(event: string, handler: (...args: any[]) => void): void;

  /**
   * Check if the provider is MetaMask
   */
  isMetaMask?: boolean;

  /**
   * Selected account address (may be undefined if not connected)
   */
  selectedAddress?: string | null;

  /**
   * Chain ID in hex format
   */
  chainId?: string;

  /**
   * Network version (deprecated, use chainId instead)
   */
  networkVersion?: string;
}

declare global {
  interface Window {
    /**
     * Ethereum provider injected by browser wallet extensions (MetaMask, etc.)
     */
    ethereum?: EthereumProvider;
  }
}

export {};
