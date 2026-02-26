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

  constructor(private config: PrividiumConfig, chain: Chain) {
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
   * Ensures admin is authenticated.
   * Call this once at startup.
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

// Singleton instance
let adminAuthServiceInstance: AdminAuthService | null = null;

/**
 * Gets or creates the singleton AdminAuthService instance.
 */
export function getAdminAuthService(config: PrividiumConfig, chain: Chain): AdminAuthService {
  if (!adminAuthServiceInstance) {
    adminAuthServiceInstance = new AdminAuthService(config, chain);
  }
  return adminAuthServiceInstance;
}
