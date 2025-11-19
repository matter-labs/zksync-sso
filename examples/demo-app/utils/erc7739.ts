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

export async function signTypedDataErc7739({
  typedData,
  smartAccountAddress,
  signer,
  chainId = 1337,
  validatorAddress,
}: SignTypedDataErc7739Args): Promise<Hex> {
  const verifierDomain: TypedDataDomain = {
    chainId,
    name: "zksync-sso-1271",
    version: "1.0.0",
    verifyingContract: smartAccountAddress,
    salt: pad("0x", { size: 32 }),
  };

  const erc7739Data: Erc7739TypedData = {
    domain: verifierDomain,
    types: typedData.types,
    primaryType: typedData.primaryType,
    message: typedData.message,
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

export function hashTypedDataErc7739({
  typedData,
  smartAccountAddress,
  chainId = 1337,
}: HashTypedDataErc7739Args): Hex {
  const verifierDomain: TypedDataDomain = {
    chainId,
    name: "zksync-sso-1271",
    version: "1.0.0",
    verifyingContract: smartAccountAddress,
    salt: pad("0x", { size: 32 }),
  };

  const erc7739Data: Erc7739TypedData = {
    domain: verifierDomain,
    types: typedData.types,
    primaryType: typedData.primaryType,
    message: typedData.message,
  };
  return hashTypedData(erc7739Data);
}
