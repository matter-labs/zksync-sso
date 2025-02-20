import { type Account, type Address, type Chain, type Client, encodeAbiParameters, encodeFunctionData, type Hash, type Hex, type Prettify, type TransactionReceipt, type Transport } from "viem";
import { waitForTransactionReceipt } from "viem/actions";
import { getGeneralPaymasterInput, sendTransaction } from "viem/zksync";
import { ByteVector } from "zksync-sso-circuits";

import { GuardianRecoveryModuleAbi } from "../../../abi/GuardianRecoveryModule.js";
import { OidcRecoveryModuleAbi } from "../../../abi/OidcRecoveryModule.js";
import { noThrow } from "../../../utils/helpers.js";
import { encodePasskeyModuleParameters } from "../../../utils/index.js";
import { getPublicKeyBytesFromPasskeySignature } from "../../../utils/passkey.js";

export type ProposeGuardianArgs = {
  newGuardian: Address;
  contracts: {
    recovery: Address; // recovery module
  };
  paymaster?: {
    address: Address;
    paymasterInput?: Hex;
  };
  onTransactionSent?: (hash: Hash) => void;
};
export type ProposeGuardianReturnType = {
  transactionReceipt: TransactionReceipt;
};
export const proposeGuardian = async <
  transport extends Transport,
  chain extends Chain,
  account extends Account,
>(client: Client<transport, chain, account>, args: Prettify<ProposeGuardianArgs>): Promise<Prettify<ProposeGuardianReturnType>> => {
  const callData = encodeFunctionData({
    abi: GuardianRecoveryModuleAbi,
    functionName: "proposeValidationKey",
    args: [args.newGuardian],
  });

  const sendTransactionArgs = {
    account: client.account,
    to: args.contracts.recovery,
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
  if (transactionReceipt.status !== "success") throw new Error("proposeGuardian transaction reverted");

  return {
    transactionReceipt,
  };
};
export type ConfirmGuardianArgs = {
  accountToGuard: Address;
  contracts: {
    recovery: Address; // recovery module
  };
  paymaster?: {
    address: Address;
    paymasterInput?: Hex;
  };
  onTransactionSent?: (hash: Hash) => void;
};
export type ConfirmGuardianReturnType = {
  transactionReceipt: TransactionReceipt;
};
export const confirmGuardian = async <
  transport extends Transport,
  chain extends Chain,
  account extends Account,
>(client: Client<transport, chain, account>, args: Prettify<ConfirmGuardianArgs>): Promise<Prettify<ConfirmGuardianReturnType>> => {
  const callData = encodeFunctionData({
    abi: GuardianRecoveryModuleAbi,
    functionName: "addValidationKey",
    args: [encodeAbiParameters(
      [{ type: "address" }],
      [args.accountToGuard],
    )],
  });

  const sendTransactionArgs = {
    account: client.account,
    to: args.contracts.recovery,
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
  if (transactionReceipt.status !== "success") throw new Error("confirmGuardian transaction reverted");

  return {
    transactionReceipt,
  };
};
export type RemoveGuardianArgs = {
  guardian: Address;
  contracts: {
    recovery: Address; // recovery module
  };
  paymaster?: {
    address: Address;
    paymasterInput?: Hex;
  };
  onTransactionSent?: (hash: Hash) => void;
};
export type RemoveGuardianReturnType = {
  transactionReceipt: TransactionReceipt;
};
export const removeGuardian = async <
  transport extends Transport,
  chain extends Chain,
  account extends Account,
>(client: Client<transport, chain, account>, args: Prettify<RemoveGuardianArgs>): Promise<Prettify<RemoveGuardianReturnType>> => {
  const callData = encodeFunctionData({
    abi: GuardianRecoveryModuleAbi,
    functionName: "removeValidationKey",
    args: [args.guardian],
  });

  const sendTransactionArgs = {
    account: client.account,
    to: args.contracts.recovery,
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
  if (transactionReceipt.status !== "success") throw new Error("removeGuardian transaction reverted");

  return {
    transactionReceipt,
  };
};

export type InitRecoveryArgs = {
  expectedOrigin: string | undefined;
  credentialPublicKey: Uint8Array;
  accountId: string;
  contracts: {
    recovery: Address; // recovery module
  };
  account: Address;
  paymaster?: {
    address: Address;
    paymasterInput?: Hex;
  };
  onTransactionSent?: (hash: Hash) => void;
};

export type InitRecoveryReturnType = {
  transactionReceipt: TransactionReceipt;
};
export const initRecovery = async <
  transport extends Transport,
  chain extends Chain,
  account extends Account,
>(client: Client<transport, chain, account>, args: Prettify<InitRecoveryArgs>): Promise<Prettify<InitRecoveryReturnType>> => {
  let origin: string | undefined = args.expectedOrigin;
  if (!origin) {
    try {
      origin = window.location.origin;
    } catch {
      throw new Error("Can't identify expectedOrigin, please provide it manually");
    }
  }

  const passkeyPublicKey = getPublicKeyBytesFromPasskeySignature(args.credentialPublicKey);
  const encodedPasskeyParameters = encodePasskeyModuleParameters({
    passkeyPublicKey,
    expectedOrigin: origin,
  });

  const callData = encodeFunctionData({
    abi: GuardianRecoveryModuleAbi,
    functionName: "initRecovery",
    args: [args.account, encodedPasskeyParameters, args.accountId],
  });

  const sendTransactionArgs = {
    account: client.account,
    to: args.contracts.recovery,
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
  if (transactionReceipt.status !== "success") throw new Error("initRecovery transaction reverted");

  return {
    transactionReceipt,
  };
};

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

export type AddOidcAccountArgs = {
  contracts: {
    recoveryOidc: Address; // oidc recovery module
  };
  paymaster?: {
    address: Address;
    paymasterInput?: Hex;
  };
  oidcData: OidcData;
  onTransactionSent?: (hash: Hash) => void;
};

export type AddOidcAccountReturnType = {
  transactionReceipt: TransactionReceipt;
};

export const addOidcAccount = async <
  transport extends Transport,
  chain extends Chain,
  account extends Account,
>(client: Client<transport, chain, account>, args: Prettify<AddOidcAccountArgs>): Promise<Prettify<AddOidcAccountReturnType>> => {
  const encodedOidcData = encodeAbiParameters(
    [
      {
        type: "tuple", name: "OidcData", components: [
          { type: "bytes", name: "oidcDigest" },
          { type: "bytes", name: "iss" },
          { type: "bytes", name: "aud" },
        ],
      },
    ],
    [args.oidcData],
  );

  const callData = encodeFunctionData({
    abi: OidcRecoveryModuleAbi,
    functionName: "addValidationKey",
    args: [encodedOidcData],
  });

  const sendTransactionArgs = {
    account: client.account,
    to: args.contracts.recoveryOidc,
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
  if (transactionReceipt.status !== "success") throw new Error("addOidcAccount transaction reverted");

  return {
    transactionReceipt,
  };
};
