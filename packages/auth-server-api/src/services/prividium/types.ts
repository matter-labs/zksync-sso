import type { Hex } from "viem";

// Prividium API response types

export interface AdminAuthResponse {
  token: string;
  expiresAt: string; // ISO date string
}

export interface SiweMessageResponse {
  msg: string;
  nonce: string;
}

export interface UserProfile {
  id: string;
  displayName?: string;
  wallets?: Array<{ walletAddress: Hex }>;
  roles?: Array<{ roleName: string }>;
}

export interface CachedAdminToken {
  token: string;
  expiresAt: Date;
}

export type UserAuthErrorType = "network_error" | "invalid_token" | "server_error";

export interface UserAuthResult {
  valid: boolean;
  userId?: string;
  error?: UserAuthErrorType;
}
