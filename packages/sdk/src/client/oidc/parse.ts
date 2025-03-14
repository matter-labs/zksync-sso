import type { Hex } from "viem";
import { ByteVector } from "zksync-sso-circuits";

export type OidcData = {
  oidcDigest: Hex;
  iss: Hex;
  aud: Hex;
};

export type ParsedOidcData = {
  oidcDigest: string;
  iss: string;
  aud: string;
};
export const parseOidcData = (oidcData: OidcData): ParsedOidcData => {
  const hexToAscii = (hex: Hex): string => ByteVector.fromHex(hex).toAsciiStr();

  return {
    oidcDigest: oidcData.oidcDigest.toString(), // Do not convert. oidcDigest is just a hash
    iss: hexToAscii(oidcData.iss),
    aud: hexToAscii(oidcData.aud),
  };
};
