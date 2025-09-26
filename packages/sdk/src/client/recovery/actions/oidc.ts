import {
  type Account,
  type Address,
  type Chain,
  type Client,
  encodeFunctionData,
  type Hash,
  type Hex,
  type Prettify,
  type TransactionReceipt,
  type Transport,
} from "viem";
import { waitForTransactionReceipt } from "viem/actions";
import {
  getGeneralPaymasterInput,
  sendTransaction,
} from "viem/zksync";

import { OidcRecoveryValidatorAbi } from "../../../abi/index.js";
import { noThrow } from "../../../utils/helpers.js";

export type AddOidcAccountArgs = {
  contracts: {
    recoveryOidc: Address; // oidc recovery module
  };
  paymaster?: {
    address: Address;
    paymasterInput?: Hex;
  };
  oidcDigest: Hex;
  iss: string;
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
    abi: OidcRecoveryValidatorAbi,
    functionName: "addOidcAccount",
    args: [args.oidcDigest, args.iss],
  });
  // Base transaction args (used for fallback as well)
  const baseTxArgs = {
    account: client.account,
    to: args.contracts.recoveryOidc,
    data: callData,
    gas: 10_000_000n, // TODO: Remove when gas estimation is fixed
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
  } as any;

  let transactionHash: Hash;
  if (args.paymaster?.address) {
    const primaryTxArgs = {
      ...baseTxArgs,
      paymaster: args.paymaster.address,
      paymasterInput: args.paymaster?.paymasterInput || getGeneralPaymasterInput({ innerInput: "0x" }),
    };
    try {
      transactionHash = await sendTransaction(client, primaryTxArgs);
    } catch (error) {
      console.error("Paymaster transaction failed, falling back to base transaction.", error);
      transactionHash = await sendTransaction(client, baseTxArgs);
    }
  } else {
    transactionHash = await sendTransaction(client, baseTxArgs);
  }

  if (args.onTransactionSent) {
    noThrow(() => args.onTransactionSent?.(transactionHash));
  }

  const transactionReceipt = await waitForTransactionReceipt(client, { hash: transactionHash });
  if (transactionReceipt.status !== "success") throw new Error(`addOidcAccount transaction reverted: ${transactionReceipt}`);

  // (Optional) Could expose whether fallback was used in the future without breaking change by adding to return type.

  return {
    transactionReceipt,
  };
};

export type RemoveOidcAccountArgs = {
  contracts: {
    recoveryOidc: Address; // oidc recovery module
  };
};

export const removeOidcAccount = async <
  transport extends Transport,
  chain extends Chain,
  account extends Account,
>(client: Client<transport, chain, account>, args: Prettify<RemoveOidcAccountArgs>): Promise<TransactionReceipt> => {
  const callData = encodeFunctionData({
    abi: OidcRecoveryValidatorAbi,
    functionName: "deleteOidcAccount",
    args: [],
  });

  const sendTransactionArgs = {
    account: client.account,
    to: args.contracts.recoveryOidc,
    data: callData,
    gas: 10_000_000n, // TODO: Remove when gas estimation is fixed
    type: "eip712",
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
  } as any;

  const transactionHash = await sendTransaction(client, sendTransactionArgs);

  const transactionReceipt = await waitForTransactionReceipt(client, { hash: transactionHash });
  if (transactionReceipt.status !== "success") throw new Error("removeOidcAccount transaction reverted");

  return transactionReceipt;
};
