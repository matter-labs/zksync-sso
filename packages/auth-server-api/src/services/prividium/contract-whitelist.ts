import type { Hex } from "viem";

/**
 * Whitelists a contract address in Prividium by creating a contract entry
 * with a template key. This allows the contract to inherit permissions
 * from the template.
 *
 * @param contractAddress The deployed contract address to whitelist
 * @param templateKey The template key to associate with the contract
 * @param authHeaders The authentication headers from SDK
 * @param apiUrl The base URL for the Prividium API
 */
export async function whitelistContract(
  contractAddress: Hex,
  templateKey: string,
  authHeaders: Record<string, string>,
  apiUrl: string,
): Promise<void> {
  const response = await fetch(`${apiUrl}/api/contracts`, {
    method: "POST",
    headers: {
      ...authHeaders,
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
