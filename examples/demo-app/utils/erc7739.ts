import { hashTypedData, wrapTypedDataSignature } from "viem/experimental/erc7739";
import { pad, concatHex } from "viem";
import type { TypedData, TypedDataDomain, Hex } from "viem";

export interface Erc7739TypedData {
  domain: TypedDataDomain;
  types: TypedData;
  primaryType: string;
  message: Record<string, unknown>;
}

export interface SignTypedDataErc7739Args {
  typedData: Erc7739TypedData;
  smartAccountAddress: Hex;
  signer: (hash: Hex) => Promise<Hex>;
  chainId?: number;
  validatorAddress?: Hex;
}

export interface HashTypedDataErc7739Args {
  typedData: Erc7739TypedData;
  smartAccountAddress: Hex;
  chainId?: number;
}

/**
 * Creates the verifier domain for ERC-7739 typed data wrapping.
 * This domain is used to wrap application-specific typed data with account verification metadata.
 *
 * @param smartAccountAddress - The smart account address that will verify the signature
 * @param chainId - The chain ID for the verifier domain (defaults to 1337 for local Anvil)
 * @returns The TypedDataDomain for the verifier (smart account) with all required fields
 */
function createVerifierDomain(smartAccountAddress: Hex, chainId = 1337) {
  return {
    chainId,
    name: "zksync-sso-1271" as const,
    version: "1.0.0" as const,
    verifyingContract: smartAccountAddress,
    salt: pad("0x", { size: 32 }),
  } as const;
}

/**
 * Signs EIP-712 typed data using ERC-7739 nested typed data wrapping.
 *
 * This function implements the ERC-7739 standard for smart account typed data signatures:
 * 1. Wraps the application's typed data in a verifier domain (smart account context)
 * 2. Computes the ERC-7739 hash (nested EIP-712 hash)
 * 3. Signs the hash with the provided signer
 * 4. Optionally prepends a validator address (required for ModularSmartAccount)
 * 5. Wraps the signature with ERC-7739 metadata (domain, contents, description)
 *
 * The resulting signature can be verified on-chain via ERC-1271's isValidSignature.
 *
 * @param typedData - The application's EIP-712 typed data to sign
 * @param smartAccountAddress - The smart account that will verify this signature
 * @param signer - Async function that signs a hash and returns a signature (e.g., EOA.sign)
 * @param chainId - Chain ID for the verifier domain (defaults to 1337)
 * @param validatorAddress - Optional validator module address to prepend before wrapping (required for modular accounts)
 * @returns The ERC-7739 wrapped signature with validator prefix and metadata
 */
export async function signTypedDataErc7739({
  typedData,
  smartAccountAddress,
  signer,
  chainId = 1337,
  validatorAddress,
}: SignTypedDataErc7739Args): Promise<Hex> {
  const verifierDomain = createVerifierDomain(smartAccountAddress, chainId);

  const erc7739Data = {
    ...typedData,
    verifierDomain,
  };
  const hash = hashTypedData(erc7739Data);
  const signature = await signer(hash);

  const signatureToWrap = validatorAddress
    ? concatHex([validatorAddress, signature])
    : signature;

  return wrapTypedDataSignature({
    domain: typedData.domain,
    types: typedData.types,
    primaryType: typedData.primaryType,
    message: typedData.message,
    signature: signatureToWrap,
  });
}

/**
 * Computes the ERC-7739 hash for typed data without signing.
 *
 * This function is useful for:
 * - Debugging signature verification issues by comparing JS and on-chain hashes
 * - Precomputing the digest that will be signed
 * - Testing hash computation without requiring a signer
 *
 * The hash is computed by wrapping the application's typed data in a verifier domain
 * (smart account context) and computing the nested EIP-712 hash per ERC-7739.
 *
 * @param typedData - The application's EIP-712 typed data
 * @param smartAccountAddress - The smart account address used in the verifier domain
 * @param chainId - Chain ID for the verifier domain (defaults to 1337)
 * @returns The ERC-7739 hash (nested EIP-712 hash) that would be signed
 */
export function hashTypedDataErc7739({
  typedData,
  smartAccountAddress,
  chainId = 1337,
}: HashTypedDataErc7739Args): Hex {
  const verifierDomain = createVerifierDomain(smartAccountAddress, chainId);

  const erc7739Data = {
    ...typedData,
    verifierDomain,
  };
  return hashTypedData(erc7739Data);
}
