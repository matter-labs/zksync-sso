import type { PrividiumSiweChain } from "prividium/siwe";
import { getAddress, type Hex } from "viem";

/**
 * Adds wallet addresses to a user in Prividium via the SDK admin namespace.
 *
 * Requires an admin-authenticated chain because it mutates another user's
 * wallet list. Existing wallets returned by the SDK may be checksum-cased
 * while incoming `addresses` may be lowercase (viem log topics, env input),
 * so both sides are normalized via viem's `getAddress` before the set-based
 * de-duplication — otherwise the unique constraint on the server would
 * reject the PUT for the same address in two casings.
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
  const existingWallets = user.wallets.map((w) => getAddress(w.walletAddress));
  const incomingWallets = addresses.map((a) => getAddress(a));
  const allWallets = [...new Set([...existingWallets, ...incomingWallets])];

  await sdk.admin.users.update(userId, { wallets: allWallets });

  console.log(`Successfully associated ${addresses.length} address(es) with user ${userId}`);
}
