import type { Hex } from "viem";

import type { FullUserResponse } from "./types.js";

/**
 * Adds wallet addresses to a user in Prividium via the admin API.
 * This uses the admin's token to update any user's wallet addresses.
 *
 * Flow:
 * 1. GET /api/users/:id to fetch current user data including existing wallets
 * 2. PUT /api/users/:id with updated wallets array (existing + new addresses)
 *
 * @param userId The Prividium user ID to add addresses to
 * @param addresses Array of wallet addresses to associate
 * @param adminToken The authenticated admin's JWT token
 * @param permissionsApiUrl The base URL for the Prividium permissions API
 */
export async function addAddressToUser(
  userId: string,
  addresses: Hex[],
  adminToken: string,
  permissionsApiUrl: string,
): Promise<void> {
  // Step 1: Get current user data
  const getUserResponse = await fetch(`${permissionsApiUrl}/api/users/${userId}`, {
    method: "GET",
    headers: {
      Authorization: `Bearer ${adminToken}`,
    },
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
  const updateResponse = await fetch(`${permissionsApiUrl}/api/users/${userId}`, {
    method: "PUT",
    headers: {
      Authorization: `Bearer ${adminToken}`,
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
