import type { Hex } from "viem";

/**
 * Whitelists a contract address in Prividium by creating a contract entry
 * with a template key. This allows the contract to inherit permissions
 * from the template.
 *
 * @param contractAddress The deployed contract address to whitelist
 * @param templateKey The template key to associate with the contract
 * @param adminToken The authenticated admin's JWT token
 * @param permissionsApiUrl The base URL for the Prividium permissions API
 */
export async function whitelistContract(
  contractAddress: Hex,
  templateKey: string,
  adminToken: string,
  permissionsApiUrl: string,
): Promise<void> {
  const response = await fetch(`${permissionsApiUrl}/api/contracts`, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${adminToken}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      contractAddress,
      templateKey,
      abi: "[]",
      name: null,
      description: null,
      discloseErc20Balance: false,
      discloseBytecode: false,
    }),
  });

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(`Failed to whitelist contract: ${response.status} ${errorText}`);
  }

  console.log(`Successfully whitelisted contract ${contractAddress} with template ${templateKey}`);
}
