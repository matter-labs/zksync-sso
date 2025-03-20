import type { Hex } from "viem";
import { ByteVector } from "zksync-sso-circuits";

export type OidcData = {
  oidcDigest: Hex;
  iss: string;
  readyToRecover: boolean;
  pendingPasskeyHash: Hex;
  recoverNonce: Hex;
};

export type ParsedOidcData = {
  oidcDigest: string;
  iss: string;
};

export const hexToAscii = (hex: Hex): string => ByteVector.fromHex(hex).toAsciiStr();

export const parseOidcData = (oidcData: OidcData): ParsedOidcData => {
  return {
    oidcDigest: oidcData.oidcDigest.toString(), // Do not convert. oidcDigest is just a hash
    iss: oidcData.iss,
  };
};
