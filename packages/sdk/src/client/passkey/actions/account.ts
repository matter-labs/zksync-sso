import { WebAuthValidatorAbi } from "src/abi/WebAuthValidator.js";
import { encodeModuleData, encodePasskeyModuleParameters } from "src/utils/encoding.js";
import { base64UrlToUint8Array, getPasskeySignatureFromPublicKeyBytes, getPublicKeyBytesFromPasskeySignature } from "src/utils/passkey.js";
import { type Account, type Address, type Chain, type Client, type Prettify, toHex, type Transport } from "viem";
import { readContract } from "viem/actions";

export type DeployPasskeyAccountArgs = {
  /** Unique id of the passkey public key (base64) */
  credentialId: string;
  /** Public key of the previously registered */
  credentialPublicKey: Uint8Array;
  /** Expected origin of the passkey */
  expectedOrigin: string;
  /** Unique account ID, can be omitted if you don't need it */
  uniqueAccountId?: string;
};

export type FetchPasskeyAccountArgs = {
  /** Passkey unique idaddresses */
  credentialId: string;
  /** The domain the passkey is bound */
  expectedOrigin: string;
};

export type FetchAccountReturnType = {
  username: string;
  passkeyPublicKey: Uint8Array;
};

const NULL_ADDRESS = "0x0000000000000000000000000000000000000000";

export const getPasskeyUsername = async () => {
  try {
    const credential = await navigator.credentials.get({
      publicKey: {
        challenge: new Uint8Array(32),
        userVerification: "discouraged",
      },
    }) as PublicKeyCredential | null;

    if (!credential) throw new Error("No registered passkeys");
    return credential.id;
  } catch {
    throw new Error("Unable to retrieve passkey");
  }
};

export const getPasskeyData = (credentialPublicKey: Uint8Array, credentialId: string, contracts: { passkey: Address }) => {
  const passkeyPublicKey = getPublicKeyBytesFromPasskeySignature(credentialPublicKey);
  const encodedPasskeyParameters = encodePasskeyModuleParameters({
    credentialId: credentialId,
    passkeyPublicKey,
    expectedOrigin: origin,
  });
  return encodeModuleData({
    address: contracts.passkey,
    parameters: encodedPasskeyParameters,
  });
};

export const fetchAccount = async <
  transport extends Transport,
  chain extends Chain,
  account extends Account,
>(
  client: Client<transport, chain, account>, // Account deployer (any viem client)
  args: Prettify<FetchAccountArgs>,
): Promise<FetchAccountReturnType> => {
  let origin = args.expectedOrigin;
  if (!origin) {
    try {
      origin = window.location.origin;
    } catch {
      throw new Error("Can't identify expectedOrigin, please provide it manually");
    }
  }

  if (!args.contracts.accountFactory) throw new Error("Account factory address is not set");
  if (!args.contracts.passkey) throw new Error("Passkey module address is not set");

  let username = args.uniqueAccountId;
  if (!username) {
    try {
      const credential = await navigator.credentials.get({
        publicKey: {
          challenge: new Uint8Array(32),
          userVerification: "discouraged",
        },
      }) as PublicKeyCredential | null;

      if (!credential) throw new Error("No registered passkeys");
      username = credential.id;
    } catch {
      throw new Error("Unable to retrieve passkey");
    }
  }

  if (!username) throw new Error("No account found");

  const credentialId = toHex(base64UrlToUint8Array(username));
  const accountAddress = await readContract(client, {
    abi: WebAuthValidatorAbi,
    address: args.contracts.passkey,
    functionName: "registeredAddress",
    args: [origin, credentialId],
  });

  if (!accountAddress || accountAddress == NULL_ADDRESS) throw new Error(`No account found for username: ${username}`);

  const publicKey = await readContract(client, {
    abi: WebAuthValidatorAbi,
    address: args.contracts.passkey,
    functionName: "getAccountKey",
    args: [origin, credentialId, accountAddress],
  });

  if (!publicKey || !publicKey[0] || !publicKey[1]) throw new Error(`Passkey credentials not found in on-chain module for passkey ${username}`);

  const passkeyPublicKey = getPasskeySignatureFromPublicKeyBytes(publicKey);

  return {
    username,
    passkeyPublicKey,
    address: accountAddress,
  };
};
