import type { PrividiumSiweChain } from "prividium/siwe";
import type { Hex } from "viem";

/**
 * Whitelists a contract address in Prividium via the SDK admin namespace,
 * creating a contract entry with a template key. This allows the contract
 * to inherit permissions from the template.
 *
 * @param contractAddress The deployed contract address to whitelist
 * @param templateKey The template key to associate with the contract
 * @param sdk The admin-authenticated Prividium SDK chain
 */
export async function whitelistContract(
  contractAddress: Hex,
  templateKey: string,
  sdk: PrividiumSiweChain,
): Promise<void> {
  await sdk.admin.contracts.create({
    contractAddress,
    templateKey,
    abi: "[]",
    name: null,
    description: null,
    discloseErc20TotalSupply: false,
    discloseBytecode: false,
    disclosureStartBlock: "0x0",
  });

  console.log(`Successfully whitelisted contract ${contractAddress} with template ${templateKey}`);
}
