// TODO: Guardian recovery functions are not yet implemented in the Rust SDK
// Once the Rust SDK supports guardian recovery encoding, uncomment and update these functions
// to use WASM encoders instead of TypeScript encoding

// Imports will be needed when functions are uncommented
// import { type Account, type Address, type Chain, type Client, encodeFunctionData, type Hash, type Hex, keccak256, type Prettify, toHex, type TransactionReceipt, type Transport } from "viem";
// import { waitForTransactionReceipt } from "viem/actions";
// import { getGeneralPaymasterInput, sendTransaction } from "viem/zksync";
// import { GuardianRecoveryValidatorAbi } from "../../../abi/GuardianRecoveryValidator.js";
// import { noThrow } from "../../../utils/helpers.js";

/*
export type ProposeGuardianArgs = {
  newGuardian: Address;
  contracts: {
    recovery: Address; // recovery module
  };
  origin?: string;
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
  let origin: string | undefined = args.origin;
  if (!origin) {
    try {
      origin = window.location.origin;
    } catch {
      throw new Error("Can't identify expectedOrigin, please provide it manually");
    }
  }

  const callData = encodeFunctionData({
    abi: GuardianRecoveryValidatorAbi,
    functionName: "proposeGuardian",
    args: [keccak256(toHex(origin)), args.newGuardian],
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
*/
/*
export type ConfirmGuardianArgs = {
  accountToGuard: Address;
  contracts: {
    recovery: Address; // recovery module
  };
  origin?: string;
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
  let origin: string | undefined = args.origin;
  if (!origin) {
    try {
      origin = window.location.origin;
    } catch {
      throw new Error("Can't identify expectedOrigin, please provide it manually");
    }
  }
  const callData = encodeFunctionData({
    abi: GuardianRecoveryValidatorAbi,
    functionName: "addGuardian",
    args: [keccak256(toHex(origin)), args.accountToGuard],
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
*/
/*
export type RemoveGuardianArgs = {
  guardian: Address;
  contracts: {
    recovery: Address; // recovery module
  };
  origin?: string;
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
  let origin: string | undefined = args.origin;
  if (!origin) {
    try {
      origin = window.location.origin;
    } catch {
      throw new Error("Can't identify expectedOrigin, please provide it manually");
    }
  }
  const callData = encodeFunctionData({
    abi: GuardianRecoveryValidatorAbi,
    functionName: "removeGuardian",
    args: [keccak256(toHex(origin)), args.guardian],
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
*/

// Export an empty object to avoid import errors
export {};
