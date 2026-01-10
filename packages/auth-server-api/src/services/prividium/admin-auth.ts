import { type Hex } from "viem";
import { privateKeyToAccount } from "viem/accounts";

import type { PrividiumConfig } from "../../config.js";
import type { AdminAuthResponse, CachedAdminToken, SiweMessageResponse } from "./types.js";

/**
 * AdminAuthService handles SIWE-based authentication for the admin user account.
 * It caches the token and automatically refreshes it before expiration.
 *
 * Uses user SIWE endpoints (not tenant):
 * - POST /api/siwe-messages/ for SIWE message
 * - POST /api/auth/login/crypto-native for authentication
 */
export class AdminAuthService {
  private cachedToken: CachedAdminToken | null = null;
  private refreshPromise: Promise<string> | null = null;

  // Refresh token 5 minutes before expiry
  private readonly EXPIRY_BUFFER_MS = 5 * 60 * 1000;

  constructor(private config: PrividiumConfig) {}

  /**
   * Gets a valid admin token, refreshing if necessary.
   * Handles concurrent requests by reusing the same promise.
   */
  async getValidToken(): Promise<string> {
    // Check if we have a valid cached token
    if (this.isTokenValid()) {
      return this.cachedToken!.token;
    }

    // Prevent concurrent refresh requests
    if (this.refreshPromise) {
      return this.refreshPromise;
    }

    // Refresh token
    this.refreshPromise = this.authenticate();
    try {
      const token = await this.refreshPromise;
      return token;
    } finally {
      this.refreshPromise = null;
    }
  }

  /**
   * Checks if the cached token is still valid (with buffer time).
   */
  private isTokenValid(): boolean {
    if (!this.cachedToken) {
      return false;
    }

    const now = new Date();
    const bufferTime = new Date(this.cachedToken.expiresAt.getTime() - this.EXPIRY_BUFFER_MS);

    return now < bufferTime;
  }

  /**
   * Authenticates with Prividium using SIWE (Sign-In With Ethereum).
   * Uses user endpoints (not tenant) since admin is a user with admin role.
   *
   * 1. Requests a SIWE message from the permissions API (user endpoint)
   * 2. Signs the message with the admin's private key
   * 3. Submits the signature to authenticate via crypto-native login
   * 4. Caches and returns the token
   */
  private async authenticate(): Promise<string> {
    console.log("Authenticating admin user with Prividium...");

    // Derive address from private key
    const account = privateKeyToAccount(this.config.adminPrivateKey as Hex);
    const adminAddress = account.address;

    // Step 1: Request SIWE message (user endpoint, not tenant)
    const siweResponse = await fetch(`${this.config.permissionsApiUrl}/api/siwe-messages/`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        address: adminAddress,
        domain: new URL(this.config.ssoAuthServerBaseUrl).host,
      }),
    });

    if (!siweResponse.ok) {
      const errorText = await siweResponse.text();
      throw new Error(`Failed to get SIWE message: ${siweResponse.status} ${errorText}`);
    }

    const siweData = (await siweResponse.json()) as SiweMessageResponse;

    // Step 2: Sign the message with admin private key
    const signature = await account.signMessage({
      message: siweData.msg,
    });

    // Step 3: Authenticate with signature (crypto-native login for users)
    const authResponse = await fetch(`${this.config.permissionsApiUrl}/api/auth/login/crypto-native`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        message: siweData.msg,
        signature,
      }),
    });

    if (!authResponse.ok) {
      const errorText = await authResponse.text();
      throw new Error(`Admin authentication failed: ${authResponse.status} ${errorText}`);
    }

    const authData = (await authResponse.json()) as AdminAuthResponse;

    // Step 4: Cache the token
    this.cachedToken = {
      token: authData.token,
      expiresAt: new Date(authData.expiresAt),
    };

    console.log("Admin authenticated successfully, token expires at:", authData.expiresAt);

    return authData.token;
  }
}

// Singleton instance
let adminAuthServiceInstance: AdminAuthService | null = null;

/**
 * Gets or creates the singleton AdminAuthService instance.
 */
export function getAdminAuthService(config: PrividiumConfig): AdminAuthService {
  if (!adminAuthServiceInstance) {
    adminAuthServiceInstance = new AdminAuthService(config);
  }
  return adminAuthServiceInstance;
}
