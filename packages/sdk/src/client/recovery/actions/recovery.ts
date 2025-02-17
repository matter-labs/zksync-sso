import { type Account, type Address, type Chain, type Client, encodeAbiParameters, encodeFunctionData, type Hash, type Hex, type Prettify, type TransactionReceipt, type Transport } from "viem";
import { waitForTransactionReceipt } from "viem/actions";
import { getGeneralPaymasterInput, sendTransaction } from "viem/zksync";

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

export type AddOidcAccountArgs = {
  contracts: {
    recoveryOidc: Address; // oidc recovery module
  };
  paymaster?: {
    address: Address;
    paymasterInput?: Hex;
  };
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
  const callData = encodeFunctionData({
    abi: OidcRecoveryModuleAbi,
    functionName: "addValidationKey",
    args: [
      encodeAbiParameters(
        [
          { type: "bytes", name: "oidcDigest" },
          { type: "bytes", name: "iss" },
          { type: "bytes", name: "aud" },
        ],
        [
          "0xdeadbeef",
          "0xdeadbeef",
          "0xdeadbeef",
        ],
      )
    ],
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