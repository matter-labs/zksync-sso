import { createPrividiumSiweChain, type PrividiumSiweChain } from "prividium/siwe";
import { type Chain, type Hex } from "viem";
import { privateKeyToAccount } from "viem/accounts";

import type { PrividiumConfig } from "../../config.js";

/**
 * Admin authentication using Prividium SDK.
 * The SDK handles all authentication, token management, and auto-reauthentication.
 */
export class AdminAuthService {
  private sdkInstance: PrividiumSiweChain;
  readonly chainId: number;

  constructor(config: PrividiumConfig, chain: Chain) {
    this.chainId = chain.id;
    const account = privateKeyToAccount(config.adminPrivateKey as Hex);

    // Initialize SDK - it handles everything
    this.sdkInstance = createPrividiumSiweChain({
      account,
      chain: {
        id: chain.id,
        name: chain.name,
        nativeCurrency: chain.nativeCurrency,
      },
      prividiumApiBaseUrl: config.apiUrl,
      domain: config.domain,
      autoReauthenticate: true,
      onReauthenticate: () => {
        console.log("Admin session automatically refreshed");
      },
      onReauthenticateError: (error: Error) => {
        console.error("Admin reauthentication failed:", error);
      },
    });
  }

  /**
   * Authenticates admin with Prividium. Call once at startup.
   */
  async initialize(): Promise<void> {
    if (!this.sdkInstance.isAuthorized()) {
      console.log("Authenticating admin with Prividium...");
      await this.sdkInstance.authorize();
      console.log("Admin authenticated successfully");
    }
  }

  /**
   * Gets the SDK instance.
   * Use sdkInstance.transport for RPC calls.
   * Use sdkInstance.getAuthHeaders() for custom API calls.
   */
  getSdkInstance(): PrividiumSiweChain {
    return this.sdkInstance;
  }
}

// Singleton instance, initialized at startup via initAdminAuthService()
let adminAuthServiceInstance: AdminAuthService | null = null;

/**
 * Initializes the singleton AdminAuthService and authenticates with Prividium.
 * Must be called once at startup before handling requests.
 */
export async function initAdminAuthService(config: PrividiumConfig, chain: Chain): Promise<void> {
  adminAuthServiceInstance = new AdminAuthService(config, chain);
  await adminAuthServiceInstance.initialize();
}

/**
 * Gets the initialized AdminAuthService singleton.
 * Throws if called before initAdminAuthService().
 */
export function getAdminAuthService(): AdminAuthService {
  if (!adminAuthServiceInstance) {
    throw new Error("AdminAuthService not initialized. Call initAdminAuthService() at startup first.");
  }
  return adminAuthServiceInstance;
}
