import { startAuthentication, startRegistration } from "@simplewebauthn/browser";
import type { AuthenticationResponseJSON, GenerateAuthenticationOptionsOpts, GenerateRegistrationOptionsOpts, PublicKeyCredentialCreationOptionsJSON, PublicKeyCredentialRequestOptionsJSON, RegistrationResponseJSON, VerifiedRegistrationResponse } from "@simplewebauthn/server";
import { generateAuthenticationOptions, generateRegistrationOptions, verifyAuthenticationResponse, verifyRegistrationResponse } from "@simplewebauthn/server";
import type { Account, Address, Chain, Client, Hash, Hex, TransactionReceipt, Transport } from "viem";
import { encodeFunctionData, toBytes, toHex } from "viem";
import { waitForTransactionReceipt } from "viem/actions";
import { getGeneralPaymasterInput, sendTransaction } from "viem/zksync";

import { WebAuthValidatorAbi } from "../../../abi/WebAuthValidator.js";
import { noThrow } from "../../../utils/helpers.js";
import { base64UrlToUint8Array } from "../../../utils/passkey.js";

const identifyPasskeyParams = () => {
  let rpName: string | undefined;
  let rpID: string | undefined;
  let origin: string | undefined;
  try {
    rpName = window.location.hostname;
    rpID = window.location.hostname;
    origin = window.location.origin;
  } catch {
    // ignore
  }

  return { rpName, rpID, origin };
};

export type GeneratePasskeyRegistrationOptionsArgs = Partial<GenerateRegistrationOptionsOpts> & { userName: string; userDisplayName: string };
export type GeneratePasskeyRegistrationOptionsReturnType = PublicKeyCredentialCreationOptionsJSON;
export const generatePasskeyRegistrationOptions = async (args: GeneratePasskeyRegistrationOptionsArgs): Promise<GeneratePasskeyRegistrationOptionsReturnType> => {
  let { rpName, rpID } = identifyPasskeyParams();
  rpName = args.rpName || rpName;
  rpID = args.rpID || rpID;
  if (!rpName || !rpID) throw new Error("Can't set rpName and rpID automatically, please provide them manually in the arguments");

  const defaultOptions: GenerateRegistrationOptionsOpts = {
    rpName,
    rpID,
    userName: args.userName,
    userDisplayName: args.userDisplayName,
    // We want a stable id for the passkey
    attestationType: "direct",
    // Not preventing users from re-registering existing authenticators
    excludeCredentials: [],
    // See "Guiding use of authenticators via authenticatorSelection" below
    authenticatorSelection: {
      residentKey: "required",
      userVerification: "discouraged",
    },
    supportedAlgorithmIDs: [-7], // only supports ES256 (no windows hello)
  };
  const params: GenerateRegistrationOptionsOpts = Object.assign({}, defaultOptions, args);
  const options = await generateRegistrationOptions(params);
  options.pubKeyCredParams = options.pubKeyCredParams.filter(
    (creds) => creds.alg == 1,
  );

  return options;
};

export type GeneratePasskeyAuthenticationOptionsArgs = Partial<GenerateAuthenticationOptionsOpts>;
export type GeneratePasskeyAuthenticationOptionsReturnType = PublicKeyCredentialRequestOptionsJSON;
export const generatePasskeyAuthenticationOptions = async (args: GeneratePasskeyAuthenticationOptionsArgs): Promise<GeneratePasskeyAuthenticationOptionsReturnType> => {
  let { rpID } = identifyPasskeyParams();
  rpID = args.rpID || rpID;
  if (!rpID) throw new Error("Can't set rpID automatically, please provide them manually in the arguments");

  const defaultOptions: GenerateAuthenticationOptionsOpts = {
    rpID: rpID,
  };
  const params: GenerateAuthenticationOptionsOpts = Object.assign({}, defaultOptions, args);
  const options = await generateAuthenticationOptions(params);
  if ("pubKeyCredParams" in options) {
    options.pubKeyCredParams = (
      options.pubKeyCredParams as Array<{ alg: number; type: string }>
    ).filter((creds) => creds.alg == -7);
  }

  return options;
};

