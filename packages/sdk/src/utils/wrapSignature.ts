import { wrapTypedDataSignature } from "viem/experimental/erc7739";

/**
 * Wraps a raw EIP-712 signature into ERC-7739 format for passkey signatures.
 * This is useful when you have a raw signature and need to convert it to
 * ERC-7739 format for smart account verification.
 *
 * @param signature - The raw EIP-712 signature to wrap
 * @returns The wrapped signature in ERC-7739 format
 */
export const wrapPasskeyTypedDataSig = wrapTypedDataSignature;
