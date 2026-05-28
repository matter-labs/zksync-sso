import type { PrividiumSiweChain } from "prividium/siwe";
import type { Hex } from "viem";

/**
 * Adds wallet addresses to a user in Prividium via the SDK admin namespace.
 *
 * Flow:
 * 1. GET the current user data to fetch existing wallet addresses
 * 2. PUT the updated wallets array (existing + new addresses, de-duplicated)
 *
 * @param userId The Prividium user ID to add addresses to
 * @param addresses Array of wallet addresses to associate
 * @param sdk The admin-authenticated Prividium SDK chain
 */
export async function addAddressToUser(
  userId: string,
  addresses: Hex[],
  sdk: PrividiumSiweChain,
): Promise<void> {
  const user = await sdk.admin.users.getById(userId);
  const existingWallets = user.wallets.map((w) => w.walletAddress);
  const allWallets = [...new Set([...existingWallets, ...addresses])];

  await sdk.admin.users.update(userId, { wallets: allWallets });

  console.log(`Successfully associated ${addresses.length} address(es) with user ${userId}`);
}