export type RegisterNewPasskeyArgs = ({ passkeyRegistrationOptions: PublicKeyCredentialCreationOptionsJSON } | GeneratePasskeyRegistrationOptionsArgs) & { origin?: string };
export type RegisterNewPasskeyReturnType = {
  passkeyRegistrationOptions: PublicKeyCredentialCreationOptionsJSON;
  passkeyRegistrationResponse: RegistrationResponseJSON;
  verificationResponse: VerifiedRegistrationResponse;
  credentialPublicKey: Uint8Array;
  credentialId: string;
};
export const registerNewPasskey = async (args: RegisterNewPasskeyArgs): Promise<RegisterNewPasskeyReturnType> => {
  let { origin } = identifyPasskeyParams();
  origin = args.origin || origin;
  if (!origin) throw new Error("Can't set origin automatically, please provide it manually in the arguments");

  const passkeyRegistrationOptions = "passkeyRegistrationOptions" in args ? args.passkeyRegistrationOptions : await generatePasskeyRegistrationOptions(args);
  const registrationResponse: RegistrationResponseJSON = await startRegistration({
    optionsJSON: passkeyRegistrationOptions,
  });
  const verification = await verifyRegistrationResponse({
    response: registrationResponse,
    expectedChallenge: passkeyRegistrationOptions.challenge,
    expectedOrigin: origin,
  });
  if (!verification.verified || !verification.registrationInfo) throw new Error("Passkey validation failed");

  return {
    passkeyRegistrationOptions,
    passkeyRegistrationResponse: registrationResponse,
    verificationResponse: verification,
    credentialPublicKey: verification.registrationInfo.credential.publicKey,
    credentialId: verification.registrationInfo.credential.id,
  };
};

export type RequestPasskeyAuthenticationArgs = {
  challenge: Hash; // Transaction hash to sign
  credentialPublicKey: Uint8Array;
  rpID?: string;
  origin?: string;
};
export type RequestPasskeyAuthenticationReturnType = {
  passkeyAuthenticationResponse: AuthenticationResponseJSON;
  passkeyAuthenticationOptions: PublicKeyCredentialRequestOptionsJSON;
};
export const requestPasskeyAuthentication = async (args: RequestPasskeyAuthenticationArgs): Promise<RequestPasskeyAuthenticationReturnType> => {
  const passkeyAuthenticationOptions = await generatePasskeyAuthenticationOptions({
    challenge: toBytes(args.challenge),
  });
  const optionsJSON: PublicKeyCredentialRequestOptionsJSON = { ...passkeyAuthenticationOptions };
  const authenticationResponse: AuthenticationResponseJSON = await startAuthentication({ optionsJSON: optionsJSON });

  let { rpID, origin } = identifyPasskeyParams();
  rpID = args.rpID || passkeyAuthenticationOptions.rpId || rpID;
  origin = args.origin || origin;
  if (!rpID || !origin) throw new Error("Can't set rpID and origin automatically, please provide them manually in the arguments");

  const verification = await verifyAuthenticationResponse({
    response: authenticationResponse,
    expectedChallenge: passkeyAuthenticationOptions.challenge,
    expectedOrigin: origin,
    expectedRPID: rpID,
    credential: {
      id: authenticationResponse.id,
      publicKey: args.credentialPublicKey,
      counter: 0, // TODO: figure out if this has to be dynamic

    },
  });
  if (!verification.verified || !verification.authenticationInfo) throw new Error("Passkey validation failed");

  return {
    passkeyAuthenticationResponse: authenticationResponse,
    passkeyAuthenticationOptions,
  };
};

export type AddAccountOwnerPasskeyArgs = {
  credentialId: string;
  rawPublicKey: readonly [Hex, Hex];
  origin: string;
  contracts: { passkey: Address };
  paymaster?: {
    address: Address;
    paymasterInput?: Hex;
  };
  onTransactionSent?: (hash: Hash) => void;
};
export type AddAccountOwnerPasskeyReturnType = {
  transactionReceipt: TransactionReceipt;
};
export const addAccountOwnerPasskey = async <
  transport extends Transport,
  chain extends Chain,
  account extends Account,
>(client: Client<transport, chain, account>, args: AddAccountOwnerPasskeyArgs): Promise<AddAccountOwnerPasskeyReturnType> => {
  const callData = encodeFunctionData({
    abi: WebAuthValidatorAbi,
    functionName: "addValidationKey",
    args: [toHex(base64UrlToUint8Array(args.credentialId)), args.rawPublicKey, args.origin],
  });

  const sendTransactionArgs = {
    account: client.account,
    to: args.contracts.passkey,
    paymaster: args.paymaster?.address,
    paymasterInput: args.paymaster?.address ? (args.paymaster?.paymasterInput || getGeneralPaymasterInput({ innerInput: "0x" })) : undefined,
    data: callData,
    gas: 10_000_000n, // TODO: Remove when gas estimation is fixed
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
  } as any;

  const transactionHash = await sendTransaction(client, sendTransactionArgs);
  if (args.onTransactionSent) {
    noThrow(() => args.onTransactionSent?.(transactionHash));
  }

  const transactionReceipt = await waitForTransactionReceipt(client, { hash: transactionHash });
  if (transactionReceipt.status !== "success") throw new Error("addValidationKey transaction reverted");

  return {
    transactionReceipt,
  };
};
