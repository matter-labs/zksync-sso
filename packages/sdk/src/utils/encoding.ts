import { type Address, encodeAbiParameters, getAbiItem, type Hash, type Hex, pad, parseAbiParameters, toHex } from "viem";

import { SessionKeyValidatorAbi } from "../abi/SessionKeyValidator.js";
import { base64UrlToUint8Array } from "../utils/passkey.js";
import { getPeriodIdsForTransaction, type SessionConfig } from "../utils/session.js";

const getSessionSpec = () => {
  return getAbiItem({
    abi: SessionKeyValidatorAbi,
    name: "createSession",
  }).inputs[0];
};

const extractSelectorFromCallData = (callData: Hash) => {
  const selector = callData.slice(0, "0x".length + 8); // first 4 bytes for function selector
  if (selector.length !== 10) return undefined;
  return selector as Hex;
};

export const encodeSession = (sessionConfig: SessionConfig) => {
  return encodeAbiParameters(
    [getSessionSpec()],
    [sessionConfig],
  );
};
export const encodeSessionTx = (args: {
  sessionConfig: SessionConfig;
  to: Address;
  callData?: Hash;
  timestamp?: bigint;
}) => {
  return encodeAbiParameters(
    [
      getSessionSpec(),
      { type: "uint64[]" },
    ],
    [
      args.sessionConfig,
      getPeriodIdsForTransaction({
        sessionConfig: args.sessionConfig,
        target: args.to,
        selector: args.callData ? extractSelectorFromCallData(args.callData) : undefined,
        timestamp: args.timestamp,
      }),
    ],
  );
};

export const encodePasskeyModuleParameters = (passkey: { credentialId: string; passkeyPublicKey: [Buffer, Buffer]; expectedOrigin: string }) => {
  return encodeAbiParameters(
    [
      { type: "bytes", name: "credentialId" },
      { type: "bytes32[2]", name: "xyPublicKeys" },
      { type: "string", name: "expectedOrigin" },
    ],
    [
      toHex(base64UrlToUint8Array(passkey.credentialId)),
      [pad(toHex(passkey.passkeyPublicKey[0])), pad(toHex(passkey.passkeyPublicKey[1]))],
      passkey.expectedOrigin,
    ],
  );
};

export const encodeModuleData = (moduleData: { address: Address; parameters: Hash }) => {
  const moduleParams = parseAbiParameters("address, bytes");
  return encodeAbiParameters(
    moduleParams,
    [moduleData.address, moduleData.parameters],
  );
};
