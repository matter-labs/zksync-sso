import { type Account, type Address, type Chain, type Client, encodeFunctionData, type Hash, type Hex, type Prettify, type TransactionReceipt, type Transport } from "viem";
import { waitForTransactionReceipt } from "viem/actions";
import { getGeneralPaymasterInput, sendTransaction } from "viem/zksync";

import { SessionKeyValidatorAbi } from "../../../abi/SessionKeyValidator.js";
import { type CustomPaymasterHandler, getTransactionWithPaymasterData } from "../../../paymaster/index.js";
import { noThrow } from "../../../utils/helpers.js";
import type { SessionConfig } from "../../../utils/session.js";

export type CreateSessionArgs = {
  sessionConfig: SessionConfig;
  contracts: {
    session: Address; // session module
  };
  paymaster?: {
    address: Address;
    paymasterInput?: Hex;
  };
  onTransactionSent?: (hash: Hash) => void;
  paymasterHandler?: CustomPaymasterHandler;
};
export type CreateSessionReturnType = {
  transactionReceipt: TransactionReceipt;
};
export const createSession = async <
  transport extends Transport,
  chain extends Chain,
  account extends Account,
>(client: Client<transport, chain, account>, args: Prettify<CreateSessionArgs>): Promise<Prettify<CreateSessionReturnType>> => {
  const callData = encodeFunctionData({
    abi: SessionKeyValidatorAbi,
    functionName: "createSession",
    args: [args.sessionConfig],
  });

  const sendTransactionArgs = {
    account: client.account,
    to: args.contracts.session,
    paymaster: args.paymaster?.address,
    paymasterInput: args.paymaster?.address ? (args.paymaster?.paymasterInput || getGeneralPaymasterInput({ innerInput: "0x" })) : undefined,
    data: callData,
    gas: 10_000_000n, // TODO: Remove when gas estimation is fixed
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
  } as any;

  const transactionWithPaymasterData: any = await getTransactionWithPaymasterData(
    client.chain.id,
    client.account.address,
    sendTransactionArgs,
    args.paymasterHandler,
  );

  const transactionHash = await sendTransaction(client, transactionWithPaymasterData);
  if (args.onTransactionSent) {
    noThrow(() => args.onTransactionSent?.(transactionHash));
  }

  const transactionReceipt = await waitForTransactionReceipt(client, { hash: transactionHash });
  if (transactionReceipt.status !== "success") throw new Error("createSession transaction reverted");

  return {
    transactionReceipt,
  };
};

export type RevokeSessionArgs = {
  sessionId: Hash;
  contracts: {
    session: Address; // session module
  };
  paymaster?: {
    address: Address;
    paymasterInput?: Hex;
  };
  onTransactionSent?: (hash: Hash) => void;
  paymasterHandler?: CustomPaymasterHandler;
};
export type RevokeSessionReturnType = {
  transactionReceipt: TransactionReceipt;
};
export const revokeSession = async <
  transport extends Transport,
  chain extends Chain,
  account extends Account,
>(client: Client<transport, chain, account>, args: Prettify<RevokeSessionArgs>): Promise<Prettify<RevokeSessionReturnType>> => {
  const callData = encodeFunctionData({
    abi: SessionKeyValidatorAbi,
    functionName: "revokeKey",
    args: [args.sessionId],
  });

  const sendTransactionArgs = {
    account: client.account,
    to: args.contracts.session,
    paymaster: args.paymaster?.address,
    paymasterInput: args.paymaster?.address ? (args.paymaster?.paymasterInput || getGeneralPaymasterInput({ innerInput: "0x" })) : undefined,
    data: callData,
    gas: 10_000_000n, // TODO: Remove when gas estimation is fixed
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
  } as any;

  const transactionWithPaymasterData: any = await getTransactionWithPaymasterData(
    client.chain.id,
    client.account.address,
    sendTransactionArgs,
    args.paymasterHandler,
  );

  const transactionHash = await sendTransaction(client, transactionWithPaymasterData);

  if (args.onTransactionSent) {
    noThrow(() => args.onTransactionSent?.(transactionHash));
  }

  const transactionReceipt = await waitForTransactionReceipt(client, { hash: transactionHash });
  if (transactionReceipt.status !== "success") throw new Error("createSession transaction reverted");

  return {
    transactionReceipt,
  };
};
