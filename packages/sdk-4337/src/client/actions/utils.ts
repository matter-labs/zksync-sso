import type { Hex } from "viem";
import {
  generate_account_id,
} from "zksync-sso-web-sdk/bundler";

/**
 * Generate an account ID (used as salt for counterfactual account addresses).
 *
 * If userId is provided, generates a deterministic account ID from the userId.
 * If userId is not provided, generates a random account ID.
 *
 * Uses Rust WASM SDK for generation (no RPC calls).
 *
 * @param userId - Optional user ID for deterministic account generation
 * @returns 32-byte hex account ID
 *
 * @example
 * ```typescript
 * import { generateAccountId } from "zksync-sso/client-new/actions";
 *
 * // Generate deterministic account ID from user ID
 * const accountId = generateAccountId("user123");
 *
 * // Generate random account ID
 * const randomAccountId = generateAccountId();
 * ```
 */
export function generateAccountId(userId?: string): Hex {
  return generate_account_id(userId || null) as Hex;
}
