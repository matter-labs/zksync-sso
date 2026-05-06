import type { Hex } from "viem";

import { authenticatedFetch, type PrividiumApiAuth } from "./authenticated-fetch.js";
import type { FullUserResponse } from "./types.js";

/**
 * Adds wallet addresses to a user in Prividium via the admin API.
 * This uses the admin's authentication headers to update any user's wallet addresses.
 *
 * Flow:
 * 1. GET /api/users/:id to fetch current user data including existing wallets
 * 2. PUT /api/users/:id with updated wallets array (existing + new addresses)
 *
 * @param userId The Prividium user ID to add addresses to
 * @param addresses Array of wallet addresses to associate
 * @param auth The admin authentication provider from SDK
 * @param apiUrl The base URL for the Prividium API
 */
export async function addAddressToUser(
  userId: string,
  addresses: Hex[],
  auth: PrividiumApiAuth,
  apiUrl: string,
): Promise<void> {
  // Step 1: Get current user data
  const getUserResponse = await authenticatedFetch(auth, `${apiUrl}/api/users/${userId}`, {
    method: "GET",
  });

  if (!getUserResponse.ok) {
    const errorText = await getUserResponse.text();
    throw new Error(`Failed to get user: ${getUserResponse.status} ${errorText}`);
  }

  const user = (await getUserResponse.json()) as FullUserResponse;
  const existingWallets = user.wallets.map((w) => w.walletAddress);

  // Merge existing wallets with new addresses (avoid duplicates)
  const allWallets = [...new Set([...existingWallets, ...addresses])];

  // Step 2: Update user with new wallets
  const updateResponse = await authenticatedFetch(auth, `${apiUrl}/api/users/${userId}`, {
    method: "PUT",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      wallets: allWallets,
    }),
  });

  if (!updateResponse.ok) {
    const errorText = await updateResponse.text();
    throw new Error(`Failed to associate address with user: ${updateResponse.status} ${errorText}`);
  }

  console.log(`Successfully associated ${addresses.length} address(es) with user ${userId}`);
}
